use core::{
    marker::{
        self,
        PhantomData,
    },
    mem,
    ops::Deref,
};
use std::sync::Arc;

use raw_struct::{
    builtins::SizedArray,
    AccessError,
    AccessMode,
    Copy,
    FromMemoryView,
    MemoryView,
    Reference,
    Viewable,
};

use crate::{
    decrypt,
    schema::EncryptedArray,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncryptedPtr64<T>
where
    T: 'static + ?Sized,
{
    pub address: u64,
    _dummy: PhantomData<T>,
}

impl<T: ?Sized + 'static> Clone for EncryptedPtr64<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized + 'static> marker::Copy for EncryptedPtr64<T> {}

impl<T: ?Sized> EncryptedPtr64<T> {
    pub fn is_null(&self) -> bool {
        self.address == 0
    }

    pub fn cast<V: ?Sized>(&self) -> EncryptedPtr64<V> {
        EncryptedPtr64::<V> {
            address: self.address,
            _dummy: Default::default(),
        }
    }
}

impl<T: marker::Copy> EncryptedPtr64<T> {
    /// Create a copy of the value the pointer points to
    #[must_use = "copied result must be used"]
    pub fn read_value(
        &self,
        memory: &dyn MemoryView,
        decrypt: &decrypt::StateDecrypt,
    ) -> Result<Option<T>, AccessError> {
        let address = unsafe { decrypt.decrypt(self.address) };
        if address > 0 {
            let memory = T::read_object(memory, address).map_err(|err| AccessError {
                source: err,

                member: None,
                object: "T".into(),
                mode: AccessMode::Read,

                offset: self.address,
                size: mem::size_of::<T>(),
            })?;

            Ok(Some(memory))
        } else {
            Ok(None)
        }
    }
}

impl<T: ?Sized + Viewable<T>> EncryptedPtr64<T> {
    #[must_use]
    pub fn value_reference(
        &self,
        memory: Arc<dyn MemoryView>,
        decrypt: &decrypt::StateDecrypt,
    ) -> Option<Reference<T>> {
        let address = unsafe { decrypt.decrypt(self.address) };
        if address > 0 {
            Some(Reference::new(memory, address))
        } else {
            None
        }
    }

    /// Create a copy of the value the pointer points to
    #[must_use = "copied result must be used"]
    pub fn value_copy(
        &self,
        memory: &dyn MemoryView,
        decrypt: &decrypt::StateDecrypt,
    ) -> Result<Option<Copy<T>>, AccessError> {
        let address = unsafe { decrypt.decrypt(self.address) };
        if address > 0 {
            let memory = T::Memory::read_object(memory, address).map_err(|err| AccessError {
                source: err,

                member: None,
                object: T::name(),
                mode: AccessMode::Read,

                offset: self.address,
                size: mem::size_of::<T::Memory>(),
            })?;

            Ok(Some(Copy::new(memory)))
        } else {
            Ok(None)
        }
    }
}

impl<T> EncryptedArray<T> for EncryptedPtr64<[T]> {
    fn start_address(&self, decrypt: &decrypt::StateDecrypt) -> u64 {
        unsafe { decrypt.decrypt(self.address) }
    }

    fn len(&self) -> Option<usize> {
        None
    }
}

impl<T> Deref for EncryptedPtr64<[T]> {
    type Target = dyn EncryptedArray<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T, const N: usize> EncryptedArray<T> for EncryptedPtr64<[T; N]> {
    fn start_address(&self, decrypt: &decrypt::StateDecrypt) -> u64 {
        unsafe { decrypt.decrypt(self.address) }
    }

    fn len(&self) -> Option<usize> {
        Some(N)
    }
}

impl<T, const N: usize> Deref for EncryptedPtr64<[T; N]> {
    type Target = dyn EncryptedArray<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T: ?Sized> EncryptedArray<T> for EncryptedPtr64<dyn EncryptedArray<T>> {
    fn start_address(&self, decrypt: &decrypt::StateDecrypt) -> u64 {
        unsafe { decrypt.decrypt(self.address) }
    }

    fn len(&self) -> Option<usize> {
        None
    }
}

impl<T: ?Sized> Deref for EncryptedPtr64<dyn EncryptedArray<T>> {
    type Target = dyn EncryptedArray<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T: ?Sized, const N: usize> EncryptedArray<T> for EncryptedPtr64<dyn SizedArray<T, N>> {
    fn start_address(&self, decrypt: &decrypt::StateDecrypt) -> u64 {
        unsafe { decrypt.decrypt(self.address) }
    }

    fn len(&self) -> Option<usize> {
        Some(N)
    }
}

impl<T: ?Sized, const N: usize> Deref for EncryptedPtr64<dyn SizedArray<T, N>> {
    type Target = dyn EncryptedArray<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}
