//! Causes survive the same Native result envelope as application violations.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CustomInvariantFailureCause {
    DeclaredValue(super::CustomInvariantRefusal),
    Access(super::CustomInvariantAccessDenial),
    Stopped,
    Content(crate::execution::RelationalExecutionDenialCause),
    Interrupted(super::CustomInvariantInterruption),
    ManagedIndex { detail: String },
    Allocation,
    CheckedHookRequired,
    MissingRequiredField { field: String },
}
impl CustomInvariantFailureCause {
    pub fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::DeclaredValue(value) => value.owned_allocation_capacity_bytes(),
            Self::Access(value) => value.owned_allocation_capacity_bytes(),
            Self::ManagedIndex { detail: field } | Self::MissingRequiredField { field } => {
                field.capacity() as u64
            }
            _ => 0,
        }
    }
}
