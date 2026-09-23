use super::{UiMountedFrameOutcome, WorthUiActiveApplicationSession};
use crate::facade::entry::{active_application_session, intent_consequence_observation};

impl WorthUiActiveApplicationSession {
    pub(crate) fn present_prepared_mounted_frame_internal(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> UiMountedFrameOutcome {
        let overlays = self.prepare_overlay_appearance_sources();
        self.present_prepared_mounted_frame_with_overlay_sources(
            frame, deadline, now, overlays, None,
        )
    }

    pub(crate) fn present_prepared_portal_frame_internal(
        &mut self,
        mut frame: crate::mounting::UiPreparedMountedFrame,
        proposal: &crate::runtime::session::UiStagedPortalProposalTransaction,
        retain_exit: bool,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> UiMountedFrameOutcome {
        frame
            .bind_motion_entrance(
                self.mounted
                    .prepare_motion_entrance(proposal.prepared_motion_entrance()),
            )
            .expect("the Portal proposal was derived from this prepared frame");
        let (transition, stage, staged_motion) = proposal.overlay_appearance_sources();
        let overlays = self.prepare_overlay_appearance_sources_for_portal_transition(
            transition,
            stage,
            staged_motion,
            retain_exit,
        );
        self.present_prepared_mounted_frame_with_overlay_sources(
            frame, deadline, now, overlays, None,
        )
    }

    pub(in crate::facade::entry) fn present_prepared_observed_frame(
        &mut self,
        mut frame: crate::mounting::UiPreparedMountedFrame,
        observation: &intent_consequence_observation::WorthUiPreparedConsequenceObservationCommit,
        proposal: Option<&crate::runtime::session::UiStagedPortalProposalTransaction>,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, crate::runtime::rebind::UiRebindPreparationDenial> {
        observation.validate(self, &frame)?;
        frame.bind_motion_entrance(self.mounted.prepare_motion_entrance(
            proposal.and_then(|proposal| proposal.prepared_motion_entrance()),
        ))?;
        let overlays = match proposal {
            Some(proposal) => {
                let (transition, stage, motion) = proposal.overlay_appearance_sources();
                self.prepare_overlay_appearance_sources_for_portal_transition(
                    transition,
                    stage,
                    motion,
                    transition.closes_portal(),
                )
            }
            None => self.prepare_overlay_appearance_sources(),
        };
        Ok(self.present_prepared_mounted_frame_with_overlay_sources(
            frame,
            deadline,
            now,
            overlays,
            Some(observation),
        ))
    }

    pub(in crate::facade::entry) fn complete_prepared_observed_frame(
        &mut self,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
        observation: &intent_consequence_observation::WorthUiPreparedConsequenceObservationCommit,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, crate::runtime::rebind::UiRebindPreparationDenial> {
        if let Err(denial) = observation.validate_owner_predecessor(self) {
            let superseded = self.supersede_mounted_presentation(in_flight);
            if matches!(
                superseded,
                UiMountedFrameOutcome::PresentationIndeterminate(_)
            ) {
                return Ok(superseded);
            }
            return Err(denial);
        }
        Ok(self.complete_mounted_presentation(in_flight, now))
    }

    fn present_prepared_mounted_frame_with_overlay_sources(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        overlays: Result<active_application_session::UiActiveOverlayAppearancePreparation, ()>,
        observation: Option<
            &intent_consequence_observation::WorthUiPreparedConsequenceObservationCommit,
        >,
    ) -> UiMountedFrameOutcome {
        let owner_receipts = observation
            .is_none()
            .then(|| self.prepare_mounted_owner_receipt_succession(&frame));
        let generation = self.generation_identity().clone();
        let portal = self.portal.as_ref();
        let motion = self.motion.as_ref();
        let presentation = &self.presentation;
        let capabilities = self.application.capabilities();
        let appearance = match observation {
            Some(observation) => observation.appearance(),
            None => self.appearance_owner_snapshot.as_ref(),
        };
        let transition = self.mounted.present_prepared_frame_with_overlays(
            &self.host_session,
            frame,
            Some(&mut self.appearance_inspection),
            deadline,
            now,
            |attempt, surfaces, prepared_binding| {
                overlays.as_ref().map_err(|_| ())?.lower(
                    attempt,
                    surfaces,
                    &mut self.overlay_composition_owners,
                    &generation,
                    portal,
                    motion,
                    presentation,
                    capabilities,
                    appearance,
                    prepared_binding,
                )
            },
        );
        let outcome = self.finish_mounted_transition(transition, now);
        if let Some(prepared) = owner_receipts {
            self.settle_new_mounted_owner_receipt_succession(prepared, &outcome);
        }
        outcome
    }

    pub(crate) fn present_prepared_superseding_mounted_frame_internal(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        predecessor: crate::mounting::UiMountedSupersedingPresentationBasis,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> UiMountedFrameOutcome {
        let owner_receipts = self.prepare_mounted_owner_receipt_succession(&frame);
        let overlays = self.prepare_overlay_appearance_sources();
        let generation = self.generation_identity().clone();
        let portal = self.portal.as_ref();
        let motion = self.motion.as_ref();
        let presentation = &self.presentation;
        let capabilities = self.application.capabilities();
        let appearance = self.appearance_owner_snapshot.as_ref();
        let transition = self
            .mounted
            .present_prepared_superseding_frame_with_overlays(
                &self.host_session,
                frame,
                predecessor,
                Some(&mut self.appearance_inspection),
                deadline,
                now,
                |attempt, surfaces, prepared_binding| {
                    overlays.as_ref().map_err(|_| ())?.lower(
                        attempt,
                        surfaces,
                        &mut self.overlay_composition_owners,
                        &generation,
                        portal,
                        motion,
                        presentation,
                        capabilities,
                        appearance,
                        prepared_binding,
                    )
                },
            );
        let outcome = self.finish_mounted_transition(transition, now);
        self.settle_new_mounted_owner_receipt_succession(owner_receipts, &outcome);
        outcome
    }
}
