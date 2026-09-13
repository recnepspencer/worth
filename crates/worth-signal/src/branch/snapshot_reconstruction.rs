use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::data::error::SignalError;
use crate::state::SignalBranchId;

use super::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchRetentionAcquisitionDenial,
};

#[derive(Debug)]
pub enum SignalBranchSnapshotReconstructionDenial {
    UnknownBranch {
        branch_id: SignalBranchId,
    },
    BasisMismatch {
        axes: Vec<FoundationalBranchReferenceMismatchAxis>,
    },
    NonPristineBranch {
        branch_id: SignalBranchId,
    },
    NonPristineRuntime,
    InactiveBranch {
        branch_id: SignalBranchId,
    },
    CrossBranchSnapshot {
        branch_id: SignalBranchId,
        snapshot_branch_id: SignalBranchId,
    },
    RetentionUnavailable {
        denial: SignalBranchRetentionAcquisitionDenial,
    },
    SnapshotCapacityExhausted {
        maximum_stored_snapshots: usize,
    },
    OwnerDeniedNoMovement {
        error: SignalError,
    },
}

/// Owner-issued result of admitting one portable snapshot during construction.
#[derive(Debug)]
pub struct SignalBranchSnapshotReconstructionOutcome {
    snapshot: AdmittedSignalBranchSnapshot,
    reconstructed_basis: AdmittedSignalBranchBasis,
}

impl SignalBranchSnapshotReconstructionOutcome {
    pub(crate) fn owner_issued(
        snapshot: AdmittedSignalBranchSnapshot,
        reconstructed_basis: AdmittedSignalBranchBasis,
    ) -> Self {
        Self {
            snapshot,
            reconstructed_basis,
        }
    }

    pub fn admitted_snapshot(&self) -> &AdmittedSignalBranchSnapshot {
        &self.snapshot
    }

    pub fn reconstructed_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.reconstructed_basis
    }

    pub fn into_parts(self) -> (AdmittedSignalBranchSnapshot, AdmittedSignalBranchBasis) {
        (self.snapshot, self.reconstructed_basis)
    }
}
