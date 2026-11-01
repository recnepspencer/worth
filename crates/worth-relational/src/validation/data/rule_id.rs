use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
mod custom_rule_name;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvariantRuleId {
    Native(NativeInvariantRuleId),
    Custom(CustomInvariantRuleId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NativeInvariantRuleId {
    LiveRecordRequiresSidecarEntity,
    LiveRecordRequiresSidecarRelation,
    MaxMergedIntents,
    MaxSnapshotEntities,
    UniqueEntityField,
    EndpointKindContract,
    CardinalityMaximumContract,
    CardinalityMinimumContract,
    UniquenessContract,
    SymmetryContract,
    EndpointDeletionIntegrityContract,
    AcyclicityContract,
    PartitionIsolationContract,
    ConnectivityMinimumContract,
}

/// Shared rule text has an intrinsic cold-read admission through every clone.
/// Ordinary construction retains the existing explicitly unbudgeted contract.
pub struct CustomInvariantRuleId(Arc<custom_rule_name::Backing>);

// Public meaning is immutable shared text. The private one-shot grant slot
// retains lifetime bookkeeping and exposes no mutable identifier semantics.
impl std::panic::RefUnwindSafe for CustomInvariantRuleId {}
impl std::panic::UnwindSafe for CustomInvariantRuleId {}

impl CustomInvariantRuleId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(Arc::new(custom_rule_name::Backing::new(id.into())))
    }
    pub fn as_str(&self) -> &str {
        self.0.text()
    }
    pub(crate) fn retain_decoded_memory(
        &self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), Option<worth_execution::ExecutionMemoryReservation>> {
        self.0.retain_decoded_memory(memory)
    }
    pub(crate) fn control_bytes() -> Option<u64> {
        custom_rule_name::Backing::control_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CustomInvariantSemanticVersion {
    pub major: u16,
    pub minor: u16,
}

impl CustomInvariantSemanticVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CustomInvariantSemanticIdentity {
    pub rule_id: CustomInvariantRuleId,
    pub semantic_version: CustomInvariantSemanticVersion,
}
