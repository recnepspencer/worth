use crate::mounting::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, UiMountedPresentationInFlight,
    UiMountedPresentationOutcome,
};

use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(crate) fn present_prepared_mounted_frame_internal(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> UiMountedFrameOutcome {
        let transition = self.mounted.present_prepared_frame(
            &self.host_session,
            frame,
            Some(&mut self.appearance_inspection),
            deadline,
            now,
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
        let transition = self.mounted.present_prepared_superseding_frame(
            &self.host_session,
            frame,
            predecessor,
            Some(&mut self.appearance_inspection),
            deadline,
            now,
        );
        self.finish_mounted_transition(transition)
    }

    pub fn present_current_mounted_frame_for_reconciliation(
        &mut self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, crate::mounting::UiMountedIdentityDenial> {
        let transition = self.mounted.present_current_for_reconciliation(
            &self.host_session,
            replacements,
            Some(&mut self.appearance_inspection),
            deadline,
            now,
        )?;
        Ok(self.finish_mounted_transition(transition))
    }

    pub(crate) fn present_prepared_mounted_frame_for_reconciliation(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, crate::mounting::UiMountedIdentityDenial> {
        let transition = self.mounted.present_prepared_for_reconciliation(
            &self.host_session,
            frame,
            replacements,
            Some(&mut self.appearance_inspection),
            deadline,
            now,
        )?;
        Ok(self.finish_mounted_transition(transition))
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
) -> UiMountedFrameOutcome {
    let active_generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        application_session,
        generation,
    );
    finish_mounted_transition_with_ports(
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
    )
}

fn finish_mounted_transition_with_ports(
    mut ports: UiMountedPublicationSettlementPorts<'_>,
    transition: crate::mounting::UiMountedPublicationTransition,
    mut appearance_inspection: Option<
        &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
    >,
    appearance_presentation: Option<
        &mut crate::runtime::presentation_state::UiApplicationPresentationState,
    >,
) -> UiMountedFrameOutcome {
    let (outcome, observation, appearance) = transition.into_parts();
    match &outcome {
        UiMountedFrameOutcome::Published(receipt)
        | UiMountedFrameOutcome::Unchanged(receipt)
        | UiMountedFrameOutcome::Reconciled(receipt) => {
            ports.host_exchange.record_presented_frame(receipt.frame());
            reconcile_focus_after_published_frame_with_ports(&mut ports, receipt);
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
                if let (Some(presentation), Some(invalidation)) =
                    (appearance_presentation, invalidation.as_ref())
                {
                    presentation.settle_appearance_invalidation(invalidation);
                }
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
                if let Some(producer) = appearance_inspection.as_deref_mut() {
                    producer.record_pre_effect_denials(records);
                }
            }
            _ => {}
        }
    }
    outcome
}

fn reconcile_focus_after_published_frame_with_ports(
    ports: &mut UiMountedPublicationSettlementPorts<'_>,
    publication: &crate::mounting::UiMountedFramePublicationReceipt,
) {
    if let Some(portal) = ports.portal.as_deref_mut() {
        rebind_portal_after_published_frame(portal, publication);
    }
    let Some(focus) = ports.focus.as_deref_mut() else {
        return;
    };
    let Some(snapshot) = ports.mounted.focus_participation_snapshot() else {
        return;
    };
    let transition = focus
        .reconcile_mounted_participation(&snapshot)
        .expect("mounted participant bounds fit the focus owner counters")
        .transition();
    let Some(transition) = transition else {
        return;
    };
    place_reconciled_focus(ports, Some(transition), publication);
}

fn place_reconciled_focus(
    ports: &mut UiMountedPublicationSettlementPorts<'_>,
    transition: Option<crate::runtime::focus::UiFocusTransitionReceipt>,
    publication: &crate::mounting::UiMountedFramePublicationReceipt,
) {
    let Some(transition) = transition else {
        return;
    };
    super::focus_placement::ports::UiFocusPlacementPorts::new(
        ports.mounted,
        ports
            .focus
            .as_deref_mut()
            .expect("focus placement requires installed Focus support"),
        ports.interaction,
        ports.host_session,
        ports.active_generation.clone(),
    )
    .place(transition, publication)
    .expect("reconciled Focus successor retains exact mounted presentation basis");
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

pub(super) fn record_mounted_observation(
    host_exchange: &mut crate::host_exchange::WorthUiHostExchangeSessionState,
    observation: crate::mounting::UiMountedHostObservationTransition,
) {
    match observation {
        crate::mounting::UiMountedHostObservationTransition::NeverPresented(frame) => {
            host_exchange.record_never_presented_frame(frame);
        }
        crate::mounting::UiMountedHostObservationTransition::Rejected(frame) => {
            host_exchange.record_rejected_frame(frame);
        }
        crate::mounting::UiMountedHostObservationTransition::Indeterminate { frame, bindings } => {
            host_exchange.record_indeterminate_frame(frame, &bindings);
        }
    }
}
