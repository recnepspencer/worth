//! Borrow actual conventional evidence; never reconstruct an authorization plan.
use super::*;
impl WorthQueryAuthorizationDecisionFact {
    pub fn source_observation(&self) -> Option<&RelationalAuthorizationObservationEvidence> {
        (self.recorded_actor.is_none()
            && self.delegation.is_none()
            && self.delegation_activation.is_none()
            && self.native_dependencies.is_none())
        .then_some(self.relational.as_ref())
    }
}
