use std::ops::Deref;

use anyhow::Context;
use raw_struct::Reference;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    decrypt::StateDecrypt,
    schema::{
        Entry,
        UWorld,
        ENTRY_OFFSET,
    },
    Module,
    StatePubgHandle,
    StatePubgMemory,
};

pub struct StateWorld(Reference<dyn UWorld>);
impl State for StateWorld {
    type Parameter = ();

    fn create(states: &StateRegistry, _: Self::Parameter) -> anyhow::Result<Self> {
        let handle = states.resolve::<StatePubgHandle>(())?;
        let memory = states.resolve::<StatePubgMemory>(())?;
        let decrypt = states.resolve::<StateDecrypt>(())?;

        let base_address = handle.memory_address(Module::Game, 0x0)?;
        let entry = Reference::<dyn Entry>::new(memory.clone(), base_address + ENTRY_OFFSET);
        let u_world = entry
            .u_world()?
            .value_reference(memory.view_arc(), &decrypt)
            .context("UWorld nullptr")?;

        Ok(Self(u_world))
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

impl Deref for StateWorld {
    type Target = Reference<dyn UWorld>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
