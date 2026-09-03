use crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity;
use crate::runtime::{WorthUiFrameworkTurn, WorthUiRuntime, WorthUiRuntimeShutdownReceipt};

use super::{
    WorthUiActiveApplicationGenerationIdentity, WorthUiActiveApplicationSessionIdentity,
    WorthUiActiveFrameworkTurnCompletion, WorthUiApp,
};
#[cfg(test)]
mod appearance_axis_close_tests;
#[cfg(test)]
#[path = "active_application_session/appearance_observation_close_tests.rs"]
mod appearance_observation_close_tests;
#[cfg(test)]
#[path = "active_application_session/appearance_projection_tests.rs"]
mod appearance_projection_tests;
#[path = "active_application_session/command_context.rs"]
mod command_context;
#[path = "active_application_session/command_observation.rs"]
mod command_observation;
#[path = "active_application_session/focus_observation.rs"]
mod focus_observation;
#[path = "active_application_session/host_session_identity.rs"]
mod host_session_identity;
#[path = "active_application_session/motion_sampling.rs"]
mod motion_sampling;
#[path = "active_application_session/portal_exit_publication.rs"]
mod portal_exit_publication;
pub(in crate::facade::entry) use portal_exit_publication::UiPortalExitTerminalProgress;
pub(in crate::facade::entry) use portal_exit_retention::{
    UiPortalExitTerminalPending, UiPortalExitTerminalPendingKind,
};
#[cfg(any(test, feature = "certification-support"))]
#[path = "active_application_session/plan_observation.rs"]
mod plan_observation;
#[path = "active_application_session/portal_exit_retention.rs"]
mod portal_exit_retention;
#[path = "active_application_session/portal_motion.rs"]
mod portal_motion;
#[path = "active_application_session/portal_observation.rs"]
mod portal_observation;
#[path = "active_application_session/runtime_access.rs"]
mod runtime_access;
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
#[path = "active_application_session/theme_values.rs"]
mod theme_values;
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
    pub(super) appearance_inspection: crate::runtime::appearance::UiAppearanceInspectionProducer,
    pub(super) appearance_projection_transitions:
        crate::runtime::appearance::UiAppearanceProjectionTransitionState,
    pub(super) appearance_owner_snapshot:
        Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
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
    pub(super) fn new(
        mut app: WorthUiApp,
        runtime: WorthUiRuntime,
        host_session: crate::facade::WorthUiHostSessionAuthority,
    ) -> Result<Self, crate::runtime::WorthUiRuntimeLaunchDenial> {
        let identity = WorthUiActiveApplicationSessionIdentity::from_host_session_value(
            host_session.identity().as_u64(),
        );
        let mounted_frame_retention_budget = app.mounted_frame_retention_budget();
        let host_observation_capacity = app.host_observation_capacity();
        let visual_policy = app.visual_inspection_policy();
        let rebind_profile = app.prepared_authority().change_profile().rebind();
        let presentation_async = app.take_presentation_async_owner();
        let presentation =
            crate::runtime::presentation_state::UiApplicationPresentationState::activate(
                app.capabilities(),
            );
        let service_policy_plan = app.service_policy_plan();
        let mut dormant_portal_stack_ordinal_issuer =
            Some(crate::runtime::portal::UiPortalStackOrdinalIssuer::new());
        let appearance_axis_demand = app
            .prepared_authority()
            .consumed_fact_index()
            .appearance_axis_demand();
        let intent_application_facts =
            crate::runtime::intent::UiIntentApplicationFactState::activate(
                app.prepared_authority().intent_application_fact_plan(),
                appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Validation),
            );
        if appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Focus)
            && service_policy_plan.focus().is_none()
        {
            return Err(
                crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceOwnerUnavailable(
                    worth_ui_dsl::UiAppearanceStateAxis::Focus,
                ),
            );
        }
        if appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Selection)
            && service_policy_plan.selection().is_none()
        {
            return Err(
                crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceOwnerUnavailable(
                    worth_ui_dsl::UiAppearanceStateAxis::Selection,
                ),
            );
        }
        let command_routing = crate::runtime::UiRuntimeServiceInstallation::from_optional(
            service_policy_plan.command_routing().map(|policy| {
                crate::runtime::command_routing::UiCommandRoutingRuntimeState::new(
                    crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
                    app.capabilities().commands(),
                    policy,
                )
            }),
        );
        let application =
            crate::runtime::session::WorthUiApplicationSessionState::new(app, runtime);
        let pointer_presence_enabled =
            appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Hover);
        let pressed_appearance_enabled =
            appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Pressed);
        let mounted = crate::mounting::WorthUiMountedSessionState::new(
            host_session.identity(),
            mounted_frame_retention_budget,
            presentation_async,
        )
        .map_err(|_| crate::runtime::WorthUiRuntimeLaunchDenial::MountedIdentityExhausted)?;
        let visual_inspection =
            crate::inspection::visual_snapshot::WorthUiVisualInspectionAuthority::seal(
                identity,
                visual_policy,
            );
        Ok(Self {
            identity,
            application,
            host_session,
            mounted,
            host_exchange: crate::host_exchange::WorthUiHostExchangeSessionState::new(
                host_observation_capacity,
            ),
            interaction: crate::runtime::interaction::UiInteractionRuntimeState::new(
                pointer_presence_enabled,
                pressed_appearance_enabled,
                crate::runtime::interaction::UiPointerPresenceCapacity::from_host_observation(
                    host_observation_capacity,
                ),
            ),
            focus: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.focus().map(|policy| {
                    let restoration = service_policy_plan
                        .portal()
                        .is_none_or(crate::declaration::UiPortalPolicy::restores_focus);
                    crate::runtime::focus::UiFocusRuntimeState::new_session_restore_candidate_with_policy(
                        policy.with_scope_restoration(
                            policy.restores_on_scope_close() && restoration,
                        ),
                    )
                }),
            ),
            portal: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.portal().map(|policy| {
                    crate::runtime::portal::UiPortalRuntimeState::new_with_policy_and_ordinal_issuer(
                        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
                        policy,
                        dormant_portal_stack_ordinal_issuer
                            .take()
                            .expect("active session retains one Portal ordinal issuer"),
                    )
                }),
            ),
            dormant_portal_stack_ordinal_issuer,
            motion: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.motion().map(|policy| {
                    crate::runtime::motion::UiMotionRuntimeState::new_with_policy(
                        crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
                        policy,
                    )
                }),
            ),
            scroll: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.scroll().map(
                    crate::runtime::scroll::UiScrollRuntimeState::new_session_restore_candidate_with_policy,
                ),
            ),
            selection: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.selection().map(
                    crate::runtime::selection::UiSelectionRuntimeState::new_session_restore_candidate_with_policy,
                ),
            ),
            command_routing,
            ime_composing: false,
            portal_exit_retention: portal_exit_retention::UiPortalExitRetentionCoordinator::new(),
            intent_evidence: crate::inspection::intent::UiIntentEvidenceRegistry::new(
                identity.as_u64(),
            ),
            intent_application_facts,
            intent_execution: crate::runtime::intent_execution::UiIntentExecutionState::new(),
            intent_admission: crate::runtime::intent::UiIntentAdmissionState::new(
                appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Operability),
            ),
            intent_confirmation: crate::runtime::intent::UiIntentConfirmationState::new(),
            intent_postures: crate::mounting::UiIntentPostureTable::new(),
            presentation,
            appearance_inspection: crate::runtime::appearance::UiAppearanceInspectionProducer::new(),
            appearance_projection_transitions:
                crate::runtime::appearance::UiAppearanceProjectionTransitionState::default(),
            appearance_owner_snapshot: None,
            visual_inspection,
            next_visual_capture_identity: 1,
            next_visual_overlay_identity: 1,
            next_portal_service_event_identity: 1_u64 << 63,
            visual_captures: crate::inspection::visual_snapshot::UiVisualCaptureRegistry::new(
                visual_policy,
            ),
            visual_overlays: crate::inspection::visual_snapshot::UiVisualOverlayRegistry::new(),
            rebind: crate::runtime::rebind::UiRebindRuntimeState::new(rebind_profile),
        })
    }

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
        let turn = self.application.execute_framework_turn(collect_sources);
        let (
            generation_identity,
            visual_trace_source,
            graph,
            active_plan_digest,
            completion,
            capabilities,
        ) = turn.into_parts();
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
            mounted: &mut self.mounted,
            host_session: &self.host_session,
            host_exchange: &mut self.host_exchange,
            focus: self.focus.as_mut(),
            portal: self.portal.as_mut(),
            interaction: &mut self.interaction,
            presentation: &mut self.presentation,
            appearance_owner_snapshot: &self.appearance_owner_snapshot,
            appearance_projection_transitions: &mut self.appearance_projection_transitions,
            appearance_inspection: &mut self.appearance_inspection,
        })
    }
}
