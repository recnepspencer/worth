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
    pub(super) appearance_theme_admission:
        Option<crate::runtime::appearance::UiPreparedThemeBindingAdmission>,
    pub(super) appearance_inspection: crate::runtime::appearance::UiAppearanceInspectionProducer,
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
            appearance_inspection: &mut self.appearance_inspection,
        })
    }
}
