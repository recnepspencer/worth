use crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity;
use crate::runtime::{WorthUiFrameworkTurn, WorthUiRuntimeShutdownReceipt};

use super::{
    WorthUiActiveApplicationGenerationIdentity, WorthUiActiveApplicationSessionIdentity,
    WorthUiActiveFrameworkTurnCompletion, WorthUiApp,
};
#[path = "active_application_session/activation.rs"]
mod activation;
#[cfg(test)]
mod appearance_axis_close_tests;
#[cfg(test)]
#[path = "active_application_session/appearance_observation_close_tests.rs"]
mod appearance_observation_close_tests;
#[cfg(test)]
#[path = "active_application_session/appearance_projection_tests.rs"]
mod appearance_projection_tests;
#[cfg(test)]
#[path = "active_application_session/appearance_receipt_distinction_tests.rs"]
mod appearance_receipt_distinction_tests;
#[path = "active_application_session/command_context.rs"]
mod command_context;
#[path = "active_application_session/command_observation.rs"]
mod command_observation;
#[path = "active_application_session/declared_overlay_binding.rs"]
mod declared_overlay_binding;
#[cfg(test)]
#[path = "active_application_session/declared_overlay_binding_tests.rs"]
mod declared_overlay_binding_tests;
#[path = "active_application_session/focus_observation.rs"]
mod focus_observation;
#[path = "active_application_session/host_session_identity.rs"]
mod host_session_identity;
#[path = "active_application_session/motion_sampling.rs"]
mod motion_sampling;
#[path = "active_application_session/overlay_appearance.rs"]
mod overlay_appearance;
pub(in crate::facade::entry) use overlay_appearance::{
    UiActiveOverlayAppearancePreparation, UiActiveOverlayCompositionOwners,
};
#[path = "active_application_session/portal_exit_publication.rs"]
mod portal_exit_publication;
pub(in crate::facade::entry) use portal_exit_publication::UiPortalExitTerminalProgress;
pub(in crate::facade::entry) use portal_exit_retention::{
    UiPortalExitTerminalPending, UiPortalExitTerminalPendingKind,
};
#[cfg(any(test, feature = "certification-support"))]
#[path = "active_application_session/plan_observation.rs"]
mod plan_observation;
#[path = "active_application_session/pointer_inspection.rs"]
mod pointer_inspection;
#[path = "active_application_session/portal_exit_retention.rs"]
mod portal_exit_retention;
#[path = "active_application_session/portal_motion.rs"]
mod portal_motion;
#[path = "active_application_session/portal_observation.rs"]
mod portal_observation;
#[path = "active_application_session/runtime_access.rs"]
mod runtime_access;
#[path = "active_application_session/scroll_geometry.rs"]
mod scroll_geometry;
#[path = "active_application_session/scroll_observation.rs"]
mod scroll_observation;
#[path = "active_application_session/semantic_text_registration.rs"]
mod semantic_text_registration;
#[path = "active_application_session/service_inspection.rs"]
mod service_inspection;
#[path = "active_application_session/service_installation_observation.rs"]
mod service_installation_observation;
#[path = "active_application_session/service_proposal_observation.rs"]
mod service_proposal_observation;
#[path = "active_application_session/service_state_observation.rs"]
mod service_state_observation;
#[path = "active_application_session/service_state_reconciliation.rs"]
mod service_state_reconciliation;
mod shutdown;
/// The one ordinary owner of a running Worth UI application generation.
pub struct WorthUiActiveApplicationSession {
    pub(super) identity: WorthUiActiveApplicationSessionIdentity,
    pub(super) application: crate::runtime::session::WorthUiApplicationSessionState,
    pub(super) host_session: crate::facade::WorthUiHostSessionAuthority,
    pub(super) mounted: crate::mounting::WorthUiMountedSessionState,
    pub(super) host_exchange: crate::host_exchange::WorthUiHostExchangeSessionState,
    pub(super) interaction: crate::runtime::interaction::UiInteractionRuntimeState,
    pub(super) focus:
        crate::runtime::UiRuntimeServiceInstallation<crate::runtime::focus::UiFocusRuntimeState>,
    pub(super) portal:
        crate::runtime::UiRuntimeServiceInstallation<crate::runtime::portal::UiPortalRuntimeState>,
    pub(super) authored_overlay_bindings: crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    pub(super) overlay_composition_owners: UiActiveOverlayCompositionOwners,
    pub(super) dormant_portal_stack_ordinal_issuer:
        Option<crate::runtime::portal::UiPortalStackOrdinalIssuer>,
    pub(super) motion:
        crate::runtime::UiRuntimeServiceInstallation<crate::runtime::motion::UiMotionRuntimeState>,
    pub(super) scroll:
        crate::runtime::UiRuntimeServiceInstallation<crate::runtime::scroll::UiScrollRuntimeState>,
    pub(super) selection: crate::runtime::UiRuntimeServiceInstallation<
        crate::runtime::selection::UiSelectionRuntimeState,
    >,
    pub(super) command_routing: crate::runtime::UiRuntimeServiceInstallation<
        crate::runtime::command_routing::UiCommandRoutingRuntimeState,
    >,
    pub(super) ime_composing: bool,
    pub(super) portal_exit_retention: portal_exit_retention::UiPortalExitRetentionCoordinator,
    pub(super) intent_evidence: crate::inspection::intent::UiIntentEvidenceRegistry,
    pub(super) intent_application_facts: crate::runtime::intent::UiIntentApplicationFactState,
    pub(super) intent_execution: crate::runtime::intent_execution::UiIntentExecutionState,
    pub(super) intent_admission: crate::runtime::intent::UiIntentAdmissionState,
    pub(super) intent_confirmation: crate::runtime::intent::UiIntentConfirmationState,
    pub(super) intent_postures: crate::mounting::UiIntentPostureTable,
    pub(super) presentation: crate::runtime::presentation_state::UiApplicationPresentationState,
    pub(super) appearance_theme_admission:
        Option<crate::runtime::appearance::UiPreparedThemeBindingAdmission>,
    pub(super) appearance_inspection: crate::runtime::appearance::UiAppearanceInspectionProducer,
    pub(super) observation_clock: Option<worth_ui_host_native::UiNativeObservationClock>,
    pub(super) pointer_affordance_snapshot:
        Option<crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
    pub(super) appearance_owner_snapshot:
        Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    pub(super) mounted_owner_receipt_successions:
        super::mounted_owner_receipt_succession::UiMountedOwnerReceiptSuccessionCoordinator,
    pub(super) visual_inspection:
        crate::inspection::visual_snapshot::WorthUiVisualInspectionAuthority,
    pub(super) next_visual_capture_identity: u64,
    pub(super) next_visual_overlay_identity: u64,
    pub(super) next_portal_service_event_identity: u64,
    pub(super) visual_captures: crate::inspection::visual_snapshot::UiVisualCaptureRegistry,
    pub(super) visual_overlays: crate::inspection::visual_snapshot::UiVisualOverlayRegistry,
    pub(super) rebind: crate::runtime::rebind::UiRebindRuntimeState,
}

impl WorthUiActiveApplicationSession {
    pub fn session_identity(&self) -> WorthUiActiveApplicationSessionIdentity {
        self.identity
    }

    pub(super) fn scroll_owner_incarnation(
        &self,
    ) -> crate::runtime::scroll::UiScrollOwnerIncarnation {
        crate::runtime::scroll::UiScrollOwnerIncarnation::new(self.identity.as_u64())
            .expect("active session identity is nonzero")
    }

    pub fn generation_identity(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        self.application.generation_identity()
    }

    pub fn active_generation_identity(&self) -> WorthUiActiveApplicationGenerationIdentity {
        WorthUiActiveApplicationGenerationIdentity::current(
            self.identity,
            self.application.generation_identity(),
        )
    }

    pub const fn rebind_deadline_at(
        &self,
        tick: u64,
    ) -> crate::runtime::rebind::UiRebindSessionDeadline {
        crate::runtime::rebind::UiRebindSessionDeadline::new(self.identity, tick)
    }

    pub const fn rebind_cancellation_request(
        &self,
    ) -> crate::runtime::rebind::UiRebindCancellationRequest {
        crate::runtime::rebind::UiRebindCancellationRequest::new(self.identity)
    }

    pub fn capabilities(&self) -> &crate::facade::registry::snapshot::CapabilitySnapshot {
        self.application.capabilities()
    }

    #[cfg(test)]
    pub(crate) const fn has_appearance_owner_snapshot_for_test(&self) -> bool {
        self.appearance_owner_snapshot.is_some()
    }

    #[cfg(test)]
    pub(crate) const fn appearance_owner_snapshot_for_test(
        &self,
    ) -> Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot> {
        self.appearance_owner_snapshot.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn replace_appearance_theme_definition_for_test(
        &mut self,
        definition: &str,
        changed_token: &crate::capability::ThemeTokenId,
    ) {
        let receipts = {
            let themes = self
                .capabilities()
                .appearance_themes()
                .expect("appearance test session must carry a theme bundle");
            let identity = crate::capability::UiThemeDefinitionIdentity::new(definition).unwrap();
            let roles = self.capabilities().appearance_roles();
            self.presentation
                .appearance_theme_state()
                .expect("appearance test session must have active theme bindings")
                .active_bindings()
                .map(|binding| {
                    crate::runtime::appearance::UiThemeCapabilityAdmission::
                        from_frozen_capabilities(
                            themes,
                            &identity,
                            roles,
                            binding.capability().host_profile(),
                        )
                        .unwrap()
                        .issue(
                            binding
                                .capability()
                                .required_roles()
                                .iter()
                                .map(|role| role.identity().clone()),
                            binding.surface(),
                            self.active_generation_identity(),
                        )
                        .unwrap()
                })
                .collect::<Vec<_>>()
        };
        let authority = self.application.prepared_authority();
        let index = authority.consumed_fact_index();
        let declarations = authority.authored_declaration_lookup();
        let authored = declarations
            .theme_token_declaration_identity(changed_token.as_str())
            .unwrap_or(changed_token.as_str());
        let selected = index
            .select_appearance_slot_consumers(index.basis(), changed_token.as_str(), authored)
            .expect("test theme switch selects declared slot consumers");
        for receipt in receipts {
            let (batch, _) =
                crate::runtime::appearance::UiAppearanceInvalidationBatch::theme_surface(
                    index,
                    &self.mounted,
                    self.graph(),
                    receipt.surface(),
                    selected.consumers(),
                )
                .expect("test theme switch selects only the bound surface");
            self.presentation
                .replace_appearance_theme_binding_for_test(receipt);
            self.presentation
                .queue_appearance_invalidation(batch)
                .expect("test theme switch queues its surface invalidation");
        }
    }

    pub fn resolve_affected_scope(
        &self,
        change: crate::facade::observation::UiClassifiedChange,
    ) -> Result<
        crate::runtime::rebind::UiResolvedAffectedScope,
        crate::runtime::rebind::UiAffectedScopeDenial,
    > {
        self.application
            .resolve_affected_scope(self.identity, change)
    }

    pub fn compile_rebind_plan(
        &self,
        lifecycle: crate::runtime::rebind::UiResolvedIdentityLifecycle,
        policy: crate::runtime::rebind::UiRebindExecutionPolicy,
    ) -> Result<crate::runtime::rebind::UiRebindPlan, crate::runtime::rebind::UiRebindPlanningDenial>
    {
        self.application
            .compile_rebind_plan(self.identity, lifecycle, policy)
    }

    pub fn compile_preservation_rebind(
        &self,
        evidence: crate::facade::observation::UiEvidenceOnlySourceChange,
        policy: crate::runtime::rebind::UiRebindExecutionPolicy,
    ) -> Result<crate::runtime::rebind::UiRebindPlan, crate::runtime::rebind::UiRebindPlanningDenial>
    {
        self.application
            .compile_preservation_rebind(self.identity, evidence, policy)
    }

    /// Borrow the graph authority for the generation this session is
    /// currently executing.
    pub fn graph(&self) -> crate::graph::UiGraphAuthority<'_> {
        self.application.graph()
    }

    pub(crate) fn execute_framework_turn(
        &mut self,
        collect_sources: impl FnOnce(&mut WorthUiFrameworkTurn<'_>),
    ) -> Result<
        WorthUiActiveFrameworkTurnCompletion<'_>,
        crate::mounting::UiMountedPublicationLeaseDenial,
    > {
        if self.mounted.has_active_presentation_attempt() {
            return Err(crate::mounting::UiMountedPublicationLeaseDenial::PresentationInFlight);
        }
        let host_session_identity = self.host_session.identity();
        let font_collection = std::sync::Arc::clone(self.application.font_collection());
        let overlay_appearance = self.prepare_overlay_appearance_sources();
        let motion = self.motion.as_ref();
        let turn = self.application.execute_framework_turn(collect_sources);
        let (
            generation_identity,
            visual_trace_source,
            graph,
            active_plan_digest,
            completion,
            capabilities,
            intent_catalog,
            consumed_facts,
        ) = turn.into_parts();
        if self
            .presentation
            .appearance_invalidation_batch()
            .is_some_and(|pending| pending.basis() != consumed_facts.basis())
        {
            // Framework measurements may advance the dependency index before
            // initial publication. Recompute pending work on that exact index.
            self.presentation
                .queue_appearance_invalidation(
                    crate::runtime::appearance::UiAppearanceInvalidationBatch::initial(
                        consumed_facts,
                    ),
                )
                .expect("appearance invalidation revision remains available");
        }
        Ok(WorthUiActiveFrameworkTurnCompletion {
            application_session_identity: self.identity,
            generation_identity,
            visual_trace_source,
            graph,
            font_collection,
            active_plan_digest,
            host_session_identity,
            completion,
            capabilities,
            intent_catalog,
            consumed_facts,
            mounted: &mut self.mounted,
            host_session: &self.host_session,
            host_exchange: &mut self.host_exchange,
            focus: self.focus.as_mut(),
            selection: self.selection.as_mut(),
            portal: self.portal.as_mut(),
            overlay_composition_owners: &mut self.overlay_composition_owners,
            interaction: &mut self.interaction,
            presentation: &mut self.presentation,
            appearance_owner_snapshot: &mut self.appearance_owner_snapshot,
            intent_admission: &mut self.intent_admission,
            intent_application_facts: &mut self.intent_application_facts,
            mounted_owner_receipt_successions: &mut self.mounted_owner_receipt_successions,
            pointer_affordance_snapshot: &self.pointer_affordance_snapshot,
            appearance_inspection: &mut self.appearance_inspection,
            overlay_appearance,
            motion,
        })
    }
}
