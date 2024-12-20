use std::mem::transmute;

use anyhow::{
    anyhow,
    Context,
};
use obfstr::obfstr;
use raw_struct::{
    AccessError,
    AccessMode,
    FromMemoryView,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};
use windows_sys::Win32::System::Memory::{
    VirtualAlloc,
    MEM_COMMIT,
    PAGE_EXECUTE_READWRITE,
};

use crate::{
    schema::{
        CStringUtil,
        PtrCStr,
    },
    Module,
    StatePubgHandle,
    StatePubgMemory,
};

type XenuineDecrypt = unsafe extern "fastcall" fn(u64, u64) -> u64;

pub const DECRYPT_OFFSET: u64 = 0xE790128;
pub const G_NAMES_OFFSET: u64 = 0x104C7068;

pub struct StateDecrypt {
    decrypt_key: u64,
    xenuine_decrypt_fn: XenuineDecrypt,
}

impl State for StateDecrypt {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        unsafe {
            let pubg_handle = _states.resolve::<StatePubgHandle>(())?;
            let memory = _states.resolve::<StatePubgMemory>(())?;
            let decrypt_ptr = u64::read_object(
                memory.view(),
                pubg_handle.memory_address(Module::Game, DECRYPT_OFFSET)?,
            )
            .map_err(|err| anyhow::anyhow!("{}", err))? as u64;
            let tmp1_add = i32::read_object(memory.view(), decrypt_ptr + 3)
                .map_err(|err| anyhow::anyhow!("{}", err))? as u64;
            let decrypt_key = tmp1_add + decrypt_ptr + 7;

            let mut code_buff: [u8; 1024] = [0; 1024];
            code_buff[0] = 0x90;
            code_buff[1] = 0x90;
            memory
                .read_memory(decrypt_ptr, &mut code_buff[2..])
                .map_err(|err| AccessError {
                    source: err,

                    mode: AccessMode::Read,
                    offset: decrypt_ptr,
                    size: 1022,

                    member: None,
                    object: "unknown".into(),
                })?;
            code_buff[2] = 0x48;
            code_buff[3] = 0x8B;
            code_buff[4] = 0xC1;
            code_buff[5] = 0x90;
            code_buff[6] = 0x90;
            code_buff[7] = 0x90;
            code_buff[8] = 0x90;

            let executable_memory = VirtualAlloc(
                std::ptr::null_mut(),
                code_buff.len() + 4,
                MEM_COMMIT,
                PAGE_EXECUTE_READWRITE,
            );

            if executable_memory.is_null() {
                return Err(anyhow!(obfstr!(
                    "Failed to allocate executable memory for XenuineDecrypt function"
                )
                .to_string()));
            }

            std::ptr::copy_nonoverlapping(
                code_buff.as_ptr(),
                executable_memory as *mut u8,
                code_buff.len(),
            );

            let xenuine_decrypt_fn = transmute(executable_memory);

            Ok(Self {
                decrypt_key,
                xenuine_decrypt_fn,
            })
        }
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

impl StateDecrypt {
    #[inline]
    pub unsafe fn decrypt(&self, a: u64) -> u64 {
        let result = (self.xenuine_decrypt_fn)(self.decrypt_key, a);
        result
    }

    #[inline]
    pub fn decrypt_c_index(value: u32) -> u32 {
        let rotated = value.rotate_left(0x0E) ^ 0x08928651;
        rotated ^ (rotated << 0x10) ^ 0xD11B42D7
    }

    #[inline]
    pub fn get_gname_by_id(&self, states: &StateRegistry, id: u32) -> anyhow::Result<String> {
        unsafe {
            let pubg_handle = states.resolve::<StatePubgHandle>(())?;
            let memory = states.resolve::<StatePubgMemory>(())?;
            let g_names_address = self.decrypt(
                u64::read_object(
                    memory.view(),
                    self.decrypt(pubg_handle.memory_address(Module::Game, G_NAMES_OFFSET)?),
                )
                .map_err(|err| anyhow::anyhow!("{}", err))?,
            );

            let f_name_ptr =
                u64::read_object(memory.view(), g_names_address + ((id as u64) / 0x402c) * 8)
                    .map_err(|err| anyhow::anyhow!("{}", err))? as u64;
            let f_name = u64::read_object(memory.view(), f_name_ptr + ((id as u64) % 0x402c) * 8)
                .map_err(|err| anyhow::anyhow!("{}", err))? as u64;
            PtrCStr::read_object(memory.view(), f_name + 0x10 + 0x08)
                .map_err(|e| anyhow!(e))?
                .read_string(memory.view())?
                .context("gname nullptr")
        }
    }
}
