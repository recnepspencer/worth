//! Borrowed comparison of the original witness against the actual locked cell.
use super::{SignalBranchExecutionCell, CELL_LIVE};
use crate::branch::owner_services::SignalBranchCellState;
use crate::branch::{SignalBranchObservation, SignalInitialEmptyStopDenial as Denial};
use std::sync::atomic::Ordering;
impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn try_initial_empty_stop_guard(
        &self,
        incarnation: u64,
        witness: &SignalBranchObservation,
    ) -> Result<impl Drop + '_, Denial> {
        let state = self.state.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => Denial::Busy,
            std::sync::TryLockError::Poisoned(_) => Denial::Poisoned,
        })?;
        if self.incarnation.get() != incarnation
            || self.lifecycle_posture.load(Ordering::Acquire) != CELL_LIVE
            || !state.matches_initial_empty_stop(witness)
        {
            return Err(Denial::Unsupported);
        }
        Ok(state)
    }
}
