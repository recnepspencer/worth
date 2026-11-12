//! Initial temporal posture under the existing partition registry mutex.
use super::SignalConditionalTemporalRegistry;
use crate::branch::SignalOwnerStopDenial as Denial;
impl SignalConditionalTemporalRegistry {
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
