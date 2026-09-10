use crate::state::SignalBranchHandle;
use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::data::error::SignalError;
use crate::state::SignalBranchId;

use super::{
    AdmittedSignalBranchBasis, SignalBranchRetentionAcquisitionDenial, SignalOwnerUnavailable,
};
use super::{ManagedSignalBranchReference, SignalBranchIdentityConstructionDenial};

#[derive(Debug)]
pub enum SignalBranchForkOperationDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OperationCapacityExhausted {
        maximum_in_flight_operations: usize,
    },
    OwnerReentry,
    CancelledNoMovement,
    LiveBranchCapacityExhausted {
        maximum_live_branches: usize,
    },
    ReservationCapacityExhausted {
        maximum_reservations: usize,
    },
    NameAlreadyReserved,
    NameAlreadyInstalled,
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
    InvalidIdentity {
        denial: SignalBranchIdentityConstructionDenial,
    },
    BranchIdentityExhausted,
    OwnerDeniedNoMovement {
        error: SignalError,
    },
}

/// Owner-issued result of a canonical Signal branch fork.
#[derive(Debug, Clone)]
pub struct SignalBranchForkOutcome {
    created_branch: SignalBranchHandle,
    created_basis: AdmittedSignalBranchBasis,
    retirement_reference: Option<ManagedSignalBranchReference>,
}

impl SignalBranchForkOutcome {
    pub(crate) fn owner_issued(
        created_branch: SignalBranchHandle,
        created_basis: AdmittedSignalBranchBasis,
        retirement_reference: ManagedSignalBranchReference,
    ) -> Self {
        Self {
            created_branch,
            created_basis,
            retirement_reference: Some(retirement_reference),
        }
    }

    pub(crate) fn owner_issued_without_service_reference(
        created_branch: SignalBranchHandle,
        created_basis: AdmittedSignalBranchBasis,
    ) -> Self {
        Self {
            created_branch,
            created_basis,
            retirement_reference: None,
        }
    }

    pub fn created_branch(&self) -> &SignalBranchHandle {
        &self.created_branch
    }

    pub fn created_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.created_basis
    }

    /// Owner-issued, incarnation-bound authority for later exact observation
    /// and retirement of the created branch.
    pub fn retirement_reference(&self) -> Option<&ManagedSignalBranchReference> {
        self.retirement_reference.as_ref()
    }

    pub fn into_parts(self) -> (SignalBranchHandle, AdmittedSignalBranchBasis) {
        (self.created_branch, self.created_basis)
    }
}
