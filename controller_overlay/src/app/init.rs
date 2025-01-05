use std::{
    rc::Rc,
    cell::RefCell,
    error::Error,
};

use anyhow::Context;
use obfstr::obfstr;
use overlay::{OverlayOptions, OverlayError, VulkanError, LoadingError, System};
use pubg::{PubgHandle, InterfaceError, StatePubgHandle, StatePubgMemory};
use rand::{thread_rng, RngCore};
use utils_console;
use utils_state::StateRegistry;
use utils_windows::version_info;

use crate::{
    settings::load_app_settings,
    enhancements::PlayerSpyer,
    app::fonts::AppFonts,
};

use super::{types::Application, settings_manager::SettingsManager};

pub fn initialize_app() -> anyhow::Result<(System, Rc<RefCell<Application>>)> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let build_info = version_info()?;
    log::info!(
        "{} v{} ({}). Windows build {}.",
        obfstr!("Valthrun_PUBG"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH"),
        build_info.dwBuildNumber
    );
    log::info!("{} {}", obfstr!("Build time:"), env!("BUILD_TIME"));

    let settings = load_app_settings()?;
    log::info!(
        "Using manual monitor selection: {}",
        settings.selected_monitor
    );
    let selected_monitor = Some(settings.selected_monitor);

    let pubg = match PubgHandle::create(false) {
        Ok(pubg) => pubg,
        Err(err) => {
            if let Some(interface_err) = err.downcast_ref::<InterfaceError>() {
                if let Some(detailed_message) = interface_err.detailed_message() {
                    utils_console::show_critical_error(&detailed_message);
                }
            }
            return Err(err);
        }
    };

    pubg.add_metrics_record(obfstr!("Valthrun_PUBG-status"), "initializing");

    let mut states = StateRegistry::new(1024 * 8);
    states.set(StatePubgHandle::new(pubg.clone()), ())?;
    states.set(StatePubgMemory::new(pubg.create_memory_view()), ())?;
    states.set(settings, ())?;

    log::debug!("Initialize overlay");
    let app_fonts = AppFonts::new();

    let mut rng = thread_rng();
    let random_app_name = format!("{:x}", rng.next_u64());

    let overlay_options = OverlayOptions {
        title: random_app_name,
        register_fonts_callback: Some(AppFonts::create_font_config_callback(app_fonts.clone())),
        monitor: selected_monitor,
    };

    let mut overlay = match overlay::init(overlay_options) {
        Err(OverlayError::Vulkan(VulkanError::DllNotFound(LoadingError::LibraryLoadFailure(
            source,
        )))) => {
            let message = match &source {
                libloading::Error::LoadLibraryExW { .. } => {
                    let error = source.source().context("LoadLibraryExW to have a source")?;
                    format!("Failed to load vulkan-1.dll.\nError: {:#}", error)
                }
                error => {
                    format!(
                        "An error occurred while loading vulkan-1.dll.\nError: {:#}",
                        error
                    )
                }
            };
            utils_console::show_critical_error(&message);
            return Err(anyhow::anyhow!("Failed to load vulkan-1.dll"));
        }
        value => value?,
    };

    let app = Application {
        fonts: app_fonts,
        states,
        pubg: pubg.clone(),
        enhancements: vec![Rc::new(RefCell::new(PlayerSpyer {}))],
        settings_manager: SettingsManager::new(),
        frame_read_calls: 0,
    };

    {
        let settings = app.settings();
        if let Some(imgui_settings) = &settings.imgui {
            overlay.imgui.load_ini_settings(imgui_settings);
        }
    }

    let app = Rc::new(RefCell::new(app));

    pubg.add_metrics_record(
        obfstr!("Valthrun_PUBG-status"),
        &format!(
            "initialized, vesion: {}, git-hash: {}, win-build: {}",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH"),
            build_info.dwBuildNumber
        ),
    );

    log::info!("App initialized.");

    Ok((overlay, app))
} 