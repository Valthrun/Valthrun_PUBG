use std::{
    cell::Ref,
    hash::Hash,
};

use raw_struct::{
    FromMemoryView,
    Reference,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::{
        ACharacter,
        USceneComponent,
    },
    StatePubgMemory,
};

pub struct StatePlayerInfoParams {
    pub character: Reference<dyn ACharacter>,
    pub root_component: Reference<dyn USceneComponent>,
}

impl PartialEq for StatePlayerInfoParams {
    fn eq(&self, other: &Self) -> bool {
        self.character.reference_address() == other.character.reference_address()
            && self.root_component.reference_address() == other.root_component.reference_address()
    }
}

impl Hash for StatePlayerInfoParams {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.character.reference_address().hash(state);
        self.root_component.reference_address().hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct StatePlayerInfo {
    pub position: [f32; 3],
    pub health: u32,
}

impl StatePlayerInfo {
    pub fn new() -> Self {
        Self {
            position: [0.0; 3],
            health: 0,
        }
    }

    /// XOR keys used for player health decryption
    const HEALTH_XOR_KEYS: [u32; 16] = [
        0xcec7a59c, 0x9b63b286, 0xca6be2bd, 0x863848f3, 0x74911d0a, 0x23ddaea4, 0x945f3c8,
        0xbd39b521, 0xbae7a58, 0xa8ef8687, 0xe27593b5, 0x9f8adb68, 0xbd7443d5, 0x8693f307,
        0xf3099e38, 0xe1d42a5c,
    ];

    const HEALTH4: u32 = 0xA30;

    pub fn decrypt_player_health(value: &mut [u8], offset: u32) {
        let xor_keys = unsafe {
            std::slice::from_raw_parts((&Self::HEALTH_XOR_KEYS as *const u32) as *const u8, 64)
        };
        let size = value.len() as u32;
        for i in 0..size as usize {
            value[i] ^= xor_keys[(i as u32 + offset) as usize & 0x3F];
        }
    }

    pub fn get_health(
        actor: Reference<dyn ACharacter>,
        memory: Ref<'_, StatePubgMemory>,
    ) -> anyhow::Result<f32> {
        let b_health_flag = actor.health_flag()? != 3;
        let b_health1 = actor.health1()? != 0;

        if b_health_flag && b_health1 {
            let b_is_encrypted = actor.health5()? != 0;
            let health3 = actor.health3()?;

            let mut health4 = f32::read_object(
                memory.view(),
                health3 as u64 + actor.reference_address() + Self::HEALTH4 as u64,
            )
            .map_err(|err| anyhow::anyhow!("{}", err))?;

            if b_is_encrypted {
                let mut health4_bytes = health4.to_le_bytes();
                Self::decrypt_player_health(&mut health4_bytes, actor.health6()?);
                health4 = f32::from_le_bytes(health4_bytes);
            }

            Ok(health4)
        } else {
            Ok(actor.health2()?)
        }
    }
}

impl State for StatePlayerInfo {
    type Parameter = StatePlayerInfoParams;

    fn create(states: &StateRegistry, params: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StatePubgMemory>(())?;
        let health = Self::get_health(params.character, memory)?;
        let health = health as u32;

        let relative_location = params.root_component.relative_location()?;

        Ok(Self {
            position: relative_location,
            health,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
