mod focus_settlement;
use focus_settlement::{place_reconciled_focus, reconcile_focus_after_published_frame_with_ports};
mod observation;
mod reconciliation;

pub(super) use observation::record_mounted_observation;

use crate::mounting::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, UiMountedPresentationInFlight,
    UiMountedPresentationOutcome,
};

use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    #[cfg(test)]
    pub(crate) fn current_mounted_projection_rc_for_test(
        &self,
    ) -> Option<std::rc::Rc<crate::mounting::UiMountedProjectionFrame>> {
        self.mounted.current_projection_rc_for_test()
    }

    pub(crate) fn present_prepared_mounted_frame_internal(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> UiMountedFrameOutcome {
        let overlays = self.prepare_overlay_appearance_sources();
        self.present_prepared_mounted_frame_with_overlay_sources(frame, deadline, now, overlays)
    }

    pub(crate) fn present_prepared_portal_frame_internal(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        proposal: &crate::runtime::session::UiStagedPortalProposalTransaction,
        retain_exit: bool,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> UiMountedFrameOutcome {
        let (transition, stage, staged_motion) = proposal.overlay_appearance_sources();
        let overlays = self.prepare_overlay_appearance_sources_for_portal_transition(
            transition,
            stage,
            staged_motion,
            retain_exit,
        );
        self.present_prepared_mounted_frame_with_overlay_sources(frame, deadline, now, overlays)
    }

    fn present_prepared_mounted_frame_with_overlay_sources(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        overlays: Result<
            super::active_application_session::UiActiveOverlayAppearancePreparation,
            (),
        >,
    ) -> UiMountedFrameOutcome {
        let generation = self.generation_identity().clone();
        let portal = self.portal.as_ref();
        let motion = self.motion.as_ref();
        let presentation = &self.presentation;
        let capabilities = self.application.capabilities();
        let appearance = self.appearance_owner_snapshot.as_ref();
        let transition = self.mounted.present_prepared_frame_with_overlays(
            &self.host_session,
            frame,
            Some(&mut self.appearance_inspection),
            deadline,
            now,
            |attempt, surfaces| {
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
                )
            },
        );
        self.finish_mounted_transition(transition)
    }

    pub(crate) fn present_prepared_superseding_mounted_frame_internal(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        predecessor: crate::mounting::UiMountedSupersedingPresentationBasis,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> UiMountedFrameOutcome {
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
                |attempt, surfaces| {
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
                    )
                },
            );
        self.finish_mounted_transition(transition)
    }

    pub fn complete_mounted_presentation(
        &mut self,
        in_flight: UiMountedPresentationInFlight,
        now: u64,
    ) -> UiMountedFrameOutcome {
        let transition = self
            .mounted
            .complete_presentation(&self.host_session, in_flight, now);
        self.finish_mounted_transition(transition)
    }

    pub fn cancel_mounted_presentation(
        &mut self,
        in_flight: UiMountedPresentationInFlight,
    ) -> UiMountedFrameOutcome {
        let transition = self
            .mounted
            .cancel_presentation(&self.host_session, in_flight);
        self.finish_mounted_transition(transition)
    }

    pub(crate) fn supersede_mounted_presentation(
        &mut self,
        in_flight: UiMountedPresentationInFlight,
    ) -> UiMountedFrameOutcome {
        let transition = self
            .mounted
            .supersede_presentation(&self.host_session, in_flight);
        self.finish_mounted_transition(transition)
    }

    pub(crate) fn admit_duplicate_native_presentation_observation(
        &mut self,
        presentation: worth_ui_host_native::UiNativePhysicalPresentationCorrelation,
    ) -> Result<(), ()> {
        self.mounted
            .admit_duplicate_native_presentation_observation(presentation)
    }

    pub fn current_mounted_publication(&self) -> Option<&UiMountedFramePublicationReceipt> {
        self.mounted.current_publication()
    }

    pub fn reconcile_mounted_presentation(
        &mut self,
        reconciliation: crate::mounting::UiHostPresentationReconciliation,
    ) -> bool {
        self.mounted.reconcile_presentation(reconciliation)
    }

    pub(super) fn finish_mounted_presentation(
        &mut self,
        outcome: UiMountedPresentationOutcome,
    ) -> UiMountedFrameOutcome {
        let transition = self.mounted.finish_presentation(outcome);
        self.finish_mounted_transition(transition)
    }

    fn finish_mounted_transition(
        &mut self,
        transition: crate::mounting::UiMountedPublicationTransition,
    ) -> UiMountedFrameOutcome {
        let active_generation = self.active_generation_identity();
        let outcome = finish_mounted_transition_with_ports(
            UiMountedPublicationSettlementPorts {
                mounted: &mut self.mounted,
                focus: self.focus.as_mut(),
                portal: self.portal.as_mut(),
                interaction: &mut self.interaction,
                host_session: &self.host_session,
                active_generation,
                host_exchange: &mut self.host_exchange,
            },
            transition,
            Some(&mut self.appearance_inspection),
            Some(&mut self.presentation),
        );
        self.overlay_composition_owners.settle(&outcome);
        if matches!(
            outcome,
            UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Reconciled(_)
        ) {
            self.reconcile_service_state_after_mounted_publication();
        }
        outcome
    }

    pub(super) fn reconcile_prepared_focus_after_published_frame(
        &mut self,
        prepared: crate::runtime::focus::UiPreparedFocusMountedReconciliation,
        publication: &crate::mounting::UiMountedFramePublicationReceipt,
    ) {
        let active_generation = self.active_generation_identity();
        let mut ports = UiMountedPublicationSettlementPorts {
            mounted: &mut self.mounted,
            focus: self.focus.as_mut(),
            portal: self.portal.as_mut(),
            interaction: &mut self.interaction,
            host_session: &self.host_session,
            active_generation,
            host_exchange: &mut self.host_exchange,
        };
        if let Some(portal) = ports.portal.as_deref_mut() {
            rebind_portal_after_published_frame(portal, publication);
        }
        let Some(focus) = ports.focus.as_deref_mut() else {
            return;
        };
        let transition = focus
            .commit_mounted_reconciliation(prepared)
            .expect("prepared Focus reconciliation retains bounded counters")
            .transition();
        place_reconciled_focus(&mut ports, transition, publication);
    }

    pub(super) fn rebind_portal_after_current_published_frame(&mut self) {
        let publication = self
            .mounted
            .current_publication()
            .expect("Portal settlement retains the just-published frame");
        rebind_portal_after_published_frame(
            self.portal
                .as_mut()
                .expect("Portal-specific rebind requires installed Portal support"),
            publication,
        );
    }
}

struct UiMountedPublicationSettlementPorts<'a> {
    mounted: &'a mut crate::mounting::WorthUiMountedSessionState,
    focus: Option<&'a mut crate::runtime::focus::UiFocusRuntimeState>,
    portal: Option<&'a mut crate::runtime::portal::UiPortalRuntimeState>,
    interaction: &'a mut crate::runtime::interaction::UiInteractionRuntimeState,
    host_session: &'a crate::facade::WorthUiHostSessionAuthority,
    active_generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    host_exchange: &'a mut crate::host_exchange::WorthUiHostExchangeSessionState,
}

pub(super) fn finish_mounted_transition(
    mounted: &mut crate::mounting::WorthUiMountedSessionState,
    focus: Option<&mut crate::runtime::focus::UiFocusRuntimeState>,
    portal: Option<&mut crate::runtime::portal::UiPortalRuntimeState>,
    interaction: &mut crate::runtime::interaction::UiInteractionRuntimeState,
    host_session: &crate::facade::WorthUiHostSessionAuthority,
    application_session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    host_exchange: &mut crate::host_exchange::WorthUiHostExchangeSessionState,
    transition: crate::mounting::UiMountedPublicationTransition,
    appearance_inspection: Option<&mut crate::runtime::appearance::UiAppearanceInspectionProducer>,
    appearance_presentation: Option<
        &mut crate::runtime::presentation_state::UiApplicationPresentationState,
    >,
    overlay_composition_owners: Option<
        &mut super::active_application_session::UiActiveOverlayCompositionOwners,
    >,
) -> UiMountedFrameOutcome {
    let active_generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        application_session,
        generation,
    );
    let outcome = finish_mounted_transition_with_ports(
        UiMountedPublicationSettlementPorts {
            mounted,
            focus,
            portal,
            interaction,
            host_session,
            active_generation,
            host_exchange,
        },
        transition,
        appearance_inspection,
        appearance_presentation,
    );
    if let Some(owners) = overlay_composition_owners {
        owners.settle(&outcome);
    }
    outcome
}

fn finish_mounted_transition_with_ports(
    mut ports: UiMountedPublicationSettlementPorts<'_>,
    transition: crate::mounting::UiMountedPublicationTransition,
    mut appearance_inspection: Option<
        &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
    >,
    mut appearance_presentation: Option<
        &mut crate::runtime::presentation_state::UiApplicationPresentationState,
    >,
) -> UiMountedFrameOutcome {
    let (outcome, observation, appearance, hit_transition) = transition.into_parts();
    match &outcome {
        UiMountedFrameOutcome::Published(receipt)
        | UiMountedFrameOutcome::Unchanged(receipt)
        | UiMountedFrameOutcome::Reconciled(receipt) => {
            if let (Some(presentation), Some(publication)) = (
                appearance_presentation.as_deref_mut(),
                ports
                    .mounted
                    .current_text_publication_for_frame(receipt.frame()),
            ) {
                presentation.settle_published_text(publication);
            }
            ports.host_exchange.record_presented_frame(receipt.frame());
            reconcile_focus_after_published_frame_with_ports(&mut ports, receipt);
        }
        _ => {}
    }
    match &outcome {
        UiMountedFrameOutcome::Published(receipt) | UiMountedFrameOutcome::Reconciled(receipt) => {
            if let Some(expected_revision) = ports
                .mounted
                .current_theme_revision_for_frame(receipt.frame())
            {
                if let Some(presentation) = appearance_presentation.as_deref_mut() {
                    presentation.settle_published_theme_values(expected_revision);
                }
            }
        }
        _ => {}
    }
    if let Some(observation) = observation {
        record_mounted_observation(ports.host_exchange, observation);
    }
    if let Some(appearance) = appearance {
        let (invalidation, records) = appearance.into_parts();
        match &outcome {
            UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Reconciled(_) => {
                if let Some(producer) = appearance_inspection.as_deref_mut() {
                    producer.record_frame_attempts(records);
                }
                if let (Some(presentation), Some(invalidation)) = (
                    appearance_presentation.as_deref_mut(),
                    invalidation.as_ref(),
                ) {
                    presentation.settle_appearance_invalidation(invalidation);
                }
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
            | UiMountedFrameOutcome::AdmissionDenied(_) => {
                if let Some(producer) = appearance_inspection.as_deref_mut() {
                    producer.record_pre_effect_denials(records);
                }
            }
            _ => {}
        }
    }
    if let Some(transition) = hit_transition {
        ports
            .interaction
            .observe_presented_hit_transition(&transition, ports.mounted);
    }
    outcome
}

fn rebind_portal_after_published_frame(
    portal: &mut crate::runtime::portal::UiPortalRuntimeState,
    publication: &crate::mounting::UiMountedFramePublicationReceipt,
) {
    if !portal.has_mounted_presentations() {
        return;
    }
    publication.with_surface_presentations(|surfaces| {
        portal.rebind_published_presentations(publication.frame(), surfaces)
    });
}
