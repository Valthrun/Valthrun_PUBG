use std::cell::{
    Ref,
    RefMut,
};

use imgui::Condition;
use obfstr::obfstr;
use overlay::{
    SystemRuntimeController,
    UnicodeTextRenderer,
};
use pubg::state::StateActorLists;
use winit::window::Window;

use super::types::{
    Application,
    UpdateContext,
};
use crate::{
    settings::{
        save_app_settings,
        AppSettings,
    },
    view::ViewController,
};

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
        if self.settings_manager.is_dirty() {
            self.settings_manager.set_dirty(false);
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

        if self.settings_manager.monitor_changed() {
            let settings = self.settings();
            overlay::System::switch_monitor(&overlay_window, settings.selected_monitor);
            log::debug!("Updating monitor to {}", settings.selected_monitor);
        }

        if self.settings_manager.screen_capture_changed() {
            let settings = self.settings();
            controller.toggle_screen_capture_visibility(!settings.hide_overlay_from_screen_capture);
            log::debug!(
                "Updating screen capture visibility to {}",
                !settings.hide_overlay_from_screen_capture
            );
        }

        if self.settings_manager.render_debug_window_changed() {
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
                    self.settings_manager.set_dirty(true);
                }
            }
        }

        if ui.is_key_pressed_no_repeat(self.settings().key_settings.0) {
            log::debug!("Toggle settings");
            self.settings_manager.toggle_visible();
            self.pubg.add_metrics_record(
                "settings-toggled",
                &format!("visible: {}", self.settings_manager.is_visible()),
            );
        }

        self.states.invalidate_states();
        if let Ok(mut view_controller) = self.states.resolve_mut::<ViewController>(()) {
            view_controller.update_screen_bounds(mint::Vector2::from_slice(&ui.io().display_size));
        }

        let update_context = UpdateContext {
            states: &self.states,
        };

        for enhancement in &self.enhancements {
            let mut enhancement = enhancement.borrow_mut();
            enhancement.update(&update_context)?;
        }

        if ui.is_key_pressed(imgui::Key::Comma) {
            log::info!("Clearing actor lists");
            self.states.resolve_mut::<StateActorLists>(())?.clear();
        }

        if ui.is_key_pressed(imgui::Key::P) {
            let read_call_stats = self.pubg.ke_interface.get_read_slice_stats();
            let stats_content = read_call_stats
                .iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect::<Vec<String>>()
                .join("\n\n");
            if let Err(e) = std::fs::write("read_call_stats.txt", stats_content) {
                log::error!("Failed to write stats to file: {}", e);
            } else {
                log::info!("Read call stats saved to read_call_stats.txt");
            }
        }

        self.pubg.ke_interface.clear_read_slice_stats();

        let read_calls = self.pubg.ke_interface.total_read_calls();
        self.frame_read_calls = read_calls - self.last_total_read_calls;
        self.last_total_read_calls = read_calls;
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

        self.settings_manager.render(self, ui, unicode_text);
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
