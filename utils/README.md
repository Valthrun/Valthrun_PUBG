# Valthrun Utilities

This directory contains utility crates that provide common functionality for the Valthrun project.

## Module Structure

### utils-common

The primary interface that applications should use. It provides a unified API that abstracts over platform-specific differences.

Features:
- `windows` - Enables Windows-specific functionality
- `linux` - Enables Linux-specific functionality

### utils-windows

Windows-specific utilities and implementations. This is used internally by `utils-common` when the `windows` feature is enabled.

### utils-linux 

Linux-specific utilities and implementations. This is used internally by `utils-common` when the `linux` feature is enabled.

### utils-state

Provides state management utilities used throughout the application.

### utils-console

Provides console/logging utilities used throughout the application.

## Usage Example

In your application's Cargo.toml:

```toml
[dependencies]
utils-common = { path = "../utils/common" }

[target.'cfg(target_os = "windows")'.dependencies]
utils-common = { path = "../utils/common", features = ["windows"] }

[target.'cfg(target_os = "linux")'.dependencies]
utils-common = { path = "../utils/common", features = ["linux"] }
```

In your code:

```rust
use utils_common::get_os_info;

fn main() -> anyhow::Result<()> {
    let os_info = get_os_info()?;
    
    let platform_info = if os_info.is_windows {
        format!("Windows build {}", os_info.build_number)
    } else {
        format!("Linux kernel {}.{}.{}", 
                os_info.major_version, 
                os_info.minor_version, 
                os_info.build_number)
    };
    
    println!("Running on: {}", platform_info);
    
    Ok(())
}
``` 