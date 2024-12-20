use core::{
    marker::{
        self,
    },
    mem,
    ops::Range,
    slice,
};
use std::{
    format,
    sync::Arc,
    vec::Vec,
};

use raw_struct::{
    AccessError,
    AccessMode,
    Copy,
    FromMemoryView,
    MemoryView,
    Reference,
    Viewable,
};

use crate::decrypt;

#[allow(clippy::len_without_is_empty)]
pub trait EncryptedArray<T: ?Sized> {
    fn start_address(&self, decrypt: &decrypt::StateDecrypt) -> u64;

    fn len(&self) -> Option<usize>;
}

impl<T: FromMemoryView> dyn EncryptedArray<T> {
    pub fn element_at(
        &self,
        memory: &dyn MemoryView,
        index: usize,
        decrypt: &decrypt::StateDecrypt,
    ) -> Result<T, AccessError> {
        let offset = (index * mem::size_of::<T>()) as u64;
        T::read_object(memory, self.start_address(decrypt) + offset).map_err(|err| AccessError {
            source: err,
            offset: self.start_address(decrypt) + offset,
            size: mem::size_of::<T>(),
            mode: AccessMode::Read,
            object: "[..]".into(),
            member: Some(format!("[{}]", index).into()),
        })
    }

    pub fn elements(
        &self,
        memory: &dyn MemoryView,
        range: Range<usize>,
        decrypt: &decrypt::StateDecrypt,
    ) -> Result<Vec<T>, AccessError> {
        let element_count = range.end - range.start;
        let mut result = Vec::with_capacity(element_count);

        unsafe {
            let buffer = slice::from_raw_parts_mut(
                result.as_mut_ptr() as *mut u8,
                element_count * mem::size_of::<T>(),
            );
            let offset = self.start_address(decrypt) + (range.start * mem::size_of::<T>()) as u64;

            memory
                .read_memory(offset, buffer)
                .map_err(|err| AccessError {
                    source: err,
                    offset,
                    size: buffer.len(),
                    mode: AccessMode::Read,
                    object: "[..]".into(),
                    member: Some(format!("[{:#?}]", range).into()),
                })?;

            result.set_len(element_count);
        };

        Ok(result)
    }
}

impl<T: ?Sized + Viewable<T>> dyn EncryptedArray<T> {
    pub fn element_reference(
        &self,
        memory: Arc<dyn MemoryView>,
        index: usize,
        decrypt: &decrypt::StateDecrypt,
    ) -> Reference<T> {
        let offset = (index * T::MEMORY_SIZE) as u64;
        Reference::new(memory, self.start_address(decrypt) + offset)
    }

    pub fn elements_reference(
        &self,
        memory: Arc<dyn MemoryView>,
        range: Range<usize>,
        decrypt: &decrypt::StateDecrypt,
    ) -> Vec<Reference<T>> {
        Vec::from_iter(range.map(|index| {
            Reference::new(
                memory.clone(),
                self.start_address(decrypt) + (index * T::MEMORY_SIZE) as u64,
            )
        }))
    }
}

impl<T: ?Sized + Viewable<T>> dyn EncryptedArray<T>
where
    T::Implementation<T::Memory>: marker::Copy,
{
    pub fn element_copy(
        &self,
        memory: &dyn MemoryView,
        index: usize,
        decrypt: &decrypt::StateDecrypt,
    ) -> Result<Copy<T>, AccessError> {
        let offset = (index * T::MEMORY_SIZE) as u64;
        Copy::read_object(memory, self.start_address(decrypt) + offset).map_err(|err| AccessError {
            source: err,
            offset: self.start_address(decrypt) + offset,
            size: T::MEMORY_SIZE,
            mode: AccessMode::Read,
            object: format!("[{}]", T::name()).into(),
            member: Some(format!("[{}]", index).into()),
        })
    }

    pub fn elements_copy(
        &self,
        memory: &dyn MemoryView,
        range: Range<usize>,
        decrypt: &decrypt::StateDecrypt,
    ) -> Result<Vec<Copy<T>>, AccessError> {
        let element_count = range.end - range.start;
        let mut result = Vec::<T::Memory>::with_capacity(element_count);

        unsafe {
            let buffer = slice::from_raw_parts_mut(
                result.as_mut_ptr() as *mut u8,
                element_count * T::MEMORY_SIZE,
            );
            let offset = self.start_address(decrypt) + (range.start * T::MEMORY_SIZE) as u64;

            memory
                .read_memory(offset, buffer)
                .map_err(|err| AccessError {
                    source: err,
                    offset,
                    size: buffer.len(),
                    mode: AccessMode::Read,
                    object: "[..]".into(),
                    member: Some(format!("[{:#?}]", range).into()),
                })?;

            result.set_len(element_count);
        };

        Ok(result.into_iter().map(Copy::<T>::new).collect::<Vec<_>>())
    }
}

pub trait SizedEncryptedArray<T: ?Sized, const N: usize>: EncryptedArray<T> {}

impl<T: ?Sized, const N: usize> dyn SizedEncryptedArray<T, N> {
    pub fn len(&self) -> usize {
        N
    }
}
