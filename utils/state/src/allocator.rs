use std::{
    any::TypeId,
    collections::HashMap,
    hash::{
        DefaultHasher,
        Hash,
        Hasher,
    },
};

pub(crate) struct StateAllocator {
    index_lookup: HashMap<(TypeId, u64), usize>,
    free_list: Vec<usize>,
}

impl StateAllocator {
    pub fn new(capacity: usize) -> Self {
        let mut free_list = Vec::with_capacity(capacity);
        for index in (0..capacity).rev() {
            free_list.push(index);
        }

        Self {
            index_lookup: Default::default(),
            free_list,
        }
    }

    pub fn calculate_state_index<T: 'static, P: Hash>(
        &mut self,
        params: &P,
        create_if_not_exists: bool,
    ) -> Option<((TypeId, u64), usize)> {
        let mut hasher = DefaultHasher::new();
        params.hash(&mut hasher);
        let params_hash = hasher.finish();

        let cache_key = (TypeId::of::<T>(), params_hash);
        let index = match self.index_lookup.entry(cache_key) {
            std::collections::hash_map::Entry::Occupied(entry) => *entry.get(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                if !create_if_not_exists {
                    /* Do not create the target entry */
                    return None;
                }

                let index = self.free_list.pop()?;
                *entry.insert(index)
            }
        };

        Some((cache_key, index))
    }

    pub fn free_entry(&mut self, cache_key: &(TypeId, u64)) {
        let index = match self.index_lookup.remove(cache_key) {
            Some(index) => index,
            None => return,
        };
        self.free_list.push(index);
    }
}
