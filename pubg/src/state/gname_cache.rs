use std::collections::HashMap;

use anyhow::Context;
use raw_struct::FromMemoryView;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::{
        CStringUtil,
        PtrCStr,
    },
    state::StateDecrypt,
    Module,
    StatePubgHandle,
    StatePubgMemory,
};

pub const G_NAMES_OFFSET: u64 = 0x10466B58;

pub struct StateGNameCache {
    cache: HashMap<u32, String>,
}

impl State for StateGNameCache {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        Ok(Self {
            cache: HashMap::new(),
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

impl StateGNameCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    #[inline]
    pub fn get_gname_by_id(&self, states: &StateRegistry, id: u32) -> anyhow::Result<String> {
        let decrypt = states.resolve::<StateDecrypt>(())?;
        let decrypted_id = StateDecrypt::decrypt_c_index(id);

        // Fast path - try cache lookup first
        {
            let gname_cache = states.resolve::<StateGNameCache>(())?;
            if let Some(name) = gname_cache.get(decrypted_id) {
                return Ok(name);
            }
        }

        // Slow path - cache miss, need to resolve name
        let mut gname_cache = states.resolve_mut::<StateGNameCache>(())?;
        unsafe {
            let pubg_handle = states.resolve::<StatePubgHandle>(())?;
            let memory = states.resolve::<StatePubgMemory>(())?;
            let g_names_address = decrypt.decrypt(
                u64::read_object(
                    memory.view(),
                    decrypt.decrypt(
                        u64::read_object(
                            memory.view(),
                            pubg_handle.memory_address(Module::Game, G_NAMES_OFFSET)?,
                        )
                        .map_err(|err| anyhow::anyhow!("{}", err))?,
                    ) + 0x8,
                )
                .map_err(|err| anyhow::anyhow!("{}", err))?,
            );

            let f_name_ptr = u64::read_object(
                memory.view(),
                g_names_address + ((decrypted_id as u64) / 0x3FD0) * 8,
            )
            .map_err(|err| anyhow::anyhow!("{}", err))? as u64;
            let f_name = PtrCStr::read_object(
                memory.view(),
                f_name_ptr + ((decrypted_id as u64) % 0x3FD0) * 8,
            )
            .map_err(|err| anyhow::anyhow!("{}", err))?;

            let name = f_name
                .read_string(memory.view(), 0x10)?
                .context("f_name nullptr")?;

            gname_cache.insert(decrypted_id, name.clone());
            Ok(name)
        }
    }

    pub fn get(&self, id: u32) -> Option<String> {
        self.cache.get(&id).cloned()
    }

    pub fn insert(&mut self, id: u32, name: String) {
        self.cache.insert(id, name);
    }
}
