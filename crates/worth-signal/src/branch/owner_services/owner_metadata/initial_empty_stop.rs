//! Initial history proof under its real owner mutex.
use super::SignalOwnerMetadata;
use crate::branch::SignalInitialEmptyStopDenial as Denial;
use std::sync::atomic::Ordering;
impl<D, I, T> SignalOwnerMetadata<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn try_initial_empty_stop_guard(
        &self,
    ) -> Result<impl Drop + '_, Denial> {
        let state = self.state.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => Denial::Busy,
            std::sync::TryLockError::Poisoned(_) => Denial::Poisoned,
        })?;
        if self.pending_snapshot_reservations.load(Ordering::Acquire) != 0
            || !state.has_initial_empty_history()
        {
            return Err(Denial::Unsupported);
        }
        Ok(state)
    }
}
