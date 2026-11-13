//! Stop admission distinguishes untouched state from actually retired partitions.
use super::SignalConditionalTemporalRegistry;
use crate::branch::SignalOwnerStopDenial as Denial;
impl SignalConditionalTemporalRegistry {
    pub(in crate::branch::owner_services) fn try_authored_home_stop_guard(
        &self,
    ) -> Result<impl Drop + '_, Denial> {
        let state = self.state.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => Denial::Busy,
            std::sync::TryLockError::Poisoned(_) => Denial::Poisoned,
        })?;
        // The live partition owner retired every cell. Its process-local
        // allocation frontier need not be zero for a graceful authored close.
        if !state.cells.is_empty() {
            return Err(Denial::Unsupported);
        }
        Ok(state)
    }
    pub(in crate::branch::owner_services) fn try_initial_empty_stop_guard(
        &self,
    ) -> Result<impl Drop + '_, Denial> {
        let state = self.state.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => Denial::Busy,
            std::sync::TryLockError::Poisoned(_) => Denial::Poisoned,
        })?;
        if !state.cells.is_empty() || state.next_partition_id != 0 {
            return Err(Denial::Unsupported);
        }
        Ok(state)
    }
}
