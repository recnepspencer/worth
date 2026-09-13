use std::sync::atomic::Ordering;

use crate::branch::{SignalBranchObservation, SignalBranchRetirementDenial};
use crate::state::SignalBranchHandle;

use super::super::{SignalBranchCellState, SignalOwnerOperationAdmission};
use super::retirement::map_retirement_cell_denial;
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

pub(in crate::branch::owner_services) struct SignalBranchRetirementPlanningCellFacts {
    pub(in crate::branch::owner_services) branch: SignalBranchHandle,
    pub(in crate::branch::owner_services) observation: SignalBranchObservation,
}

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn retirement_planning_facts(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
    ) -> Result<SignalBranchRetirementPlanningCellFacts, SignalBranchRetirementDenial> {
        self.validate_admission(admission)
            .map_err(|denial| map_retirement_cell_denial(denial, self.branch_id))?;
        let _cell_hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)
            .map_err(|denial| map_retirement_cell_denial(denial, self.branch_id))?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        self.require_live_posture()
            .map_err(|denial| map_retirement_cell_denial(denial, self.branch_id))?;
        let state = self
            .lock_state_after_contention_observation()
            .map_err(|denial| map_retirement_cell_denial(denial, self.branch_id))?;
        self.require_live_posture()
            .map_err(|denial| map_retirement_cell_denial(denial, self.branch_id))?;
        let observation = state.observation().map_err(|_| {
            SignalBranchRetirementDenial::OwnerInvariantViolation {
                branch_id: self.branch_id,
            }
        })?;
        Ok(SignalBranchRetirementPlanningCellFacts {
            branch: state.handle().clone(),
            observation,
        })
    }
}
