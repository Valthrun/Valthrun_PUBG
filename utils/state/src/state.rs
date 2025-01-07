use std::{
    any::Any,
    hash::Hash,
    time::Duration,
};

use anyhow::Result;

use crate::StateRegistry;

pub enum StateCacheType {
    /// The state will be cached and never removed
    Persistent,

    /// The cache entry will be invalidated if not accessed within
    /// the target durection.
    Timed(Duration),

    /// The state will be removed as soon it get's invalidated.
    /// The update method will only be called once uppon creation.
    Volatile,
}

pub trait State: Any + Sized + Send {
    type Parameter: Hash + PartialEq;

    /// Create a new instance of this state.
    /// Note: update will be called after creation automatically.
    fn create(_states: &StateRegistry, _param: Self::Parameter) -> Result<Self> {
        anyhow::bail!("state must be manually set")
    }

    /// Return how the state should be cached
    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }

    /// Update the state
    fn update(&mut self, _states: &StateRegistry) -> Result<()> {
        Ok(())
    }
}
