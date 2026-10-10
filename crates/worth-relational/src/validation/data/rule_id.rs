use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CustomInvariantRuleId(Arc<str>);

impl CustomInvariantRuleId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
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
