//! Causes survive the same Native result envelope as application violations.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CustomInvariantFailureCause {
    DeclaredValue(crate::validation::data::custom_rule::refusal::CustomInvariantRefusal),
    Access(crate::validation::data::custom_rule::access_denial::CustomInvariantAccessDenial),
    Stopped,
    Content(crate::execution::denial_cause::RelationalExecutionDenialCause),
    Interrupted(crate::validation::custom_rule::interruption::CustomInvariantInterruption),
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
