//! Nonwaiting release of an original published result's observation obligation.
use super::*;
#[cfg(test)]
mod tests;

impl RelationalRetentionGuard {
    pub(crate) fn try_retire_observation(&mut self) -> bool {
        use std::sync::TryLockError;
        if self.terminal {
            return true;
        }
        let Some(owner) = self.owner.upgrade() else {
            return false;
        };
        let mut state = match self.lane.state.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return false,
        };
        if let Some(record) = state.obligations.get(&self.obligation_id) {
            assert_eq!(
                /* Proof: Only RelationalObservationRetentionObligation::try_retire calls this private door; its acquire fixes Observation and one root, and the non-Clone guard retains that exact registration. */
                record.kind(),
                RelationalRetentionObligationKind::Observation
            );
        }
        let retired = state.release_obligation(self.obligation_id, &self.roots, &owner);
        self.terminal = true;
        drop(state);
        drop(retired);
        self.roots.clear();
        true
    }
}
