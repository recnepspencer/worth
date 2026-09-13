use serde::{Deserialize, Serialize};

use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};

use super::{AdmittedSignalBranchBasis, SignalOwnerUnavailable};

/// Descriptive lifecycle posture bound into a transported Signal basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchBasisLifecyclePosture {
    Live,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchRetirementReason {
    Rejected,
    Merged,
    Superseded,
    DependencyCancellation,
    ProjectionRebuild,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchRetirementDenial {
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
    CurrentBranch {
        branch_id: SignalBranchId,
    },
    CanonicalBranch {
        branch_id: SignalBranchId,
    },
    StaleBranchHead {
        expected_generation: u64,
        observed_generation: u64,
    },
    CanonicalBasisMismatch,
    LiveChildren {
        branch_id: SignalBranchId,
        child_branch_ids: Vec<SignalBranchId>,
    },
    MergeParticipant {
        branch_id: SignalBranchId,
    },
    RetainedComponentBasis {
        branch_id: SignalBranchId,
        active_leases: u32,
    },
    RetainedAdmittedBasis {
        branch_id: SignalBranchId,
        active_leases: u32,
    },
    SharedAdmittedBasis {
        branch_id: SignalBranchId,
        shared_holders: usize,
    },
    ForeignRetirementSnapshot {
        expected_runtime_instance_id: u64,
        observed_runtime_instance_id: u64,
    },
    RetirementSnapshotBranchMismatch {
        branch_id: SignalBranchId,
        snapshot_branch_id: SignalBranchId,
    },
    RetirementReceiptCapacityExhausted {
        maximum_retained_receipts: usize,
    },
    OwnerInvariantViolation {
        branch_id: SignalBranchId,
    },
}

/// Linear owner-issued plan. The admitted basis remains held until execution,
/// so no production retirement path can bypass canonical Signal authority.
#[derive(Debug)]
pub struct PlannedSignalBranchRetirement {
    pub(crate) branch: SignalBranchHandle,
    pub(crate) reason: SignalBranchRetirementReason,
    pub(crate) terminal_basis_digest: String,
    pub(crate) planned_child_membership_count: u32,
    pub(crate) admitted_basis: AdmittedSignalBranchBasis,
}

impl PlannedSignalBranchRetirement {
    pub fn planned_child_membership_count(&self) -> u32 {
        self.planned_child_membership_count
    }

    pub(crate) fn branch(&self) -> &SignalBranchHandle {
        &self.branch
    }

    pub(crate) fn reason(&self) -> SignalBranchRetirementReason {
        self.reason
    }

    pub(crate) fn admitted_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.admitted_basis
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchRetirementBatchDenial {
    Empty,
    DuplicateBranch {
        branch_id: SignalBranchId,
    },
    Retirement {
        position: u32,
        denial: SignalBranchRetirementDenial,
    },
}

/// Linear owner-issued batch of individually authorized retirement plans.
#[derive(Debug)]
pub struct PlannedSignalBranchRetirementBatch {
    pub(crate) plans: Vec<PlannedSignalBranchRetirement>,
}

impl PlannedSignalBranchRetirementBatch {
    pub fn breadth(&self) -> u32 {
        self.plans.len() as u32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalBranchRetirementReceipt {
    pub(crate) retired_branch: SignalBranchHandle,
    pub(crate) parent_branch_id: SignalBranchId,
    pub(crate) forked_from_snapshot_id: Option<SignalSnapshotId>,
    pub(crate) terminal_head_snapshot_id: Option<SignalSnapshotId>,
    pub(crate) reason: SignalBranchRetirementReason,
    pub(crate) terminal_basis_digest: String,
    pub(crate) closeout_digest: String,
    pub(crate) reclaimed_branch_state_count: u32,
    pub(crate) reclaimed_snapshot_state_count: u32,
    pub(crate) reclaimed_runtime_meta_count: u32,
    pub(crate) retained_proof_record_count: u32,
}

impl SignalBranchRetirementReceipt {
    pub fn retired_branch(&self) -> &SignalBranchHandle {
        &self.retired_branch
    }

    pub fn parent_branch_id(&self) -> SignalBranchId {
        self.parent_branch_id
    }

    pub fn forked_from_snapshot_id(&self) -> Option<SignalSnapshotId> {
        self.forked_from_snapshot_id
    }

    pub fn terminal_head_snapshot_id(&self) -> Option<SignalSnapshotId> {
        self.terminal_head_snapshot_id
    }

    pub fn reason(&self) -> SignalBranchRetirementReason {
        self.reason
    }

    pub fn terminal_basis_digest(&self) -> &str {
        &self.terminal_basis_digest
    }

    pub fn closeout_digest(&self) -> &str {
        &self.closeout_digest
    }

    pub fn reclaimed_branch_state_count(&self) -> u32 {
        self.reclaimed_branch_state_count
    }

    pub fn reclaimed_snapshot_state_count(&self) -> u32 {
        self.reclaimed_snapshot_state_count
    }

    pub fn reclaimed_runtime_meta_count(&self) -> u32 {
        self.reclaimed_runtime_meta_count
    }

    pub fn retained_proof_record_count(&self) -> u32 {
        self.retained_proof_record_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalBranchRetirementBatchReceipt {
    receipts: Vec<SignalBranchRetirementReceipt>,
}

impl SignalBranchRetirementBatchReceipt {
    pub fn receipts(&self) -> &[SignalBranchRetirementReceipt] {
        &self.receipts
    }

    pub(crate) fn new(receipts: Vec<SignalBranchRetirementReceipt>) -> Self {
        Self { receipts }
    }
}
