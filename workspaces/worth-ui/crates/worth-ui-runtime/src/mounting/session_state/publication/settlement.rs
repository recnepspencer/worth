use super::{
    indeterminate_observation, presentation_attempt, UiMountedFrameOutcome,
    UiMountedHostObservationTransition, UiMountedPresentationInFlight,
    UiMountedPresentationOutcome, UiMountedPublicationTransition, WorthUiMountedSessionState,
};

impl WorthUiMountedSessionState {
    pub(crate) fn complete_presentation(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        in_flight: UiMountedPresentationInFlight,
        now: u64,
    ) -> UiMountedPublicationTransition {
        match self
            .presentation
            .complete(in_flight, host.effect_port(), now)
        {
            Ok(outcome) => self.finish_presentation(outcome),
            Err(denial) => {
                UiMountedPublicationTransition::new(UiMountedFrameOutcome::CompletionDenied(denial))
            }
        }
    }

    pub(crate) fn cancel_presentation(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        in_flight: UiMountedPresentationInFlight,
    ) -> UiMountedPublicationTransition {
        match self.presentation.cancel(in_flight, host.effect_port()) {
            Ok(outcome) => self.finish_presentation(outcome),
            Err(denial) => {
                UiMountedPublicationTransition::new(UiMountedFrameOutcome::CompletionDenied(denial))
            }
        }
    }

    pub(crate) fn supersede_presentation(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        in_flight: UiMountedPresentationInFlight,
    ) -> UiMountedPublicationTransition {
        match self.presentation.supersede(in_flight, host.effect_port()) {
            Ok(outcome) => self.finish_presentation(outcome),
            Err(denial) => {
                UiMountedPublicationTransition::new(UiMountedFrameOutcome::CompletionDenied(denial))
            }
        }
    }

    pub(crate) fn admit_duplicate_native_presentation_observation(
        &mut self,
        presentation: worth_ui_host_native::UiNativePhysicalPresentationCorrelation,
    ) -> Result<(), ()> {
        self.presentation
            .admit_duplicate_native_presentation_observation(presentation)
    }

    pub(crate) fn finish_presentation(
        &mut self,
        outcome: UiMountedPresentationOutcome,
    ) -> UiMountedPublicationTransition {
        let hit_predecessor = self.retention.current_hit_evidence();
        let attempt = presentation_attempt(&outcome);
        let retain_appearance = matches!(&outcome, UiMountedPresentationOutcome::InFlight(_));
        let appearance_batch = (!retain_appearance)
            .then(|| self.presentation.take_appearance_attempt(attempt))
            .flatten();
        let mut transition = if self.reconciliation_reservations.contains_key(&attempt) {
            self.finish_reconciliation(outcome, attempt)
        } else {
            match outcome {
                UiMountedPresentationOutcome::Presented(presented) => {
                    let attempt = presented.receipt().attempt();
                    let reservation = self
                        .publication_reservations
                        .remove(&attempt)
                        .expect("every presented attempt has a pre-effect publication reservation");
                    match reservation.commit_presented(presented, &mut self.identity) {
                        crate::mounting::UiMountedFramePublicationCommit::Current(receipt) => {
                            self.retention.refresh_presented_hit_motion(
                                &self.motion_sampling,
                                &self.motion_sampling.retained_targets(),
                            );
                            UiMountedPublicationTransition::new(UiMountedFrameOutcome::Published(
                                receipt,
                            ))
                        }
                        crate::mounting::UiMountedFramePublicationCommit::Superseded(frame) => {
                            UiMountedPublicationTransition::new(UiMountedFrameOutcome::Superseded(
                                frame,
                            ))
                        }
                    }
                }
                UiMountedPresentationOutcome::RejectedBeforeEffects(rejected) => {
                    self.remove_publication_reservation(rejected.attempt());
                    let frame = rejected.frame().canonical_core().frame();
                    UiMountedPublicationTransition::with_observation(
                        UiMountedFrameOutcome::RejectedBeforeEffects(rejected),
                        UiMountedHostObservationTransition::Rejected(frame),
                    )
                }
                UiMountedPresentationOutcome::Superseded(superseded) => {
                    self.remove_publication_reservation(superseded.attempt());
                    UiMountedPublicationTransition::new(UiMountedFrameOutcome::Superseded(
                        superseded,
                    ))
                }
                UiMountedPresentationOutcome::InFlight(in_flight) => {
                    UiMountedPublicationTransition::new(UiMountedFrameOutcome::InFlight(in_flight))
                }
                UiMountedPresentationOutcome::PresentationIndeterminate(indeterminate) => {
                    self.remove_publication_reservation(indeterminate.report().attempt());
                    let observation = indeterminate_observation(&indeterminate);
                    UiMountedPublicationTransition::with_observation(
                        UiMountedFrameOutcome::PresentationIndeterminate(indeterminate),
                        observation,
                    )
                }
            }
        };
        if matches!(
            transition.outcome,
            UiMountedFrameOutcome::Published(_)
                | UiMountedFrameOutcome::Unchanged(_)
                | UiMountedFrameOutcome::Reconciled(_)
        ) {
            transition.hit_transition = self.retention.committed_hit_transition(hit_predecessor);
        }
        transition.appearance = appearance_batch;
        transition
    }
}
