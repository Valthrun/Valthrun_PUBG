use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
};

use pubg::PubgHandle;
use utils_state::StateRegistry;

use crate::enhancements::Enhancement;

use super::{fonts::AppFonts, settings_manager::SettingsManager};

pub struct UpdateContext<'a> {
    pub states: &'a StateRegistry,
}

pub struct Application {
    pub fonts: AppFonts,
    pub states: StateRegistry,
    pub pubg: Arc<PubgHandle>,
    pub enhancements: Vec<Rc<RefCell<dyn Enhancement>>>,
    pub settings_manager: SettingsManager,
    pub frame_read_calls: usize,
} 