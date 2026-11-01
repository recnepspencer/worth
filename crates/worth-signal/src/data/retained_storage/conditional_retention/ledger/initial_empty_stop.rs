//! The initial-owner stop closes acquisition without releasing retained values.
use super::{RetainedConditionalUsage, SignalConditionalRetentionLedger};
use crate::branch::SignalInitialEmptyStopDenial as Denial;
use std::sync::MutexGuard;
pub(crate) struct SignalInitialConditionalCloseGuard<'a>(MutexGuard<'a, RetainedConditionalUsage>);
impl SignalConditionalRetentionLedger {
    pub(crate) fn try_initial_empty_stop_guard(
        &self,
    ) -> Result<SignalInitialConditionalCloseGuard<'_>, Denial> {
        let state = self.state.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => Denial::Busy,
            std::sync::TryLockError::Poisoned(_) => Denial::Poisoned,
        })?;
        if state.closed {
            return Err(Denial::OwnerUnavailable);
        }
        Ok(SignalInitialConditionalCloseGuard(state))
    }
}
impl SignalInitialConditionalCloseGuard<'_> {
    pub(crate) fn close(&mut self) {
        self.0.closed = true;
    }
}
