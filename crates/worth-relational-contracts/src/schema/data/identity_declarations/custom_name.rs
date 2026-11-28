//! A custom identity basis name retains its own shared backing admission.
use crate::admitted_shared::AdmittedShared;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
/// Shared text with borrowed access. Ordinary `From` construction moves its
/// input and allocates an unbudgeted private control cell; it grants no authority.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdentityBasisName(AdmittedShared<str>);

impl std::fmt::Debug for IdentityBasisName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}
impl IdentityBasisName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<Arc<str>> for IdentityBasisName {
    fn from(text: Arc<str>) -> Self {
        Self(AdmittedShared::new(text))
    }
}
impl From<&str> for IdentityBasisName {
    fn from(text: &str) -> Self {
        Self::from(Arc::<str>::from(text))
    }
}
impl From<String> for IdentityBasisName {
    fn from(text: String) -> Self {
        Self::from(Arc::<str>::from(text))
    }
}
impl std::ops::Deref for IdentityBasisName {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
impl AsRef<str> for IdentityBasisName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for IdentityBasisName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&**self, f)
    }
}

/// Operations used by the Relational owners.
pub trait IdentityBasisNameOwnerPort: Sized {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>>;
    fn control_bytes() -> Option<u64>;
    fn custom_name_storage_bytes_for_test(length: usize) -> Option<u64>;
}

impl IdentityBasisNameOwnerPort for IdentityBasisName {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.retain_decoded_memory(memory)
    }

    fn control_bytes() -> Option<u64> {
        AdmittedShared::<str>::control_bytes()
    }

    fn custom_name_storage_bytes_for_test(length: usize) -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::array::<u8>(length).ok()?)
            .ok()?
            .0
            .pad_to_align();
        Self::control_bytes()?.checked_add(u64::try_from(layout.size()).ok()?)
    }
}
