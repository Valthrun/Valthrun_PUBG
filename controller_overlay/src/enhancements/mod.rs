use overlay::UnicodeTextRenderer;
use utils_state::StateRegistry;

use crate::settings::AppSettings;

pub trait Enhancement {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()>;
    fn update_settings(
        &mut self,
        _ui: &imgui::Ui,
        _settings: &mut AppSettings,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn render(
        &self,
        states: &StateRegistry,
        ui: &imgui::Ui,
        unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()>;
    fn render_debug_window(
        &mut self,
        _states: &StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

mod player;
pub use player::*;

use crate::UpdateContext;
