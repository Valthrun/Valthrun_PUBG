use std::{
    cell::{
        Ref,
        RefCell,
        RefMut,
    },
    env,
    error::Error,
    rc::Rc,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
    time::{
        Duration,
        Instant,
    },
};

use anyhow::Context;
use enhancements::{
    Enhancement,
    PlayerSpyer,
};
use imgui::{
    Condition,
    FontConfig,
    FontId,
    FontSource,
    Ui,
};
use obfstr::obfstr;
use overlay::{
    LoadingError,
    OverlayError,
    OverlayOptions,
    SystemRuntimeController,
    UnicodeTextRenderer,
    VulkanError,
};
use pubg::{
    InterfaceError,
    PubgHandle,
    StatePubgHandle,
    StatePubgMemory,
};
use rand::{
    thread_rng,
    RngCore,
};
use settings::{
    load_app_settings,
    save_app_settings,
    AppSettings,
    SettingsUI,
};
use utils_state::StateRegistry;
use utils_windows::version_info;
use view::ViewController;
use winit::window::Window;

mod enhancements;
mod settings;
mod utils;
mod view;

pub trait KeyboardInput {
    fn is_key_down(&self, key: imgui::Key) -> bool;
    fn is_key_pressed(&self, key: imgui::Key, repeating: bool) -> bool;
}

impl KeyboardInput for imgui::Ui {
    fn is_key_down(&self, key: imgui::Key) -> bool {
        Ui::is_key_down(self, key)
    }

    fn is_key_pressed(&self, key: imgui::Key, repeating: bool) -> bool {
        if repeating {
            Ui::is_key_pressed(self, key)
        } else {
            Ui::is_key_pressed_no_repeat(self, key)
        }
    }
}

pub struct UpdateContext<'a> {
    pub input: &'a dyn KeyboardInput,
    pub states: &'a StateRegistry,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FontReference {
    inner: Arc<RefCell<Option<FontId>>>,
}

impl FontReference {
    pub fn font_id(&self) -> Option<FontId> {
        self.inner.borrow().clone()
    }

    pub fn set_id(&self, font_id: FontId) {
        *self.inner.borrow_mut() = Some(font_id);
    }
}

#[derive(Clone, Default)]
pub struct AppFonts {
    valthrun: FontReference,
}

pub struct Application {
    pub fonts: AppFonts,
    pub states: StateRegistry,

    pub pubg: Arc<PubgHandle>,
    pub enhancements: Vec<Rc<RefCell<dyn Enhancement>>>,

    pub settings_visible: bool,
    pub settings_dirty: bool,
    pub settings_ui: RefCell<SettingsUI>,
    pub settings_screen_capture_changed: AtomicBool,
    pub settings_monitor_changed: AtomicBool,
    pub settings_render_debug_window_changed: AtomicBool,

    pub frame_read_calls: usize,
}

impl Application {
    pub fn settings(&self) -> Ref<'_, AppSettings> {
        self.states
            .get::<AppSettings>(())
            .expect("app settings to be present")
    }

    pub fn settings_mut(&self) -> RefMut<'_, AppSettings> {
        self.states
            .get_mut::<AppSettings>(())
            .expect("app settings to be present")
    }

    pub fn pre_update(
        &mut self,
        controller: &mut SystemRuntimeController,
        overlay_window: &Window,
    ) -> anyhow::Result<()> {
        if self.settings_dirty {
            self.settings_dirty = false;
            let mut settings = self.settings_mut();

            settings.imgui = None;
            if let Ok(value) = serde_json::to_string(&*settings) {
                self.pubg.add_metrics_record("settings-updated", &value);
            }

            let mut imgui_settings = String::new();
            controller.imgui.save_ini_settings(&mut imgui_settings);
            settings.imgui = Some(imgui_settings);

            if let Err(error) = save_app_settings(&*settings) {
                log::warn!("Failed to save user settings: {}", error);
            };
        }

        if self.settings_monitor_changed.swap(false, Ordering::Relaxed) {
            let settings = self.settings();
            overlay::System::switch_monitor(&overlay_window, settings.selected_monitor);
            log::debug!("Updating monitor to {}", settings.selected_monitor);
        }

        if self
            .settings_screen_capture_changed
            .swap(false, Ordering::Relaxed)
        {
            let settings = self.settings();
            controller.toggle_screen_capture_visibility(!settings.hide_overlay_from_screen_capture);
            log::debug!(
                "Updating screen capture visibility to {}",
                !settings.hide_overlay_from_screen_capture
            );
        }

        if self
            .settings_render_debug_window_changed
            .swap(false, Ordering::Relaxed)
        {
            let settings = self.settings();
            controller.toggle_debug_overlay(settings.render_debug_window);
        }

        Ok(())
    }

    fn update(&mut self, ui: &imgui::Ui) -> anyhow::Result<()> {
        {
            for enhancement in self.enhancements.iter() {
                let mut hack = enhancement.borrow_mut();
                if hack.update_settings(ui, &mut *self.settings_mut())? {
                    self.settings_dirty = true;
                }
            }
        }

        if ui.is_key_pressed_no_repeat(self.settings().key_settings.0) {
            log::debug!("Toogle settings");
            self.settings_visible = !self.settings_visible;
            self.pubg.add_metrics_record(
                "settings-toggled",
                &format!("visible: {}", self.settings_visible),
            );

            if !self.settings_visible {
                /* overlay has just been closed */
                self.settings_dirty = true;
            }
        }

        self.states.invalidate_states();
        if let Ok(mut view_controller) = self.states.resolve_mut::<ViewController>(()) {
            view_controller.update_screen_bounds(mint::Vector2::from_slice(&ui.io().display_size));
        }

        let update_context = UpdateContext {
            states: &self.states,
            input: ui,
        };

        for enhancement in &self.enhancements {
            let mut enhancement = enhancement.borrow_mut();
            enhancement.update(&update_context)?;
        }

        let read_calls = self.pubg.ke_interface.total_read_calls();
        self.frame_read_calls = read_calls;

        Ok(())
    }

    pub fn render(&self, ui: &imgui::Ui, unicode_text: &UnicodeTextRenderer) {
        ui.window("overlay")
            .draw_background(false)
            .no_decoration()
            .no_inputs()
            .size(ui.io().display_size, Condition::Always)
            .position([0.0, 0.0], Condition::Always)
            .build(|| self.render_overlay(ui, unicode_text));

        {
            for enhancement in self.enhancements.iter() {
                let mut enhancement = enhancement.borrow_mut();
                if let Err(err) = enhancement.render_debug_window(&self.states, ui, unicode_text) {
                    log::error!("{:?}", err);
                }
            }
        }

        if self.settings_visible {
            let mut settings_ui = self.settings_ui.borrow_mut();
            settings_ui.render(self, ui, unicode_text)
        }
    }

    fn render_overlay(&self, ui: &imgui::Ui, unicode_text: &UnicodeTextRenderer) {
        let settings = self.settings();

        if settings.valthrun_watermark {
            {
                let text_buf;
                let text = obfstr!(text_buf = "Valthrun Overlay");

                ui.set_cursor_pos([
                    ui.window_size()[0] - ui.calc_text_size(text)[0] - 10.0,
                    10.0,
                ]);
                ui.text(text);
            }
            {
                let text = format!("{:.2} FPS", ui.io().framerate);
                ui.set_cursor_pos([
                    ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
                    24.0,
                ]);
                ui.text(text)
            }
            {
                let text = format!("{} Reads", self.frame_read_calls);
                ui.set_cursor_pos([
                    ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
                    38.0,
                ]);
                ui.text(text)
            }
        }

        for enhancement in self.enhancements.iter() {
            let hack = enhancement.borrow();
            if let Err(err) = hack.render(&self.states, ui, unicode_text) {
                log::error!("{:?}", err);
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    //utils_console::init().expect("Failed to initialize logger");
    if let Err(err) = real_main() {
        utils_console::show_critical_error(&format!("{:#}", err));
        //utils_console::flush_frame_logs();
    }
}

fn real_main() -> anyhow::Result<()> {
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
            if let Some(err) = err.downcast_ref::<InterfaceError>() {
                if let Some(detailed_message) = err.detailed_message() {
                    utils_console::show_critical_error(&detailed_message);
                    //utils_console::flush_frame_logs();
                    return Ok(());
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
    let app_fonts: AppFonts = Default::default();

    let mut rng = thread_rng();
    let random_app_name = format!("{:x}", rng.next_u64());

    let overlay_options = OverlayOptions {
        title: random_app_name,
        register_fonts_callback: Some(Box::new({
            let app_fonts = app_fonts.clone();

            move |atlas| {
                let font_size = 18.0;
                let valthrun_font = atlas.add_font(&[FontSource::TtfData {
                    data: include_bytes!("../resources/Valthrun-Regular.ttf"),
                    size_pixels: font_size,
                    config: Some(FontConfig {
                        rasterizer_multiply: 1.5,
                        oversample_h: 4,
                        oversample_v: 4,
                        ..FontConfig::default()
                    }),
                }]);

                app_fonts.valthrun.set_id(valthrun_font);
            }
        })),
        monitor: selected_monitor,
    };

    let mut overlay = match overlay::init(overlay_options) {
        Err(OverlayError::Vulkan(VulkanError::DllNotFound(LoadingError::LibraryLoadFailure(
            source,
        )))) => {
            match &source {
                libloading::Error::LoadLibraryExW { .. } => {
                    let error = source.source().context("LoadLibraryExW to have a source")?;
                    let message = format!("Failed to load vulkan-1.dll.\nError: {:#}", error);
                    utils_console::show_critical_error(&message);
                }
                error => {
                    let message = format!(
                        "An error occurred while loading vulkan-1.dll.\nError: {:#}",
                        error
                    );
                    utils_console::show_critical_error(&message);
                }
            }
            return Ok(());
        }
        value => value?,
    };

    {
        let settings = states.resolve::<AppSettings>(())?;
        if let Some(imgui_settings) = &settings.imgui {
            overlay.imgui.load_ini_settings(imgui_settings);
        }
    }

    let app = Application {
        fonts: app_fonts,
        states,

        pubg: pubg.clone(),

        enhancements: vec![Rc::new(RefCell::new(PlayerSpyer {}))],

        settings_visible: false,
        settings_dirty: false,
        settings_ui: RefCell::new(SettingsUI::new()),
        settings_screen_capture_changed: AtomicBool::new(true),
        settings_monitor_changed: AtomicBool::new(true),
        settings_render_debug_window_changed: AtomicBool::new(true),

        frame_read_calls: 0,
    };
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

    let mut update_fail_count = 0;
    let mut update_timeout: Option<(Instant, Duration)> = None;

    overlay.main_loop(
        {
            let app = app.clone();
            move |controller, window| {
                let mut app = app.borrow_mut();
                if let Err(err) = app.pre_update(controller, window) {
                    utils_console::show_critical_error(&format!("{:#}", err));
                    false
                } else {
                    true
                }
            }
        },
        move |ui, unicode_text| {
            let mut app = app.borrow_mut();

            if let Some((timeout, target)) = &update_timeout {
                if timeout.elapsed() > *target {
                    update_timeout = None;
                } else {
                    /* Not updating. On timeout... */
                    return true;
                }
            }

            if let Err(err) = app.update(ui) {
                /*if update_fail_count >= 10 {
                    log::error!("Over 10 errors occurred. Waiting 1s and try again.");
                    log::error!("Last error: {:#}", err);

                    update_timeout = Some((Instant::now(), Duration::from_millis(1000)));
                    update_fail_count = 0;
                    utils_console::flush_frame_logs();
                    return true;
                } else {
                    update_fail_count += 1;
                }*/
            }

            app.render(ui, unicode_text);

            // Update display
            //utils_console::flush_frame_logs();
            true
        },
    );

    Ok(())
}
