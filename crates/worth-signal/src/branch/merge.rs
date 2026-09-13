use std::ops::Deref;

use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::data::error::SignalError;
use crate::logic::transaction::BranchMergeResult;
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::{AdmittedSignalBranchBasis, SignalBranchRetentionAcquisitionDenial};

/// Owner-issued result of a merge into one canonical Signal branch head.
#[derive(Debug)]
pub struct SignalBranchMergeOutcome {
    target_basis: AdmittedSignalBranchBasis,
    result: BranchMergeResult,
}

impl SignalBranchMergeOutcome {
    pub(crate) fn owner_issued(
        target_basis: AdmittedSignalBranchBasis,
        result: BranchMergeResult,
    ) -> Self {
        Self {
            target_basis,
            result,
        }
    }

    pub fn target_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.target_basis
    }

    pub fn result(&self) -> &BranchMergeResult {
        &self.result
    }

    pub fn into_parts(self) -> (AdmittedSignalBranchBasis, BranchMergeResult) {
        (self.target_basis, self.result)
    }

    pub fn into_basis(self) -> AdmittedSignalBranchBasis {
        self.target_basis
    }
}

impl Deref for SignalBranchMergeOutcome {
    type Target = BranchMergeResult;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

#[derive(Debug)]
pub enum SignalBranchMergeDenial {
    UnknownSourceBranch {
        branch_id: SignalBranchId,
    },
    UnknownTargetBranch {
        branch_id: SignalBranchId,
    },
    SourceBasisMismatch {
        axes: Vec<FoundationalBranchReferenceMismatchAxis>,
    },
    TargetBasisMismatch {
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
    OwnerFailed {
        error: SignalError,
    },
}
