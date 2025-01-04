use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use imgui::{Condition, FontId};
use obfstr::obfstr;
use overlay::{SystemRuntimeController, UnicodeTextRenderer};
use pubg::PubgHandle;
use utils_state::StateRegistry;
use winit::window::Window;

use crate::{
    enhancements::Enhancement,
    settings::{save_app_settings, AppSettings, SettingsUI},
    view::ViewController,
};

use super::input::KeyboardInput;

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
    pub valthrun: FontReference,
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

    pub fn update(&mut self, ui: &imgui::Ui) -> anyhow::Result<()> {
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