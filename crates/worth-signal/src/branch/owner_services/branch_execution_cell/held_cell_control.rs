//! Deterministic control holds the real target cell without exposing its state.
use super::super::SignalOwnerOperationAdmission;
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};
impl<S> SignalBranchExecutionCell<S> {
    pub(in crate::branch::owner_services) fn with_held_cell_control<R>(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        hold: impl FnOnce() -> R,
    ) -> Result<R, SignalBranchCellAdmissionDenial> {
        self.validate_admission(admission)?;
        let _hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)?;
        let _state = self.lock_state_after_contention_observation()?;
        self.require_live_posture()?;
        Ok(hold())
    }
}
