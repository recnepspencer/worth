mod focus_settlement;
use focus_settlement::{place_reconciled_focus, reconcile_focus_after_published_frame_with_ports};
mod observation;
mod prepared;
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

    pub fn complete_mounted_presentation(
        &mut self,
        in_flight: UiMountedPresentationInFlight,
        now: u64,
    ) -> UiMountedFrameOutcome {
        if !self.pending_mounted_owner_receipt_succession_is_current(in_flight.attempt()) {
            let transition = self
                .mounted
                .supersede_presentation(&self.host_session, in_flight);
            return self.finish_mounted_transition(transition, now);
        }
        let transition = self
            .mounted
            .complete_presentation(&self.host_session, in_flight, now);
        self.finish_mounted_transition(transition, now)
    }

    pub fn cancel_mounted_presentation(
        &mut self,
        in_flight: UiMountedPresentationInFlight,
    ) -> UiMountedFrameOutcome {
        let transition = self
            .mounted
            .cancel_presentation(&self.host_session, in_flight);
        self.finish_mounted_transition(transition, u64::MAX)
    }

    pub(crate) fn supersede_mounted_presentation(
        &mut self,
        in_flight: UiMountedPresentationInFlight,
    ) -> UiMountedFrameOutcome {
        let transition = self
            .mounted
            .supersede_presentation(&self.host_session, in_flight);
        self.finish_mounted_transition(transition, u64::MAX)
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
        self.finish_mounted_transition(transition, u64::MAX)
    }

    fn finish_mounted_transition(
        &mut self,
        transition: crate::mounting::UiMountedPublicationTransition,
        now: u64,
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
                expressions: &mut self.expressions,
                application_facts: &self.intent_application_facts,
            },
            transition,
            Some(&mut self.appearance_inspection),
            Some(&mut self.presentation),
        );
        self.overlay_composition_owners.settle(&outcome);
        self.settle_presented_scroll_extent(&outcome, now);
        self.settle_pending_mounted_owner_receipt_succession(&outcome);
        outcome
    }

    pub(in crate::facade::entry) fn settle_presented_scroll_extent(
        &mut self,
        outcome: &UiMountedFrameOutcome,
        now: u64,
    ) {
        super::active_application_session::settle_presented_scroll_extent(
            self.scroll.as_mut(),
            self.motion.as_mut(),
            &mut self.mounted,
            &mut self.interaction,
            &self.owed_scroll_settles,
            outcome,
            now,
        );
        if matches!(
            outcome,
            UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Reconciled(_)
        ) {
            self.reconcile_service_state_after_mounted_publication();
        }
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
            expressions: &mut self.expressions,
            application_facts: &self.intent_application_facts,
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

/// The owners a published mounted frame settles into.
pub(super) struct UiMountedPublicationSettlementPorts<'a> {
    pub(super) mounted: &'a mut crate::mounting::WorthUiMountedSessionState,
    pub(super) focus: Option<&'a mut crate::runtime::focus::UiFocusRuntimeState>,
    pub(super) portal: Option<&'a mut crate::runtime::portal::UiPortalRuntimeState>,
    pub(super) interaction: &'a mut crate::runtime::interaction::UiInteractionRuntimeState,
    pub(super) host_session: &'a crate::facade::WorthUiHostSessionAuthority,
    pub(super) active_generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(super) host_exchange: &'a mut crate::host_exchange::WorthUiHostExchangeSessionState,
    pub(super) expressions: &'a mut crate::runtime::expression::UiExpressionRuntimeState,
    pub(super) application_facts: &'a crate::runtime::intent::UiIntentApplicationFactState,
}

pub(super) fn finish_mounted_transition(
    ports: UiMountedPublicationSettlementPorts<'_>,
    transition: crate::mounting::UiMountedPublicationTransition,
    appearance_inspection: Option<&mut crate::runtime::appearance::UiAppearanceInspectionProducer>,
    appearance_presentation: Option<
        &mut crate::runtime::presentation_state::UiApplicationPresentationState,
    >,
    overlay_composition_owners: Option<
        &mut super::active_application_session::UiActiveOverlayCompositionOwners,
    >,
) -> UiMountedFrameOutcome {
    let outcome = finish_mounted_transition_with_ports(
        ports,
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
    appearance_inspection: Option<&mut crate::runtime::appearance::UiAppearanceInspectionProducer>,
    mut appearance_presentation: Option<
        &mut crate::runtime::presentation_state::UiApplicationPresentationState,
    >,
) -> UiMountedFrameOutcome {
    let (outcome, observation, appearance, hit_transition) = transition.into_parts();
    match &outcome {
        UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Reconciled(_) => ports
            .expressions
            .invalidate_published_frame(&crate::runtime::expression::UiExpressionInputs {
                generation: &ports.active_generation,
                mounted: ports.mounted,
                facts: ports.application_facts,
            }),
        UiMountedFrameOutcome::Unchanged(_)
        | UiMountedFrameOutcome::RejectedBeforeEffects(_)
        | UiMountedFrameOutcome::InFlight(_)
        | UiMountedFrameOutcome::PresentationIndeterminate(_)
        | UiMountedFrameOutcome::Superseded(_)
        | UiMountedFrameOutcome::RetentionDenied(_)
        | UiMountedFrameOutcome::AdmissionDenied(_)
        | UiMountedFrameOutcome::CompletionDenied(_) => {}
    }
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
        UiMountedFrameOutcome::RejectedBeforeEffects(_)
        | UiMountedFrameOutcome::InFlight(_)
        | UiMountedFrameOutcome::PresentationIndeterminate(_)
        | UiMountedFrameOutcome::Superseded(_)
        | UiMountedFrameOutcome::RetentionDenied(_)
        | UiMountedFrameOutcome::AdmissionDenied(_)
        | UiMountedFrameOutcome::CompletionDenied(_) => {}
    }
    if let Some(observation) = observation {
        record_mounted_observation(ports.host_exchange, observation);
    }
    if let Some(appearance) = appearance {
        let (invalidation, records) = appearance.into_parts();
        match &outcome {
            UiMountedFrameOutcome::Published(_)
            | UiMountedFrameOutcome::Unchanged(_)
            | UiMountedFrameOutcome::Reconciled(_) => {
                if let Some(producer) = appearance_inspection {
                    producer.record_frame_attempts(records);
                }
                if let (Some(presentation), Some(invalidation)) =
                    (appearance_presentation, invalidation.as_ref())
                {
                    presentation.settle_appearance_invalidation(invalidation);
                }
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
            | UiMountedFrameOutcome::AdmissionDenied(_) => {
                if let Some(producer) = appearance_inspection {
                    producer.record_pre_effect_denials(records);
                }
            }
            UiMountedFrameOutcome::InFlight(_)
            | UiMountedFrameOutcome::PresentationIndeterminate(_)
            | UiMountedFrameOutcome::Superseded(_)
            | UiMountedFrameOutcome::RetentionDenied(_)
            | UiMountedFrameOutcome::CompletionDenied(_) => {}
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
        portal.rebind_published_presentations(
            publication.frame(),
            surfaces,
            publication.portal_placements(),
        )
    });
}
