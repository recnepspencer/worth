use serde::{Deserialize, Serialize};

use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};

use super::SignalBranchBasisArtifact;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchForkRequestBasis {
    CurrentBranchHead,
    ParentBranchHead {
        parent_branch_id: SignalBranchId,
    },
    ParentBranchSnapshot {
        parent_branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalBranchForkRequest {
    branch_name: String,
    basis: SignalBranchForkRequestBasis,
}

impl SignalBranchForkRequest {
    pub fn from_current_branch_head(name: impl Into<String>) -> Self {
        Self {
            branch_name: name.into(),
            basis: SignalBranchForkRequestBasis::CurrentBranchHead,
        }
    }

    pub fn from_parent_branch_head(
        name: impl Into<String>,
        parent_branch_id: SignalBranchId,
    ) -> Self {
        Self {
            branch_name: name.into(),
            basis: SignalBranchForkRequestBasis::ParentBranchHead { parent_branch_id },
        }
    }

    pub fn from_parent_branch_snapshot(
        name: impl Into<String>,
        parent_branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    ) -> Self {
        Self {
            branch_name: name.into(),
            basis: SignalBranchForkRequestBasis::ParentBranchSnapshot {
                parent_branch_id,
                snapshot_id,
            },
        }
    }

    pub fn branch_name(&self) -> &str {
        &self.branch_name
    }

    pub fn basis(&self) -> &SignalBranchForkRequestBasis {
        &self.basis
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchForkDenial {
    InvalidBranchIdentity,
    BranchIdentityExhausted,
    UnknownParentBranch {
        parent_branch_id: SignalBranchId,
    },
    UnknownForkSnapshot {
        parent_branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    },
    SnapshotBasisMismatch {
        requested_snapshot_id: SignalSnapshotId,
        provided_snapshot_id: SignalSnapshotId,
    },
    SnapshotPayloadRequiredForFork {
        request: SignalBranchForkRequest,
    },
    IncompatibleForkSnapshotLineage {
        parent_branch_id: SignalBranchId,
        snapshot_branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    },
    ManagedQueueBranchTransferDenied {
        bound_queue_count: u32,
    },
}

#[derive(Debug, Clone)]
pub struct SignalBranchForkReceipt {
    pub(super) request: SignalBranchForkRequest,
    pub(super) parent_basis: SignalBranchBasisArtifact,
    pub(super) requested_snapshot_basis: Option<SignalBranchBasisArtifact>,
    pub(super) created_branch: SignalBranchHandle,
    pub(super) created_branch_basis: SignalBranchBasisArtifact,
    pub(super) active_branch_after_fork_basis: SignalBranchBasisArtifact,
}

impl SignalBranchForkReceipt {
    pub(crate) fn request(&self) -> &SignalBranchForkRequest {
        &self.request
    }

    pub(crate) fn parent_basis(&self) -> &SignalBranchBasisArtifact {
        &self.parent_basis
    }

    pub(crate) fn requested_snapshot_basis(&self) -> Option<&SignalBranchBasisArtifact> {
        self.requested_snapshot_basis.as_ref()
    }

    pub fn created_branch(&self) -> &SignalBranchHandle {
        &self.created_branch
    }

    pub(crate) fn created_branch_basis(&self) -> &SignalBranchBasisArtifact {
        &self.created_branch_basis
    }

    pub(crate) fn active_branch_after_fork_basis(&self) -> &SignalBranchBasisArtifact {
        &self.active_branch_after_fork_basis
    }
}
