//! Omit this live candidate's genuine index batch for the composite refusal court.
use super::PreparedRelationalCommitCandidate;
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;

impl PreparedRelationalCommitCandidate {
    /// Fault only this receiver after automatic preparation has completed.
    /// Missing or expired custody refuses rather than accepting an ineffective fault.
    pub fn omit_prepared_index_generations_for_test(&mut self) -> Result<(), Denial> {
        if self.lifetime_expired() {
            return Err(Denial::CandidateLifetimeExpired {
                maximum_lifetime_millis: self.maximum_lifetime_millis,
            });
        }
        let mut payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        payload
            .as_mut()
            .ok_or(Denial::CandidateUnavailable)?
            .execution
            .discard_prepared_index_publication()
    }
}
