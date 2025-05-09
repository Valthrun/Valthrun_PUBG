use std::collections::HashMap;
use std::fs::{OpenOptions, File};
use std::io::Write;
use std::path::Path;

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

pub const G_NAMES_OFFSET: u64 = 0x10812CE8;
pub const ELEMENTS_PER_CHUNK: u64 = 0x41A4;
pub const G_NAMES_OFFSET2: u64 = 0x10;

pub struct StateGNameCache {
    cache: HashMap<u32, String>,
    log_file: Option<File>,
}

impl State for StateGNameCache {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("gnames_log.txt")
            .ok();
            
        Ok(Self {
            cache: HashMap::new(),
            log_file,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

impl StateGNameCache {
    pub fn new() -> Self {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("gnames_log.txt")
            .ok();
            
        Self {
            cache: HashMap::new(),
            log_file,
        }
    }

    #[inline]
    pub fn get_gname_by_id(
        &mut self,
        decrypt: &StateDecrypt,
        pubg_handle: &StatePubgHandle,
        memory: &StatePubgMemory,
        id: u32,
    ) -> anyhow::Result<String> {
        let decrypted_id = StateDecrypt::decrypt_c_index(id);

        // Fast path - try cache lookup first
        {
            if let Some(name) = self.get(decrypted_id) {
                return Ok(name);
            }
        }

        // Slow path - cache miss, need to resolve name
        unsafe {
            let g_names_address = decrypt.decrypt(
                u64::read_object(
                    memory.view(),
                    decrypt.decrypt(
                        u64::read_object(
                            memory.view(),
                            pubg_handle.memory_address(Module::Game, G_NAMES_OFFSET)?,
                        )
                        .map_err(|err| anyhow::anyhow!("{}", err))?,
                    ) + G_NAMES_OFFSET2,
                )
                .map_err(|err| anyhow::anyhow!("{}", err))?,
            );
            let f_name_ptr = u64::read_object(
                memory.view(),
                g_names_address + ((decrypted_id as u64) / ELEMENTS_PER_CHUNK) * 8,
            )
            .map_err(|err| anyhow::anyhow!("{}", err))? as u64;
            let f_name = PtrCStr::read_object(
                memory.view(),
                f_name_ptr + ((decrypted_id as u64) % ELEMENTS_PER_CHUNK) * 8,
            )
            .map_err(|err| anyhow::anyhow!("{}", err))?;
            let name = f_name
                .read_string(memory.view(), 0x10)?
                .context("f_name nullptr")?;

            // Write the new GName to the log file
            self.log_gname(decrypted_id, &name);
            
            self.insert(decrypted_id, name.clone());
            Ok(name)
        }
    }

    pub fn get(&self, id: u32) -> Option<String> {
        self.cache.get(&id).cloned()
    }

    pub fn insert(&mut self, id: u32, name: String) {
        self.cache.insert(id, name);
    }
    
    fn log_gname(&mut self, id: u32, name: &str) {
        if let Some(file) = &mut self.log_file {
            let _ = writeln!(file, "ID: {}, Name: {}", id, name);
            let _ = file.flush();
        }
    }
}
