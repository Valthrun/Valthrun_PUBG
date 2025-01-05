use std::any::{Any, TypeId, type_name};
use std::cell::{RefCell, Ref, RefMut};
use std::ops::DerefMut;
use std::time::Instant;
use anyhow::{anyhow, Context, Result};

use crate::allocator::StateAllocator;
use crate::ref_utils::{transpose_ref_opt, transpose_ref_mut_opt};
use crate::state::{State, StateCacheType};

struct InternalState {
    value: Box<dyn Any + Send>,
    value_update: fn(&mut Box<dyn Any + Send>, states: &StateRegistry) -> Result<()>,

    cache_key: (TypeId, u64),
    cache_type: StateCacheType,

    dirty: bool,
    last_access: Instant,
}

pub struct StateRegistry {
    allocator: RefCell<StateAllocator>,
    states: Vec<RefCell<Option<InternalState>>>,
}

fn value_update_proxy<T: State>(
    value: &mut Box<dyn Any + Send>,
    states: &StateRegistry,
) -> Result<()> {
    let value = value.downcast_mut::<T>().expect("to be of type T");
    value.update(states)
}

impl StateRegistry {
    pub fn new(capacity: usize) -> Self {
        let mut states = Vec::with_capacity(capacity);
        states.resize_with(capacity, Default::default);
        Self {
            allocator: RefCell::new(StateAllocator::new(capacity)),
            states,
        }
    }

    pub fn invalidate_states(&mut self) {
        /* As we're mutable there should be no more references to the underlying state */
        let mut allocator = self.allocator.borrow_mut();

        let now = Instant::now();
        for state in self.states.iter_mut() {
            let mut state_ref = state.borrow_mut();
            let state = if let Some(state) = state_ref.deref_mut() {
                state
            } else {
                continue;
            };

            if !state.dirty {
                /* State has been accessed. */
                state.last_access = now;
                state.dirty = true;
            }

            let state_expired = match state.cache_type {
                StateCacheType::Persistent => false,
                StateCacheType::Volatile => true,
                StateCacheType::Timed(timeout) => state.last_access.elapsed() > timeout,
            };
            if state_expired {
                allocator.free_entry(&state.cache_key);
                *state_ref = None;
            }
        }
    }

    pub fn set<T: State>(&mut self, value: T, params: T::Parameter) -> Result<()> {
        let (cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T, _>(&params, true)
            .context("state capacity exceeded")?;

        let mut state_ref = self.states[index].borrow_mut();
        *state_ref = Some(InternalState {
            value: Box::new(value),
            value_update: value_update_proxy::<T>,

            cache_key,
            cache_type: T::cache_type(),

            dirty: false,
            last_access: Instant::now(),
        });
        Ok(())
    }

    pub fn get<T: State>(&self, params: T::Parameter) -> Option<Ref<'_, T>> {
        let (_cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T, _>(&params, false)?;

        let value = self.states[index]
            .try_borrow()
            .ok()
            .map(transpose_ref_opt)
            .flatten()?;

        let value = Ref::map(value, |value| {
            value.value.downcast_ref::<T>().expect("to be type T")
        });

        Some(value)
    }

    pub fn get_mut<T: State>(&self, params: T::Parameter) -> Option<RefMut<'_, T>> {
        let (_cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T, _>(&params, false)?;

        let value = self.states[index]
            .try_borrow_mut()
            .ok()
            .map(transpose_ref_mut_opt)
            .flatten()?;

        let value = RefMut::map(value, |value| {
            value.value.downcast_mut::<T>().expect("to be type T")
        });

        Some(value)
    }

    fn initialize_value<T: State>(
        &self,
        cache_key: (TypeId, u64),
        value: &mut RefMut<'_, Option<InternalState>>,
        params: T::Parameter,
    ) -> Result<()> {
        let value = match value.as_mut() {
            Some(value) => value,
            None => {
                /* create a new value */
                let state = Box::new(
                    T::create(self, params)
                        .with_context(|| format!("create {}", type_name::<T>()))?,
                );
                **value = Some(InternalState {
                    value: state,
                    value_update: value_update_proxy::<T>,

                    cache_key,
                    cache_type: T::cache_type(),

                    dirty: true,
                    last_access: Instant::now(),
                });

                value.as_mut().unwrap()
            }
        };

        if value.dirty {
            (value.value_update)(&mut value.value, self)
                .with_context(|| format!("update {}", type_name::<T>()))?;
            value.dirty = false;
        }

        Ok(())
    }

    pub fn resolve_mut<T: State>(&self, params: T::Parameter) -> Result<RefMut<'_, T>> {
        let (cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T, _>(&params, true)
            .context("state capacity exceeded")?;

        let mut value = self.states[index]
            .try_borrow_mut()
            .context("value already borrowed")?;

        self.initialize_value::<T>(cache_key, &mut value, params)?;
        let value = transpose_ref_mut_opt(value).context("expected a valid value")?;

        Ok(RefMut::map(value, |value| {
            value.value.downcast_mut::<T>().expect("to be of type T")
        }))
    }

    pub fn resolve<T: State>(&self, params: T::Parameter) -> Result<Ref<'_, T>> {
        let (cache_key, index) = self
            .allocator
            .borrow_mut()
            .calculate_state_index::<T, _>(&params, true)
            .context("state capacity exceeded")?;

        if let Ok(mut value) = self.states[index].try_borrow_mut() {
            self.initialize_value::<T>(cache_key, &mut value, params)?;
        } else {
            /* We already borrowed that state, hence it must be initialized & not dirty */
        }

        let value = self.states[index].try_borrow().map_err(|_| {
            anyhow!(
                "circular state initialisation for {}",
                type_name::<T>()
            )
        })?;

        let value = Ref::map(value, |value| {
            let value = value.as_ref().expect("to be present");
            value.value.downcast_ref::<T>().expect("to be of type T")
        });
        Ok(value)
    }
} 