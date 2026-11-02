//! Borrow actual conventional evidence; never reconstruct an authorization plan.
use super::*;
impl WorthQueryAuthorizationDecisionFact {
    pub(in crate::domain_computation::authorization) fn source_observation(
        &self,
    ) -> Option<&RelationalAuthorizationObservationEvidence> {
        (self.recorded_actor.is_none()
            && self.delegation.is_none()
            && self.delegation_activation.is_none()
            && self.native_dependencies.is_none()
            && self.relational.proposed_effects().is_empty())
        .then_some(self.relational.as_ref())
    }
}
