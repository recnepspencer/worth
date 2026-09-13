use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::data::error::SignalError;
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::{SignalBranchRetentionAcquisitionDenial, SignalOwnerUnavailable};

#[derive(Debug)]
pub enum SignalBranchRestoreDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OperationCapacityExhausted {
        maximum_in_flight_operations: usize,
    },
    OwnerReentry,
    CancelledNoMovement,
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
    BasisMismatch {
        axes: Vec<FoundationalBranchReferenceMismatchAxis>,
    },
    CrossBranchSnapshot {
        branch_id: SignalBranchId,
        snapshot_branch_id: SignalBranchId,
    },
    UnavailableSnapshot {
        branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    },
    RetentionUnavailable {
        denial: SignalBranchRetentionAcquisitionDenial,
    },
    ForeignSnapshotOwner {
        expected_runtime_instance_id: u64,
        observed_runtime_instance_id: u64,
    },
    OwnerDeniedNoMovement {
        error: SignalError,
    },
}
