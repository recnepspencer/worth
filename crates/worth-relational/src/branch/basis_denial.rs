use crate::history::data::BranchId;
use serde::{Deserialize, Serialize};

/// Exact descriptor/root axis that failed owner readmission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalBranchBasisMismatchAxis {
    Branch,
    Target,
    TruthVersion,
    RootIdentity,
    TruthRoot,
    SchemaRoot,
    Visibility,
    Commit,
}

/// Typed failure to resolve or admit a descriptive branch basis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalBranchBasisDenial {
    MalformedDescriptor,
    UnsupportedDescriptorVersion {
        supported: u16,
        actual: u16,
    },
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    UnknownBranch(BranchId),
    ArchivedBranch(BranchId),
    DeletingBranch(BranchId),
    Cancelled,
    TimedOut,
    StaleReferenceGeneration,
    WrongBranchLocalTruthVersion,
    EmptyCommittedTargetMismatch,
    WrongImmutableTarget,
    MixedAxis(RelationalBranchBasisMismatchAxis),
    UnavailableRetainedTarget,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    OwnerUnavailable,
    OwnerFailure,
}
