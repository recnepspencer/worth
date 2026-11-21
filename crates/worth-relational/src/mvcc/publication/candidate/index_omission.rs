//! Omit this live candidate's genuine index batch for the composite refusal court.
use super::PreparedRelationalCommitCandidate;
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;

impl PreparedRelationalCommitCandidate {
    /// Fault only this receiver after automatic preparation has completed.
    /// Missing custody refuses rather than accepting an ineffective fault.
    pub fn omit_prepared_index_generations_for_test(&mut self) -> Result<(), Denial> {
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
