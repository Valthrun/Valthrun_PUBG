#!/usr/bin/env python3
"""
Update PUBG offsets in the Rust project from pubg/src/schema/offsets.txt.

Usage:
  python scripts/update_offsets.py [--workspace E:\\Programming\\Valthrun_PUBG]

If --workspace is omitted, the current working directory is used.

This script updates:
  - Rust constants with trailing `// <Key>` comments, where <Key> is in offsets.txt
  - Specific constants without comments (e.g., HEALTH4)
  - #[field(offset = 0x...)] attributes in pubg/src/schema/client.rs
  - HEALTH_XOR_KEYS array in pubg/src/state/player.rs (HealthXorKeys0..15)

Writes files only if actual changes are needed.
"""

import argparse
import pathlib
import re
import sys
from typing import Dict, List, Tuple


OFFSETS_TXT_REL = pathlib.Path("pubg/src/schema/offsets.txt")

# Files to process
FILES_WITH_TRAILING_COMMENT_CONSTS = [
    pathlib.Path("pubg/src/schema/client.rs"),
    pathlib.Path("pubg/src/state/decrypt.rs"),
    pathlib.Path("pubg/src/state/gname_cache.rs"),
    pathlib.Path("pubg/src/state/player.rs"),
]

PLAYER_RS = pathlib.Path("pubg/src/state/player.rs")
CLIENT_RS = pathlib.Path("pubg/src/schema/client.rs")


def read_text(p: pathlib.Path) -> str:
    return p.read_text(encoding="utf-8")


def write_text_if_changed(p: pathlib.Path, new_text: str) -> bool:
    old = read_text(p)
    if old == new_text:
        return False
    p.write_text(new_text, encoding="utf-8")
    return True


def parse_offsets(offsets_txt: str) -> Dict[str, int]:
    pattern = re.compile(r"constexpr\s+uint64_t\s+([A-Za-z0-9_]+)\s*=\s*(0x[0-9A-Fa-f]+|\d+);")
    result: Dict[str, int] = {}
    for line in offsets_txt.splitlines():
        line = line.strip()
        if not line or line.startswith("//"):
            continue
        m = pattern.search(line)
        if not m:
            continue
        key = m.group(1)
        val_raw = m.group(2)
        try:
            if val_raw.lower().startswith("0x"):
                val = int(val_raw, 16)
            else:
                val = int(val_raw)
        except ValueError:
            continue
        result[key] = val
    return result


def fmt_hex(val: int, width: int = 0) -> str:
    # Use uppercase hex like 0x10FE9A98. Width is best-effort; if not provided, no zero-padding.
    if width > 0:
        return f"0x{val:0{width}X}"
    return f"0x{val:X}"


def update_constants_with_trailing_comments(src: str, offsets: Dict[str, int]) -> str:
    # Matches: pub const NAME: u64 = 0x...; // Key
    # or:      const NAME: u32 = 123; // Key
    pattern = re.compile(
        r"(?P<prefix>\b(?:pub\s+)?const\s+[A-Z0-9_]+\s*:\s*u(?:32|64)\s*=\s*)"  # prefix
        r"(?P<value>0x[0-9A-Fa-f]+|\d+)"                                           # value
        r"(?P<suffix>;\s*//\s*(?P<key>[A-Za-z0-9_]+))"                             # ; // Key
    )

    def repl(m: re.Match) -> str:
        key = m.group("key")
        if key not in offsets:
            return m.group(0)
        old_val = m.group("value")
        # Try to preserve width if hex
        width = 0
        if old_val.lower().startswith("0x"):
            hex_digits = old_val[2:]
            width = len(hex_digits)
        new_val = fmt_hex(offsets[key], width)
        if new_val == old_val:
            return m.group(0)
        return f"{m.group('prefix')}{new_val}{m.group('suffix')}"

    return pattern.sub(repl, src)


def update_health_xor_keys_in_player_rs(src: str, offsets: Dict[str, int]) -> str:
    # Update HEALTH_XOR_KEYS array from HealthXorKeys0..15
    xor_keys: List[int] = []
    have_all = all((f"HealthXorKeys{i}" in offsets) for i in range(16))
    if have_all:
        for i in range(16):
            xor_keys.append(offsets[f"HealthXorKeys{i}"])

        # Regex to capture full array block, preserving leading indent and header
        block_pat = re.compile(
            r"(^[\t ]*const\s+HEALTH_XOR_KEYS\s*:\s*\[u32;\s*16\]\s*=\s*\[)([\s\S]*?)(\];)",
            re.MULTILINE,
        )

        def block_repl(m: re.Match) -> str:
            header = m.group(1)
            trailer = m.group(3)
            # Determine indent from header line start
            indent_match = re.match(r"^[\t ]*", header)
            indent = indent_match.group(0) if indent_match else ""
            first = ", ".join(fmt_hex(v) for v in xor_keys[:8])
            second = ", ".join(fmt_hex(v) for v in xor_keys[8:])
            new_values = f"\n{indent}{first},\n{indent}{second},\n"
            return f"{header}{new_values}{trailer}"

        src = block_pat.sub(block_repl, src, count=1)

    return src


def update_client_rs_field_offsets(src: str, offsets: Dict[str, int]) -> str:
    # Mapping (Struct, field) -> offset key in offsets.txt
    mapping: Dict[Tuple[str, str], str] = {
        ("UWorld", "u_level"): "CurrentLevel",
        ("UWorld", "game_instance"): "GameInstance",
        ("ULevel", "actors"): "Actors",
        ("GameInstance", "local_player"): "LocalPlayers",
        ("ULocalPlayer", "player_controller"): "PlayerController",
        ("AActor", "root_component"): "RootComponent",
        ("AActor", "id"): "offset",
        ("APlayerController", "player_camera_manager"): "PlayerCameraManager",
        ("APawn", "last_team_num"): "LastTeamNum",
        ("ACharacter", "health_flag"): "Health0",
        ("ACharacter", "health"): "Health",
        ("ACharacter", "health1"): "Health1",
        ("ACharacter", "health2"): "Health2",
        ("ACharacter", "health3"): "Health3",
        ("ACharacter", "health5"): "Health5",
        ("ACharacter", "health6"): "Health6",
        ("ACharacter", "mesh"): "Mesh",
        ("USkeletalMeshComponent", "always_create_physics_state"): "bAlwaysCreatePhysicsState",
        ("APlayerCameraManager", "camera_rot"): "CameraRot",
        ("APlayerCameraManager", "camera_pos"): "CameraPos",
        ("USceneComponent", "relative_location"): "ComponentLocation",
    }

    lines = src.splitlines(keepends=True)
    out_lines: List[str] = []
    current_struct: str = ""
    i = 0
    while i < len(lines):
        line = lines[i]
        out_lines.append(line)

        # Track current struct name
        if line.startswith("pub struct "):
            m = re.match(r"pub\s+struct\s+([A-Za-z0-9_]+)", line)
            if m:
                current_struct = m.group(1)

        # Look for attribute line with offset
        attr_m = re.search(r"#\[field\(offset\s*=\s*(0x[0-9A-Fa-f]+|\d+)\)\]", line)
        if attr_m:
            # Peek the next non-empty, non-attribute line to find field name
            j = i + 1
            while j < len(lines) and lines[j].strip() == "":
                j += 1
            if j < len(lines):
                field_line = lines[j]
                field_m = re.search(r"pub\s+([A-Za-z0-9_]+)\s*:\s*", field_line)
                if field_m and current_struct:
                    field_name = field_m.group(1)
                    key = mapping.get((current_struct, field_name))
                    if key and key in offsets:
                        old_val = attr_m.group(1)
                        width = len(old_val[2:]) if old_val.lower().startswith("0x") else 0
                        new_val = fmt_hex(offsets[key], width)
                        if new_val != old_val:
                            new_attr = re.sub(
                                r"(offset\s*=\s*)(0x[0-9A-Fa-f]+|\d+)",
                                lambda m2: f"{m2.group(1)}{new_val}",
                                line,
                            )
                            out_lines[-1] = new_attr
        i += 1

    return "".join(out_lines)


def _type_size_bytes(type_str: str) -> int:
    t = type_str.strip()
    # Pointer wrappers are 8 bytes on 64-bit
    if t.startswith("EncryptedPtr64<") or t.startswith("Ptr64<"):
        return 8
    # Fixed-size arrays like [f32; 3]
    m = re.match(r"^\[\s*(u8|u32|f32)\s*;\s*(\d+)\s*\]$", t)
    if m:
        base = m.group(1)
        count = int(m.group(2))
        base_size = {"u8": 1, "u32": 4, "f32": 4}[base]
        return base_size * count
    # Primitives
    if t in ("u8", "u32", "f32"):
        return {"u8": 1, "u32": 4, "f32": 4}[t]
    # Default conservative size for unknowns
    return 8


def update_client_rs_struct_sizes(src: str) -> str:
    lines = src.splitlines(keepends=True)
    out_lines: List[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        m_attr = re.search(r"#\[raw_struct\(size\s*=\s*(0x[0-9A-Fa-f]+|\d+)\)\]", line)
        if not m_attr:
            out_lines.append(line)
            i += 1
            continue

        # Found a raw_struct attribute line; tentatively append, possibly replaced later
        old_attr_line = line

        # Find struct header line after attribute
        j = i + 1
        while j < len(lines) and lines[j].strip() == "":
            j += 1
        if j >= len(lines) or not lines[j].lstrip().startswith("pub struct "):
            # Not a struct; just append as-is
            out_lines.append(line)
            i += 1
            continue

        # Determine struct body end '}' line index
        k = j + 1
        max_end = 0
        while k < len(lines):
            l = lines[k]
            # Stop at closing brace of struct
            if l.strip().startswith('}'):
                break
            # Match field attribute
            m_field_attr = re.search(r"#\[field\(offset\s*=\s*(0x[0-9A-Fa-f]+|\d+)\)\]", l)
            if m_field_attr:
                # Next non-empty line should be field declaration
                t = k + 1
                while t < len(lines) and lines[t].strip() == "":
                    t += 1
                if t < len(lines):
                    field_decl = lines[t].strip()
                    m_decl = re.search(r"pub\s+[A-Za-z0-9_]+\s*:\s*([^,]+)", field_decl)
                    if m_decl:
                        type_str = m_decl.group(1).strip()
                        offset_str = m_field_attr.group(1)
                        offset = int(offset_str, 16) if offset_str.lower().startswith("0x") else int(offset_str)
                        size_b = _type_size_bytes(type_str)
                        end = offset + size_b
                        if end > max_end:
                            max_end = end
            k += 1

        # Compute new size and replace in attribute line if needed
        if max_end > 0:
            old_val = m_attr.group(1)
            width = len(old_val[2:]) if old_val.lower().startswith("0x") else 0
            new_val = fmt_hex(max_end, width)
            if new_val != old_val:
                line = re.sub(
                    r"(size\s*=\s*)(0x[0-9A-Fa-f]+|\d+)",
                    lambda m2: f"{m2.group(1)}{new_val}",
                    old_attr_line,
                )
        out_lines.append(line)

        # Advance to next line (struct header and body will be appended in subsequent iterations)
        i += 1
        
    return "".join(out_lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Update PUBG offsets from offsets.txt")
    parser.add_argument("--workspace", type=str, default=str(pathlib.Path.cwd()), help="Project root directory")
    args = parser.parse_args()

    root = pathlib.Path(args.workspace)
    offsets_txt_path = root / OFFSETS_TXT_REL
    if not offsets_txt_path.exists():
        print(f"ERROR: Offsets file not found: {offsets_txt_path}", file=sys.stderr)
        return 1

    offsets = parse_offsets(read_text(offsets_txt_path))
    if not offsets:
        print("ERROR: No offsets parsed from offsets.txt", file=sys.stderr)
        return 1

    changed_files: List[pathlib.Path] = []

    # 1) Update trailing comment constants in designated files
    for rel in FILES_WITH_TRAILING_COMMENT_CONSTS:
        p = root / rel
        if not p.exists():
            continue
        new_src = update_constants_with_trailing_comments(read_text(p), offsets)
        if write_text_if_changed(p, new_src):
            changed_files.append(rel)

    # 2) Update specific constants and XOR keys in player.rs
    p_player = root / PLAYER_RS
    if p_player.exists():
        new_src = update_health_xor_keys_in_player_rs(read_text(p_player), offsets)
        if write_text_if_changed(p_player, new_src):
            changed_files.append(PLAYER_RS)

    # 3) Update #[field(offset=...)] attributes, then recompute struct sizes in client.rs
    p_client = root / CLIENT_RS
    if p_client.exists():
        src_client = read_text(p_client)
        new_src = update_client_rs_field_offsets(src_client, offsets)
        new_src = update_client_rs_struct_sizes(new_src)
        if write_text_if_changed(p_client, new_src):
            changed_files.append(CLIENT_RS)

    if changed_files:
        print("Updated:")
        for rel in changed_files:
            print(f" - {rel}")
    else:
        print("No changes required. All offsets are up to date.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())


