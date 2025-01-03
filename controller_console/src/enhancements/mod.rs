pub trait Enhancement {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()>;
}

mod player;
pub use player::*;

use crate::UpdateContext;
