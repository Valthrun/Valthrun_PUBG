use anyhow::Result;
use imgui::Ui;
use overlay::UnicodeTextRenderer;
use utils_state::StateRegistry;

use crate::{
    app::types::UpdateContext,
    settings::AppSettings,
};

use super::Enhancement;

pub struct PlayerSpyer {}

impl Default for PlayerSpyer {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerSpyer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Enhancement for PlayerSpyer {
    fn update_settings(&mut self, _ui: &Ui, _settings: &mut AppSettings) -> Result<bool> {
        Ok(false)
    }

    fn update(&mut self, _context: &UpdateContext) -> Result<()> {
        Ok(())
    }

    fn render(&self, _states: &StateRegistry, _ui: &Ui, _unicode_text: &UnicodeTextRenderer) -> Result<()> {
        Ok(())
    }

    fn render_debug_window(&mut self, _states: &StateRegistry, _ui: &Ui, _unicode_text: &UnicodeTextRenderer) -> Result<()> {
        Ok(())
    }
}
