use std::{
    cell::RefCell,
    sync::atomic::{
        AtomicBool,
        Ordering,
    },
};

use crate::settings::SettingsUI;

pub struct SettingsManager {
    visible: bool,
    dirty: bool,
    ui: RefCell<SettingsUI>,
    screen_capture_changed: AtomicBool,
    monitor_changed: AtomicBool,
    render_debug_window_changed: AtomicBool,
}

impl SettingsManager {
    pub fn new() -> Self {
        Self {
            visible: false,
            dirty: false,
            ui: RefCell::new(SettingsUI::new()),
            screen_capture_changed: AtomicBool::new(true),
            monitor_changed: AtomicBool::new(true),
            render_debug_window_changed: AtomicBool::new(true),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        if !visible {
            self.dirty = true;
        }
    }

    pub fn toggle_visible(&mut self) {
        self.set_visible(!self.visible);
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn ui(&self) -> &RefCell<SettingsUI> {
        &self.ui
    }

    pub fn screen_capture_changed(&self) -> bool {
        self.screen_capture_changed.swap(false, Ordering::Relaxed)
    }

    pub fn monitor_changed(&self) -> bool {
        self.monitor_changed.swap(false, Ordering::Relaxed)
    }

    pub fn render_debug_window_changed(&self) -> bool {
        self.render_debug_window_changed
            .swap(false, Ordering::Relaxed)
    }

    pub fn mark_screen_capture_changed(&self) {
        self.screen_capture_changed.store(true, Ordering::Relaxed);
    }

    pub fn mark_monitor_changed(&self) {
        self.monitor_changed.store(true, Ordering::Relaxed);
    }

    pub fn mark_render_debug_window_changed(&self) {
        self.render_debug_window_changed
            .store(true, Ordering::Relaxed);
    }

    pub fn render(
        &self,
        app: &crate::app::Application,
        ui: &imgui::Ui,
        unicode_text: &overlay::UnicodeTextRenderer,
    ) {
        if self.visible {
            let mut settings_ui = self.ui.borrow_mut();
            settings_ui.render(app, ui, unicode_text)
        }
    }
}
