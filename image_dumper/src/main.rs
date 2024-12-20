use std::{
    fs::File,
    io::Write,
};

use obfstr::obfstr;
use pubg::{
    InterfaceError,
    Module,
    PubgHandle,
    StatePubgHandle,
    StatePubgMemory,
};
use utils_console::show_critical_error;
use utils_state::StateRegistry;
use utils_windows::version_info;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    if let Err(err) = real_main() {
        show_critical_error(&format!("{:#}", err));
    }
}

fn real_main() -> anyhow::Result<()> {
    let build_info = version_info()?;
    log::info!(
        "{} v{} ({}). Windows build {}.",
        obfstr!("Pubg_Valthrun-image-dumper"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH"),
        build_info.dwBuildNumber
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

    const BYTES_READ_COUNT: usize = 0x8;
    let mut buffer = vec![0u8; (module_size - 0x1000) as usize];
    for i in 0..(module_size - 0x1000) / BYTES_READ_COUNT {
        if i % 10000 == 0 {
            log::info!(
                "Reading module {}/{}",
                i + 1,
                (module_size - 0x1000) / BYTES_READ_COUNT
            );
        }
        let mut temp_buffer = [0u8; BYTES_READ_COUNT];
        match memory.read_memory(
            module_address + 0x1000 + (i * BYTES_READ_COUNT) as u64,
            &mut temp_buffer,
        ) {
            Ok(()) => {
                buffer[(i * BYTES_READ_COUNT)..i * BYTES_READ_COUNT + BYTES_READ_COUNT]
                    .copy_from_slice(&temp_buffer);
            }
            Err(err) => {
                log::info!(
                    "Reading module {}/{}",
                    i + 1,
                    (module_size - 0x1000) / BYTES_READ_COUNT
                );
                log::error!("Failed to read module: {}", err);
                continue;
            }
        }
    }

    let mut file = File::create(format!("dump1.bin"))?;
    file.write_all(&buffer)?;

    log::info!("Done");
    Ok(())
}
