#[cfg(any(test, feature = "certification-support"))]
use super::UiIntentOccupancyObservation;
use super::UiIntentOperabilityDecision;

#[must_use]
pub enum UiIntentOperabilityOutcome {
    Operable(UiIntentOperabilityProof),
    Inoperable(UiInoperableIntentCandidate),
}

#[must_use]
pub struct UiIntentOperabilityProof {
    candidate: super::super::payload::UiPreparedIntentPayload,
    decision: UiIntentOperabilityDecision,
}

#[must_use]
pub struct UiInoperableIntentCandidate {
    candidate: super::super::payload::UiPreparedIntentPayload,
    decision: UiIntentOperabilityDecision,
}

impl UiIntentOperabilityProof {
    pub(crate) fn new(
        candidate: super::super::payload::UiPreparedIntentPayload,
        decision: UiIntentOperabilityDecision,
    ) -> Self {
        debug_assert!(decision.is_operable());
        Self {
            candidate,
            decision,
        }
    }

    pub const fn decision(&self) -> &UiIntentOperabilityDecision {
        &self.decision
    }

    pub fn declaration_identity(&self) -> &str {
        self.candidate.declaration_identity()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn occupancy_observation(&self) -> &UiIntentOccupancyObservation {
        self.candidate.operability_basis().occupancy()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        super::super::payload::UiPreparedIntentPayload,
        UiIntentOperabilityDecision,
    ) {
        (self.candidate, self.decision)
    }

    pub(crate) const fn candidate_for_standing_fact(
        &self,
    ) -> &super::super::payload::UiPreparedIntentPayload {
        &self.candidate
    }
}

impl UiInoperableIntentCandidate {
    pub(crate) fn new(
        candidate: super::super::payload::UiPreparedIntentPayload,
        decision: UiIntentOperabilityDecision,
    ) -> Self {
        debug_assert!(!decision.is_operable());
        Self {
            candidate,
            decision,
        }
    }

    pub const fn decision(&self) -> &UiIntentOperabilityDecision {
        &self.decision
    }

    pub fn declaration_identity(&self) -> &str {
        self.candidate.declaration_identity()
    }

    pub(crate) const fn candidate(&self) -> &super::super::payload::UiPreparedIntentPayload {
        &self.candidate
    }

    pub(crate) fn is_exclusively_confirmable(&self) -> bool {
        let mut causes = self.decision.causes();
        matches!(
            causes.next(),
            Some(super::UiIntentInoperableCause::ConfirmationRequired { .. })
        ) && causes.next().is_none()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        super::super::payload::UiPreparedIntentPayload,
        UiIntentOperabilityDecision,
    ) {
        (self.candidate, self.decision)
    }
}
