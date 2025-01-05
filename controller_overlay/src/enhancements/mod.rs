use anyhow::Result;
use imgui::Ui;
use overlay::UnicodeTextRenderer;
use utils_state::StateRegistry;

use crate::{
    app::types::UpdateContext,
    settings::AppSettings,
};

mod player;
pub use player::*;

pub trait Enhancement {
    fn update_settings(&mut self, ui: &Ui, settings: &mut AppSettings) -> Result<bool>;
    fn update(&mut self, context: &UpdateContext) -> Result<()>;
    fn render(&self, states: &StateRegistry, ui: &Ui, unicode_text: &UnicodeTextRenderer) -> Result<()>;
    fn render_debug_window(&mut self, states: &StateRegistry, ui: &Ui, unicode_text: &UnicodeTextRenderer) -> Result<()>;
}
