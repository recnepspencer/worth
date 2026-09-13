use std::sync::atomic::Ordering;

use crate::state::SignalBranchHandle;

use super::super::{SignalBranchCellState, SignalOwnerOperationAdmission};
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn descriptive_handle(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
    ) -> Result<SignalBranchHandle, SignalBranchCellAdmissionDenial> {
        self.validate_admission(admission)?;
        let _cell_hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        self.require_live_posture()?;
        let state = self.lock_state_after_contention_observation()?;
        self.require_live_posture()?;
        Ok(state.handle().clone())
    }
}
