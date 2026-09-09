use super::*;

impl UiMountedPublicationTransition {
    pub(super) fn new(outcome: UiMountedFrameOutcome) -> Self {
        Self {
            outcome,
            observation: None,
            appearance: None,
            hit_transition: None,
        }
    }

    pub(super) fn with_observation(
        outcome: UiMountedFrameOutcome,
        observation: UiMountedHostObservationTransition,
    ) -> Self {
        Self {
            outcome,
            observation: Some(observation),
            appearance: None,
            hit_transition: None,
        }
    }

    pub(super) fn with_observation_and_appearance(
        outcome: UiMountedFrameOutcome,
        observation: UiMountedHostObservationTransition,
        appearance: crate::runtime::appearance::UiAppearanceInspectionAttemptBatch,
    ) -> Self {
        Self {
            outcome,
            observation: Some(observation),
            appearance: Some(appearance),
            hit_transition: None,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        UiMountedFrameOutcome,
        Option<UiMountedHostObservationTransition>,
        Option<crate::runtime::appearance::UiAppearanceInspectionAttemptBatch>,
        Option<crate::mounting::UiCommittedPresentedHitTransition>,
    ) {
        (
            self.outcome,
            self.observation,
            self.appearance,
            self.hit_transition,
        )
    }
}
