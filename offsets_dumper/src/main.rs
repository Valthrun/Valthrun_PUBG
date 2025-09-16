use std::fmt::Write;

use obfstr::obfstr;
use pubg::{
    InterfaceError,
    Module,
    PubgHandle,
    SearchPattern,
    Signature,
    SignatureType,
    StatePubgHandle,
    StatePubgMemory,
};
use utils_common::get_os_info;
use utils_console::show_critical_error;
use utils_state::StateRegistry;

#[derive(Clone, Copy)]
enum SelectMode {
    First,
    Last,
}

struct SigEntry {
    spec: Signature,
    select: SelectMode,
}

fn main() {
    env_logger::Builder::from_default_env()
        .init();
    if let Err(err) = real_main() {
        show_critical_error(&format!("{:#}", err));
    }
}

fn find_last(
    handle: &PubgHandle,
    base: u64,
    size: usize,
    pattern: &dyn SearchPattern,
) -> anyhow::Result<Option<u64>> {
    let mut start_offset = 0usize;
    let mut last: Option<u64> = None;

    loop {
        if start_offset >= size {
            break;
        }
        let address = base + start_offset as u64;
        let length = size - start_offset;
        match handle.find_pattern(address, length, pattern)? {
            Some(found) => {
                last = Some(found);
                let next = (found - base) as usize + 1;
                if next <= start_offset {
                    break;
                }
                start_offset = next;
            }
            None => break,
        }
    }

    Ok(last)
}

fn read_u32_le(mem: &StatePubgMemory, address: u64) -> anyhow::Result<u32> {
    let mut buf = [0u8; 4];
    mem.read_memory(address, &mut buf)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(u32::from_le_bytes(buf))
}

fn resolve_sig_entry(
    handle: &PubgHandle,
    memory: &StatePubgMemory,
    module_base: u64,
    module_size: usize,
    entry: &SigEntry,
) -> anyhow::Result<Option<u64>> {
    match entry.select {
        SelectMode::First => {
            let value = handle
                .resolve_signature(Module::Game, &entry.spec)
                .map_err(|e| {
                    anyhow::anyhow!(
                        "resolve_signature for {} failed: {}",
                        entry.spec.debug_name,
                        e
                    )
                })?;
            Ok(Some(value))
        }
        SelectMode::Last => {
            let inst_addr = find_last(handle, module_base, module_size, &*entry.spec.pattern)
                .map_err(|e| {
                    anyhow::anyhow!(
                        "pattern search (last) for {} failed: {}",
                        entry.spec.debug_name,
                        e
                    )
                })?;
            let Some(inst_addr) = inst_addr else {
                return Ok(None);
            };

            let value = match entry.spec.value_type {
                SignatureType::RelativeAddress { inst_length } => {
                    let disp = read_u32_le(memory, inst_addr + entry.spec.offset).map_err(|e| {
                        anyhow::anyhow!(
                            "read displacement for {} at {:#x} failed: {}",
                            entry.spec.debug_name,
                            inst_addr + entry.spec.offset,
                            e
                        )
                    })? as u64;
                    Some(inst_addr + disp + inst_length)
                }
                SignatureType::Offset => {
                    let off = read_u32_le(memory, inst_addr + entry.spec.offset).map_err(|e| {
                        anyhow::anyhow!(
                            "read offset for {} at {:#x} failed: {}",
                            entry.spec.debug_name,
                            inst_addr + entry.spec.offset,
                            e
                        )
                    })? as u64;
                    Some(off)
                }
            };

            Ok(value)
        }
    }
}

fn real_main() -> anyhow::Result<()> {
    let build_info = get_os_info()?;
    let platform_info = if build_info.is_windows {
        format!("Windows build {}", build_info.build_number)
    } else {
        format!(
            "Linux kernel {}.{}.{}",
            build_info.major_version, build_info.minor_version, build_info.build_number
        )
    };
    log::info!(
        "{} v{} ({}). {}.",
        obfstr!("Pubg_Valthrun-offsets-dumper"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH"),
        platform_info
    );
    log::info!("{} {}", obfstr!("Build time:"), env!("BUILD_TIME"));

    let pubg = match PubgHandle::create(false) {
        Ok(pubg) => pubg,
        Err(err) => {
            if let Some(err) = err.downcast_ref::<InterfaceError>() {
                if let Some(detailed_message) = err.detailed_message() {
                    show_critical_error(&detailed_message);
                    return Ok(());
                }
            }

            return Err(err);
        }
    };

    let mut states = StateRegistry::new(1024 * 8);
    states.set(StatePubgHandle::new(pubg.clone()), ())?;
    states.set(StatePubgMemory::new(pubg.create_memory_view()), ())?;

    log::info!("Initialized.");

    let memory = states.resolve::<StatePubgMemory>(())?;
    let handle = states.resolve::<StatePubgHandle>(())?;

    let module_size = handle.module_size(Module::Game)? as usize;
    let module_address = handle.memory_address(Module::Game, 0x0)?;
    log::info!("Module size: {:#x}", module_size);
    log::info!("Module address: {:#x}", module_address);

    // Hardcoded signatures (subset of the provided list)
    let signatures: Vec<SigEntry> = vec![
        SigEntry {
            spec: Signature::relative_address(
                "GNames",
                "48 89 05 ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? E9 ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 83 3D ? ? ? ? ? 0F 85 ? ? ? ? 48 8D 0D ? ? ? ? 48 83 3D ? ? ? ? ? 75 13 48 8B D1 B9 ? ? ? ? 48 8B 05 ? ? ? ? FF D0 EB 35",
                3,
                7,
            ),
            select: SelectMode::First,
        },
        SigEntry {
            spec: Signature::relative_address(
                "UWorld",
                "48 89 05 ? ? ? ? 48 83 C4 28 C3 C7 44 24 ? ? ? ? ? C7 44 24 ? ? ? ? ? 48 8B 44 24 ? 48 89 05 ? ? ? ? 48 83 C4 28 C3",
                3,
                7,
            ),
            select: SelectMode::Last,
        },
        SigEntry {
            spec: Signature::offset(
                "CurrentLevel",
                "49 8B 56 50 4D 85 C0 75 0D 8B CE 48 8B 05 ? ? ? ? FF D0 EB 29",
                3,
            ),
            select: SelectMode::First,
        },
        SigEntry {
            spec: Signature::offset("GameInstance", "4C 8B ?? ? ? ? ? 4D 85 ?? 75 ??", 3),
            select: SelectMode::First,
        },
        SigEntry {
            spec: Signature::offset("LocalPlayers", "48 8B 86 ? ? ? ? 48 8B 0C D8 4D 85 C0", 3),
            select: SelectMode::First,
        },
        SigEntry {
            spec: Signature::offset(
                "Actors",
                "48 8B 88 ? ? ? ? 48 39 1D ? ? ? ? 75 13 48 8B D1 B9 ? ? ? ? 48 8B 05 ? ? ? ? FF D0 EB 38",
                3,
            ),
            select: SelectMode::First,
        },
        SigEntry {
            spec: Signature::offset(
                "AcknowledgedPawn",
                "48 8B 83 B8 04 00 00 48 89 84 24 90 00 00 00 4D 85 C0 75 10 48 8B D0 8B CE 48 8B 05 ? ? ? ? FF D0 EB 42",
                3,
            ),
            select: SelectMode::First,
        },
        SigEntry {
            spec: Signature::offset(
                "PlayerCameraManager",
                "48 8B 88 ? ? ? ? 48 8B 01 FF 90 ? ? ? ? F3 41 0F 11 46 ? 48 83 3D ? ? ? ? ? 48 8B 0B",
                3,
            ),
            select: SelectMode::First,
        },
        SigEntry {
            spec: Signature::offset(
                "RootComponent",
                "4C 8B 97 ? ? ? ? 4D 85 C9 75 19 48 8B 05 ? ? ? ? 49 8B D2 B9 ? ? ? ? FF D0 48 8B C8 E9 ? ? ? ?",
                3,
            ),
            select: SelectMode::Last,
        },
    ];

    log::info!("Scanning signatures via handle.find_pattern...");
    let mut out = String::new();
    for entry in &signatures {
        match resolve_sig_entry(&handle, &memory, module_address, module_size, entry) {
            Ok(Some(value)) => match entry.spec.value_type {
                SignatureType::RelativeAddress { .. } => {
                    log::info!(
                        "{}: abs {:#x} (RVA {:#x})",
                        entry.spec.debug_name,
                        value,
                        value - module_address
                    );
                    let _ = writeln!(out, "{} = 0x{:X} (abs)", entry.spec.debug_name, value);
                }
                SignatureType::Offset => {
                    log::info!("{}: offset {:#x}", entry.spec.debug_name, value);
                    let _ = writeln!(out, "{} = 0x{:X}", entry.spec.debug_name, value);
                }
            },
            Ok(None) => {
                log::warn!("{}: not found", entry.spec.debug_name);
                let _ = writeln!(out, "{} = NOT_FOUND", entry.spec.debug_name);
            }
            Err(err) => {
                let msg = format!("{}", err);
                if msg.to_lowercase().contains("paged out") {
                    log::error!(
                        "{}: failed due to paged out memory: {}",
                        entry.spec.debug_name,
                        msg
                    );
                } else {
                    log::error!("{}: failed: {}", entry.spec.debug_name, msg);
                }
                let _ = writeln!(out, "{} = ERROR: {}", entry.spec.debug_name, msg);
            }
        }
    }

    std::fs::write("offsets.txt", out)?;

    log::info!("Done");
    Ok(())
}
