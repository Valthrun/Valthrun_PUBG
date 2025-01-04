use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    sync::Arc,
};

pub struct StateRegistry {
    states: HashMap<(TypeId, ()), RefCell<Box<dyn Any>>>,
    capacity: usize,
}

impl StateRegistry {
    pub fn new(capacity: usize) -> Self {
        Self {
            states: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn set<T: 'static>(&mut self, value: T, key: ()) -> anyhow::Result<()> {
        self.states.insert((TypeId::of::<T>(), key), RefCell::new(Box::new(value)));
        Ok(())
    }

    pub fn get<T: 'static>(&self, key: ()) -> Option<Ref<'_, T>> {
        self.states
            .get(&(TypeId::of::<T>(), key))
            .map(|value| Ref::map(value.borrow(), |v| v.downcast_ref::<T>().unwrap()))
    }

    pub fn get_mut<T: 'static>(&self, key: ()) -> Option<RefMut<'_, T>> {
        self.states
            .get(&(TypeId::of::<T>(), key))
            .map(|value| RefMut::map(value.borrow_mut(), |v| v.downcast_mut::<T>().unwrap()))
    }

    pub fn resolve<T: 'static + Clone>(&self, key: ()) -> anyhow::Result<Arc<T>> {
        self.get::<T>(key)
            .map(|value| Arc::new((*value).clone()))
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve state"))
    }

    pub fn resolve_mut<T: 'static>(&self, key: ()) -> anyhow::Result<RefMut<'_, T>> {
        self.get_mut::<T>(key)
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve state"))
    }

    pub fn invalidate_states(&mut self) {
        self.states.clear();
        self.states = HashMap::with_capacity(self.capacity);
    }
} 