//! Opaque shared declared identity keys keep their original allocation and funding together.
use crate::admitted_shared::AdmittedShared;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
type Element = worth_foundational::facade::AspectKey;

/// Shared keys with borrowed access. Ordinary `From` construction moves its
/// input and allocates an unbudgeted private control cell; it grants no authority.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeclaredIdentityKeys(AdmittedShared<[Element]>);

impl std::fmt::Debug for DeclaredIdentityKeys {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}
impl DeclaredIdentityKeys {
    pub fn as_slice(&self) -> &[Element] {
        &self.0
    }
}
impl From<Arc<[Element]>> for DeclaredIdentityKeys {
    fn from(values: Arc<[Element]>) -> Self {
        Self(AdmittedShared::new(values))
    }
}
impl From<Vec<Element>> for DeclaredIdentityKeys {
    fn from(values: Vec<Element>) -> Self {
        Self::from(Arc::<[Element]>::from(values))
    }
}
impl<const N: usize> From<[Element; N]> for DeclaredIdentityKeys {
    fn from(values: [Element; N]) -> Self {
        Self::from(Arc::<[Element]>::from(values))
    }
}
impl std::ops::Deref for DeclaredIdentityKeys {
    type Target = [Element];
    fn deref(&self) -> &[Element] {
        self.as_slice()
    }
}
impl AsRef<[Element]> for DeclaredIdentityKeys {
    fn as_ref(&self) -> &[Element] {
        self.as_slice()
    }
}
impl<'a> IntoIterator for &'a DeclaredIdentityKeys {
    type Item = &'a Element;
    type IntoIter = std::slice::Iter<'a, Element>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

/// Operations used by the Relational owners.
pub trait DeclaredIdentityKeysOwnerPort: Sized {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>>;
    fn control_bytes() -> Option<u64>;
    fn declared_keys_storage_bytes_for_test(count: usize) -> Option<u64>;
}

impl DeclaredIdentityKeysOwnerPort for DeclaredIdentityKeys {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.retain_decoded_memory(memory)
    }

    fn control_bytes() -> Option<u64> {
        AdmittedShared::<[Element]>::control_bytes()
    }

    fn declared_keys_storage_bytes_for_test(count: usize) -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::array::<Element>(count).ok()?)
            .ok()?
            .0
            .pad_to_align();
        Self::control_bytes()?.checked_add(u64::try_from(layout.size()).ok()?)
    }
}
