use std::ops::Deref;

use anyhow::Context;
use raw_struct::{
    builtins::Ptr64,
    Copy,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    decrypt::StateDecrypt,
    schema::{
        AActor,
        TArray,
    },
    state::StateWorld,
    StatePubgMemory,
};

pub struct StateActorList(Copy<dyn TArray<Ptr64<dyn AActor>>>);
impl State for StateActorList {
    type Parameter = ();

    fn create(states: &StateRegistry, _: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StatePubgMemory>(())?;
        let decrypt = states.resolve::<StateDecrypt>(())?;
        let u_world = states.resolve::<StateWorld>(())?;

        let u_level = u_world
            .u_level()
            .context("u_level nullptr")?
            .value_reference(memory.view_arc(), &decrypt)
            .context("nullptr")?;

        let actor_array = u_level
            .actors()
            .context("actor_array nullptr")?
            .value_copy(memory.view(), &decrypt)?
            .context("nullptr")?;

        Ok(Self(actor_array))
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

impl Deref for StateActorList {
    type Target = Copy<dyn TArray<Ptr64<dyn AActor>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
