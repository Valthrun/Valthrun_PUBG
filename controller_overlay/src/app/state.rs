use std::ops::{Deref, DerefMut};
use utils_state::{State, StateCacheType, StateRegistry};
use crate::{
    view::ViewController,
};
use pubg::{StatePubgHandle, StatePubgMemory};

// Local wrapper types for external types
pub struct LocalPubgHandle(StatePubgHandle);
pub struct LocalPubgMemory(StatePubgMemory);
pub struct LocalViewController(ViewController);

impl Deref for LocalPubgHandle {
    type Target = StatePubgHandle;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LocalPubgHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<StatePubgHandle> for LocalPubgHandle {
    fn from(handle: StatePubgHandle) -> Self {
        Self(handle)
    }
}

impl Deref for LocalPubgMemory {
    type Target = StatePubgMemory;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LocalPubgMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<StatePubgMemory> for LocalPubgMemory {
    fn from(memory: StatePubgMemory) -> Self {
        Self(memory)
    }
}

impl Deref for LocalViewController {
    type Target = ViewController;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LocalViewController {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl State for LocalPubgHandle {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        anyhow::bail!("LocalPubgHandle must be manually set")
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

impl State for LocalPubgMemory {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        anyhow::bail!("LocalPubgMemory must be manually set")
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

impl State for LocalViewController {
    type Parameter = ();

    fn create(states: &StateRegistry, param: Self::Parameter) -> anyhow::Result<Self> {
        Ok(Self(ViewController::create(states, param)?))
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }

    fn update(&mut self, states: &StateRegistry) -> anyhow::Result<()> {
        self.0.update(states)
    }
} 