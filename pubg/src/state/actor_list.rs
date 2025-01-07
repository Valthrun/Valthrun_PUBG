use std::{
    collections::{
        HashMap,
        HashSet,
    },
    ops::Deref,
};

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

const CACHED_ACTOR_IDS: [u32; 2] = [717966208, 751521152];

#[derive(Default)]
pub struct ActorCache {
    pub actors: HashMap<u32, Vec<(u64, Ptr64<dyn AActor>)>>,
    pub added_this_frame: Vec<(u32, Ptr64<dyn AActor>)>,
    pub removed_this_frame: Vec<(u32, Ptr64<dyn AActor>)>,
    tracked_addresses: HashSet<u64>,
}

pub struct StateActorLists {
    array: Copy<dyn TArray<Ptr64<dyn AActor>>>,
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
            .value_copy(memory.view(), &decrypt)?
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
        self.cache.added_this_frame.clear();
        self.cache.removed_this_frame.clear();

        // Scan through all actors and collect the ones we want to cache
        /*for actor_ptr in self
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
                let actor_id = actor.id()?;

                // Check if this exact actor was in the previous frame
                let was_present = self
                    .cache
                    .actors
                    .get(&actor_id)
                    .map(|actors| actors.iter().any(|(addr, _)| *addr == actor_addr))
                    .unwrap_or(false);

                if !was_present {
                    self.cache
                        .added_this_frame
                        .push((actor_id, actor_ptr.clone()));
                }

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

        // Find removed actors
        for (actor_id, prev_actors) in &self.cache.actors {
            let current_actors_for_id = current_actors.get(actor_id);

            for (prev_addr, prev_actor) in prev_actors {
                let still_exists = current_actors_for_id
                    .map(|actors| actors.iter().any(|(addr, _)| addr == prev_addr))
                    .unwrap_or(false);

                if !still_exists {
                    self.cache
                        .removed_this_frame
                        .push((*actor_id, prev_actor.clone()));
                }
            }
        }*/

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

    pub fn added_this_frame(&self) -> &[(u32, Ptr64<dyn AActor>)] {
        &self.cache.added_this_frame
    }

    pub fn removed_this_frame(&self) -> &[(u32, Ptr64<dyn AActor>)] {
        &self.cache.removed_this_frame
    }
}
