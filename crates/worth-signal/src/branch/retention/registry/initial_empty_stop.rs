//! Retain existing lease custody while fencing future acquisitions.
use super::{SignalBranchRetentionRegistry, SignalRetentionLedgerState};
use crate::branch::SignalInitialEmptyStopDenial as Denial;
use std::sync::MutexGuard;
pub(crate) struct SignalInitialRetentionCloseGuard<'a>(MutexGuard<'a, SignalRetentionLedgerState>);
impl SignalBranchRetentionRegistry {
    pub(crate) fn try_initial_empty_stop_guard(
        &self,
    ) -> Result<SignalInitialRetentionCloseGuard<'_>, Denial> {
        let state = self.ledger.state.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => Denial::Busy,
            std::sync::TryLockError::Poisoned(_) => Denial::Poisoned,
        })?;
        if !state.owner_open {
            return Err(Denial::OwnerUnavailable);
        }
        Ok(SignalInitialRetentionCloseGuard(state))
    }
}
impl SignalInitialRetentionCloseGuard<'_> {
    pub(crate) fn close(&mut self) {
        self.0.owner_open = false;
    }
}
