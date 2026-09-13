use serde::{Deserialize, Serialize};
use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::data::error::SignalError;
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::{SignalBranchRetentionAcquisitionDenial, SignalOwnerUnavailable};

#[derive(Debug)]
pub enum SignalBranchBasisObservationDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OperationCapacityExhausted {
        maximum_in_flight_operations: usize,
    },
    OwnerReentry,
    ManagedReferenceDenied {
        denial: super::ManagedSignalBranchReferenceAdmissionDenial,
    },
    UnknownBranch {
        branch_id: SignalBranchId,
    },
    RetirementInProgress {
        branch_id: SignalBranchId,
    },
    RetiredBranch {
        branch_id: SignalBranchId,
    },
    QuarantinedBranch {
        branch_id: SignalBranchId,
    },
    OwnerCellMisuse {
        branch_id: SignalBranchId,
    },
    OwnerInvariantViolation {
        branch_id: SignalBranchId,
    },
    InvalidOwnerObservation {
        error: SignalError,
    },
    RetentionUnavailable {
        denial: SignalBranchRetentionAcquisitionDenial,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchBasisReadmissionDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OperationCapacityExhausted {
        maximum_in_flight_operations: usize,
    },
    OwnerReentry,
    ManagedReferenceDenied {
        denial: super::ManagedSignalBranchReferenceAdmissionDenial,
    },
    UnsupportedDescriptorVersion {
        observed: u16,
        supported: u16,
    },
    LifecycleMismatch,
    OwnerMismatch {
        descriptor_graph_instance_id: String,
        runtime_graph_instance_id: String,
    },
    DefinitionMismatch {
        descriptor_definition_basis: u64,
        runtime_definition_basis: u64,
    },
    UnknownBranch {
        branch_id: SignalBranchId,
    },
    RetirementInProgress {
        branch_id: SignalBranchId,
    },
    RetiredBranch {
        branch_id: SignalBranchId,
    },
    QuarantinedBranch {
        branch_id: SignalBranchId,
    },
    OwnerCellMisuse {
        branch_id: SignalBranchId,
    },
    OwnerInvariantViolation {
        branch_id: SignalBranchId,
    },
    UnavailableSnapshot {
        branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    },
    UnavailableRetention {
        maximum_active_leases: usize,
    },
    RetentionIdentityExhausted,
    ReferenceMismatch {
        axes: Vec<FoundationalBranchReferenceMismatchAxis>,
    },
}

/// Why one readmission through a live external retention obligation failed.
///
/// This is the vocabulary of exact readmission. It has no currentness axis on
/// purpose: the obligation names an exact immutable target, and readmitting it
/// is legitimate long after the branch has moved on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchRetainedReadmissionDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OperationCapacityExhausted {
        maximum_in_flight_operations: usize,
    },
    OwnerReentry,
    /// The obligation was issued by a different live Signal owner.
    ForeignRetention,
    /// The obligation no longer retains anything, or its owner is gone.
    UnavailableRetainedTarget,
    /// The descriptor is not the one this obligation retains.
    DescriptorMismatch,
    UnsupportedDescriptorVersion {
        observed: u16,
        supported: u16,
    },
    LifecycleMismatch,
    /// The retained target no longer satisfies exact admission.
    UnavailableExactTarget(SignalBranchRetentionAcquisitionDenial),
    UnavailableRetention {
        maximum_active_leases: usize,
    },
    RetentionIdentityExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchBasisCompatibilityDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OwnerMismatch,
    DefinitionMismatch,
    SnapshotMismatch,
    RestoreMismatch,
}
