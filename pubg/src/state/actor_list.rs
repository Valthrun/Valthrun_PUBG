use std::collections::HashMap;

use anyhow::Context;
use raw_struct::builtins::Ptr64;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::AActor,
    state::{
        StateDecrypt,
        StateGNameCache,
        StateWorld,
    },
    StatePubgHandle,
    StatePubgMemory,
};

pub struct StateActorLists {
    pub actors: HashMap<u32, Vec<(u64, Ptr64<dyn AActor>)>>,
}

impl State for StateActorLists {
    type Parameter = ();

    fn create(states: &StateRegistry, _: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StatePubgMemory>(())?;
        let decrypt = states.resolve::<StateDecrypt>(())?;
        let u_world = states.resolve::<StateWorld>(())?;
        let mut gname_cache = states.resolve_mut::<StateGNameCache>(())?;
        let pubg_handle = states.resolve::<StatePubgHandle>(())?;

        let u_level = u_world
            .u_level()
            .context("u_level nullptr")?
            .value_reference(memory.view_arc(), &decrypt)
            .context("nullptr")?;

        let actor_array = u_level
            .actors()
            .context("actor_array nullptr")?
            .value_reference(memory.view_arc(), &decrypt)
            .context("nullptr")?;

        let mut current_actors: HashMap<u32, Vec<(u64, Ptr64<dyn AActor>)>> = HashMap::new();

        // Scan through all actors and collect the ones we want to cache
        for actor_ptr in actor_array
            .data()?
            .elements(memory.view(), 0..actor_array.count()? as usize)?
        {
            if actor_ptr.is_null() {
                continue;
            }

            let actor = actor_ptr
                .value_reference(memory.view_arc())
                .context("actor nullptr")?;

            let actor_id = actor.id()?;
            let name = gname_cache.get_gname_by_id(&decrypt, &pubg_handle, &memory, actor_id)?;

            if name != "PlayerFemale_A_C" && name != "PlayerMale_A_C" {
                continue;
            }

            let actor_addr = actor.reference_address();

            current_actors
                .entry(actor_id)
                .or_default()
                .push((actor_addr, actor_ptr.clone()));
        }

        Ok(Self {
            actors: current_actors,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
