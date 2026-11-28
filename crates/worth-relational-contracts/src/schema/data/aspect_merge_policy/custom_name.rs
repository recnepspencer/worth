//! A custom merge policy name retains its own shared backing admission.
use crate::admitted_shared::AdmittedShared;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
/// Shared text with borrowed access. Ordinary `From` construction moves its
/// input and allocates an unbudgeted private control cell; it grants no authority.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MergePolicyName(AdmittedShared<str>);

impl std::fmt::Debug for MergePolicyName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}
impl MergePolicyName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<Arc<str>> for MergePolicyName {
    fn from(text: Arc<str>) -> Self {
        Self(AdmittedShared::new(text))
    }
}
impl From<&str> for MergePolicyName {
    fn from(text: &str) -> Self {
        Self::from(Arc::<str>::from(text))
    }
}
impl From<String> for MergePolicyName {
    fn from(text: String) -> Self {
        Self::from(Arc::<str>::from(text))
    }
}
impl std::ops::Deref for MergePolicyName {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
impl AsRef<str> for MergePolicyName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for MergePolicyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&**self, f)
    }
}

/// Operations used by the Relational owners.
pub trait MergePolicyNameOwnerPort: Sized {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>>;
    fn control_bytes() -> Option<u64>;
}

impl MergePolicyNameOwnerPort for MergePolicyName {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.retain_decoded_memory(memory)
    }

    fn control_bytes() -> Option<u64> {
        AdmittedShared::<str>::control_bytes()
    }
}
