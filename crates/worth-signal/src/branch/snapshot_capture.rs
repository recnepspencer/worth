use std::sync::Arc;

use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::data::error::SignalError;
use crate::state::{SignalBranchId, SignalSnapshotId, SignalSnapshotV1};

use super::{
    AdmittedSignalBranchBasis, SignalBranchAdmissionLease, SignalBranchRetentionAcquisitionDenial,
    SignalOwnerUnavailable,
};

/// Owner-bound snapshot authority accepted by canonical restore operations.
///
/// The payload remains inspectable, but only Signal can bind it to a live
/// runtime instance. Portable payloads must pass through owner reconstruction
/// before they can move a branch reference.
#[derive(Debug, Clone)]
pub struct AdmittedSignalBranchSnapshot {
    owner_runtime_instance_id: u64,
    snapshot: SignalSnapshotV1,
    _retention: Arc<SignalBranchAdmissionLease>,
}

impl AdmittedSignalBranchSnapshot {
    pub(crate) fn owner_issued(
        owner_runtime_instance_id: u64,
        snapshot: SignalSnapshotV1,
        retention: SignalBranchAdmissionLease,
    ) -> Self {
        Self {
            owner_runtime_instance_id,
            snapshot,
            _retention: Arc::new(retention),
        }
    }

    pub(crate) const fn owner_runtime_instance_id(&self) -> u64 {
        self.owner_runtime_instance_id
    }

    pub(crate) fn retention_identity(&self) -> u64 {
        self._retention.lease_id()
    }

    pub fn snapshot(&self) -> &SignalSnapshotV1 {
        &self.snapshot
    }

    pub fn into_snapshot(self) -> SignalSnapshotV1 {
        self.snapshot
    }
}

#[derive(Debug)]
pub enum SignalBranchSnapshotCaptureDenial {
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
    RetentionUnavailable {
        denial: SignalBranchRetentionAcquisitionDenial,
    },
    SnapshotCapacityExhausted {
        maximum_stored_snapshots: usize,
    },
    SnapshotIdentityExhausted {
        next_snapshot_id: SignalSnapshotId,
    },
    OwnerDeniedNoMovement {
        error: SignalError,
    },
}

/// Owner-issued result of capturing a snapshot and moving its branch reference.
#[derive(Debug)]
pub struct SignalBranchSnapshotCaptureOutcome {
    snapshot: AdmittedSignalBranchSnapshot,
    captured_basis: AdmittedSignalBranchBasis,
}

impl SignalBranchSnapshotCaptureOutcome {
    pub(crate) fn owner_issued(
        snapshot: AdmittedSignalBranchSnapshot,
        captured_basis: AdmittedSignalBranchBasis,
    ) -> Self {
        Self {
            snapshot,
            captured_basis,
        }
    }

    pub fn snapshot(&self) -> &SignalSnapshotV1 {
        self.snapshot.snapshot()
    }

    pub fn admitted_snapshot(&self) -> &AdmittedSignalBranchSnapshot {
        &self.snapshot
    }

    pub fn captured_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.captured_basis
    }

    pub fn into_parts(self) -> (AdmittedSignalBranchSnapshot, AdmittedSignalBranchBasis) {
        (self.snapshot, self.captured_basis)
    }
}
