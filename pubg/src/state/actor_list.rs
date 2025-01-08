use std::collections::{
    HashMap,
    HashSet,
};

use anyhow::Context;
use raw_struct::{
    builtins::Ptr64,
    Reference,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::{
        AActor,
        TArray,
    },
    state::{
        StateDecrypt,
        StateWorld,
    },
    StatePubgMemory,
};

const CACHED_ACTOR_IDS: [u32; 2] = [147548544, 181103488];

#[derive(Default)]
pub struct ActorCache {
    pub actors: HashMap<u32, Vec<(u64, Ptr64<dyn AActor>)>>,
    tracked_addresses: HashSet<u64>,
}

pub struct StateActorLists {
    array: Reference<dyn TArray<Ptr64<dyn AActor>>>,
    cache: ActorCache,
}

impl State for StateActorLists {
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
            .value_reference(memory.view_arc(), &decrypt)
            .context("nullptr")?;

        Ok(Self {
            array: actor_array,
            cache: ActorCache::default(),
        })
    }

    fn update(&mut self, states: &StateRegistry) -> anyhow::Result<()> {
        let memory = states.resolve::<StatePubgMemory>(())?;
        let decrypt = states.resolve::<StateDecrypt>(())?;
        self.update_cache(&memory, &decrypt)?;
        Ok(())
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

impl StateActorLists {
    pub fn update_cache(
        &mut self,
        memory: &StatePubgMemory,
        _decrypt: &StateDecrypt,
    ) -> anyhow::Result<()> {
        let mut current_actors: HashMap<u32, Vec<(u64, Ptr64<dyn AActor>)>> = CACHED_ACTOR_IDS
            .iter()
            .map(|&id| (id, Vec::new()))
            .collect();
        let mut current_addresses = HashSet::new();

        // Scan through all actors and collect the ones we want to cache
        for actor_ptr in self
            .array
            .data()?
            .elements(memory.view(), 0..self.array.count()? as usize)?
        {
            if actor_ptr.is_null() {
                continue;
            }

            let actor = actor_ptr
                .value_reference(memory.view_arc())
                .context("actor nullptr")?;

            let actor_addr = actor.reference_address();

            // Skip if we've never seen this actor with our IDs before
            if !self.cache.tracked_addresses.contains(&actor_addr) {
                let actor_id = match actor.id() {
                    Ok(id) => id,
                    Err(_) => continue,
                };

                current_actors
                    .entry(actor_id)
                    .or_default()
                    .push((actor_addr, actor_ptr.clone()));

                current_addresses.insert(actor_addr);
            } else {
                // We know this actor's ID since we've seen it before
                for (id, actors) in &self.cache.actors {
                    if actors.iter().any(|(addr, _)| *addr == actor_addr) {
                        current_actors
                            .entry(*id)
                            .or_default()
                            .push((actor_addr, actor_ptr.clone()));
                        current_addresses.insert(actor_addr);
                        break;
                    }
                }
            }
        }

        self.cache.actors = current_actors;
        self.cache.tracked_addresses = current_addresses;
        Ok(())
    }

    pub fn get_cached_actors(&self, actor_id: u32) -> Option<&Vec<(u64, Ptr64<dyn AActor>)>> {
        self.cache.actors.get(&actor_id)
    }

    pub fn cached_actors(&self) -> &HashMap<u32, Vec<(u64, Ptr64<dyn AActor>)>> {
        &self.cache.actors
    }

    pub fn clear(&mut self) {
        self.cache.actors.clear();
        self.cache.tracked_addresses.clear();
    }
}
