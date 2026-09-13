use std::sync::atomic::Ordering;

use crate::branch::{SignalBranchBasisObservationDenial, SignalBranchObservation};

use super::super::{SignalBranchCellState, SignalOwnerOperationAdmission, SignalOwnerUnavailable};
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Exact one-cell observation seam for basis observation and readmission.
    pub(crate) fn observe_exact(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
    ) -> Result<SignalBranchObservation, SignalBranchBasisObservationDenial> {
        self.validate_admission(admission)
            .map_err(|denial| map_basis_cell_denial(denial, self.branch_id))?;
        let _cell_hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)
            .map_err(|denial| map_basis_cell_denial(denial, self.branch_id))?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        self.require_live_posture()
            .map_err(|denial| map_basis_cell_denial(denial, self.branch_id))?;
        let state = self
            .lock_state_after_contention_observation()
            .map_err(|denial| map_basis_cell_denial(denial, self.branch_id))?;
        self.require_live_posture()
            .map_err(|denial| map_basis_cell_denial(denial, self.branch_id))?;
        state
            .observation()
            .map_err(|error| SignalBranchBasisObservationDenial::InvalidOwnerObservation { error })
    }
}

pub(in crate::branch::owner_services) fn map_basis_cell_denial(
    denial: SignalBranchCellAdmissionDenial,
    branch_id: crate::state::SignalBranchId,
) -> SignalBranchBasisObservationDenial {
    match denial {
        SignalBranchCellAdmissionDenial::ForeignOwner
        | SignalBranchCellAdmissionDenial::ExpiredLifecycle => {
            SignalBranchBasisObservationDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchCellAdmissionDenial::SecondCellWhileHeld => {
            SignalBranchBasisObservationDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchCellAdmissionDenial::ExecutingThreadReentry => {
            SignalBranchBasisObservationDenial::OwnerReentry
        }
        SignalBranchCellAdmissionDenial::RetirementInProgress => {
            SignalBranchBasisObservationDenial::RetirementInProgress { branch_id }
        }
        SignalBranchCellAdmissionDenial::RetiredIncarnation => {
            SignalBranchBasisObservationDenial::RetiredBranch { branch_id }
        }
        SignalBranchCellAdmissionDenial::PoisonedIncarnation => {
            SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id }
        }
    }
}
