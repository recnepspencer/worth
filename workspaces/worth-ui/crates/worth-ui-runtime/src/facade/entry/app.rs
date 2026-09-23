use worth_ui_inspection::{UiInspectionQuery, UiInspectionScope, UiInspectionSupportReport};

use crate::admission::UiAdmissionBoundary;
use crate::declaration::{
    UiDeclarationArtifact, UiDeclarationAuthoredEvidenceIndex, UiDeclarationCloseoutReport,
};
use crate::evidence::{UiEvidenceExpansion, UiEvidenceRef};
#[cfg(any(test, feature = "certification-support"))]
use crate::facade::runtime_handoff::WorthUiRuntimeLaunch;
use crate::facade::{
    inspection::expand_evidence_ref as expand_inspection_evidence_ref,
    inspection_bridge::{
        route_inspection, UiInspectionClosureReport, UiInspectionFacadeObservation,
        UiInspectionReceipt,
    },
    lifecycle::WorthUiFacadeLifecycleBootstrap,
    prepared_application_authority::{
        WorthUiPreparedApplicationAuthority, WorthUiPreparedApplicationGenerationIdentity,
    },
    registry::snapshot::CapabilitySnapshot,
    retained_obligation_registry::WorthUiRetainedObligationRegistry,
    runtime_handoff::{WorthUiRuntime, WorthUiRuntimeLaunchDenial},
};
use crate::graph::{
    UiGraphAspectEvidenceIndexes, UiGraphAuthority, UiGraphCloseoutReport,
    UiGraphNodeEvidenceIndex, UiGraphSnapshot,
};
use crate::lifecycle::WorthUiRuntimeSupportInventory;
use crate::obligations::closeout::UiObligationCloseoutReport;
use crate::runtime::WorthUiRetainedAllocationPlanningEvidenceRegistry;
use std::rc::Rc;

#[path = "app_presentation_async.rs"]
mod presentation_async;
#[path = "app_required_appearance_roles.rs"]
mod required_appearance_roles;

/// Runtime facade entrypoint for building Worth UI applications.
pub struct WorthUi {
    _sealed: (),
}

impl WorthUi {
    /// Start a Worth UI application definition.
    pub fn app() -> crate::facade::entry::WorthUiApplicationBuilder<
        crate::facade::entry::UiChangeProfileMissing,
        crate::facade::entry::UiIntentWiringSatisfied,
    > {
        crate::facade::entry::WorthUiApplicationBuilder::new()
    }
}

/// Worth UI application after capability registration has frozen.
pub struct WorthUiApp {
    prepared: WorthUiPreparedApplicationAuthority,
    host_session_plan: crate::facade::prepared_application_authority::WorthUiHostSessionPlan,
    retained_obligations: WorthUiRetainedObligationRegistry,
    retained_allocation_planning_evidence: Rc<WorthUiRetainedAllocationPlanningEvidenceRegistry>,
    font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    presentation_async:
        Option<crate::native_platform::text_presentation::UiPresentationAsyncRuntime>,
}

impl WorthUiApp {
    pub(crate) fn from_prepared_authority(
        prepared: WorthUiPreparedApplicationAuthority,
        host_session_plan: crate::facade::prepared_application_authority::WorthUiHostSessionPlan,
        font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    ) -> Self {
        Self {
            prepared,
            host_session_plan,
            retained_obligations: WorthUiRetainedObligationRegistry::default(),
            retained_allocation_planning_evidence: Rc::default(),
            font_collection,
            presentation_async: None,
        }
    }

    /// Inspect the comparison-safe identity of this prepared generation.
    pub fn generation_identity(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        self.prepared.generation_identity()
    }

    pub fn font_collection(&self) -> &std::sync::Arc<worth_ui_text::UiGlobalFontCollection> {
        &self.font_collection
    }

    pub(super) fn mounted_frame_retention_budget(
        &self,
    ) -> crate::mounting::UiMountedFrameRetentionBudget {
        self.host_session_plan.mounted_frame_retention_budget()
    }

    pub(super) fn host_observation_capacity(
        &self,
    ) -> crate::host_exchange::observation_report_validation::UiHostObservationCapacity {
        self.host_session_plan.host_observation_capacity()
    }

    pub(super) const fn visual_inspection_policy(
        &self,
    ) -> worth_ui_inspection::UiVisualInspectionPolicy {
        self.prepared.visual_inspection_policy()
    }

    /// Borrow the sealed prepared authority without transferring any
    /// independently launchable constituent.
    pub(crate) fn prepared_authority(&self) -> &WorthUiPreparedApplicationAuthority {
        &self.prepared
    }

    pub(crate) fn motion_support_installed(&self) -> bool {
        self.prepared.service_policy_plan().motion().is_some()
    }

    pub(crate) const fn runtime_service_support(
        &self,
    ) -> crate::capability::UiRuntimeServiceSupport {
        self.prepared
            .service_policy_plan()
            .runtime_service_support()
    }

    pub(crate) fn host_session_plan(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiHostSessionPlan {
        &self.host_session_plan
    }

    pub(crate) fn visual_trace_source(
        &self,
    ) -> crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource {
        self.prepared.visual_trace_source()
    }

    /// Inspect the immutable capability snapshot owned by this app.
    pub fn capabilities(&self) -> &CapabilitySnapshot {
        self.prepared.capabilities()
    }

    pub const fn service_policy_plan(&self) -> crate::declaration::UiNormalizedServicePolicyPlan {
        self.prepared.service_policy_plan()
    }

    /// Resolve the compact installed-operation reference retained for one
    /// registered Query view. The returned reference can enter Query only
    /// through the attempt-scoped operating-world gateway.
    pub fn resolve_query_view(
        &self,
        identity: &worth_ui_query_binding::WorthUiQueryViewIdentity,
        shape: worth_ui_query_binding::WorthUiQueryViewShape,
    ) -> Option<worth_ui_query_binding::WorthUiInstalledQueryBindingReference> {
        self.prepared
            .query_binding_plan()
            .resolve_definition(identity, shape)
    }

    /// Inspect the canonical declaration artifacts admitted during app freeze.
    pub fn declaration_artifacts(&self) -> &[UiDeclarationArtifact] {
        self.prepared.declaration_artifacts()
    }

    /// Inspect the proof-bearing graph authority surface owned by this app.
    pub fn graph(&self) -> UiGraphAuthority<'_> {
        UiGraphAuthority::new(self.prepared.graph_snapshot())
    }

    /// Enter the runtime-owned admission boundary through one formal facade lane.
    pub(crate) fn admission(&self) -> UiAdmissionBoundary<'_> {
        UiAdmissionBoundary::new(
            self.prepared.declaration_artifacts(),
            self.prepared.graph_snapshot(),
        )
    }

    pub(crate) fn graph_snapshot(&self) -> &UiGraphSnapshot {
        self.prepared.graph_snapshot()
    }

    pub(crate) fn advance_prepared_graph(
        &mut self,
        committed: crate::graph::UiGraphMutationCommitResult,
    ) {
        self.prepared.advance_graph_snapshot(committed);
    }

    pub(crate) fn commit_evidence_only_prepared_authority(
        &mut self,
        successor: WorthUiPreparedApplicationAuthority,
    ) -> (
        WorthUiPreparedApplicationGenerationIdentity,
        WorthUiPreparedApplicationGenerationIdentity,
    ) {
        let prior = self.prepared.generation_identity().clone();
        let active = successor.generation_identity().clone();
        self.prepared = successor;
        (prior, active)
    }

    pub(crate) fn prepare_graph_successor(
        &self,
        committed: crate::graph::UiGraphMutationCommitResult,
    ) -> Result<
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGraphSuccessor,
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGraphSuccessorDenial,
    >{
        self.prepared.prepare_graph_successor(committed)
    }

    pub(crate) fn commit_graph_successor(
        &mut self,
        successor: crate::facade::prepared_application_authority::WorthUiPreparedApplicationGraphSuccessor,
    ) -> Result<
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationLoweringAuthority,
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGraphSuccessorDenial,
    >{
        self.prepared.commit_graph_successor(successor)
    }

    pub(crate) fn lifecycle(&self) -> &WorthUiFacadeLifecycleBootstrap {
        self.prepared.lifecycle()
    }

    pub(crate) fn authored_evidence_index(&self) -> &UiDeclarationAuthoredEvidenceIndex {
        self.prepared.authored_evidence_index()
    }

    pub(crate) fn graph_node_evidence_index(&self) -> &UiGraphNodeEvidenceIndex {
        self.prepared.graph_node_evidence_index()
    }

    pub(crate) fn graph_aspect_evidence_indexes(&self) -> &UiGraphAspectEvidenceIndexes {
        self.prepared.graph_aspect_evidence_indexes()
    }

    pub(crate) fn measurement_inspection_evidence(
        &self,
    ) -> &crate::facade::measurement_inspection_evidence::UiMeasurementInspectionEvidenceSnapshot
    {
        self.prepared.lifecycle().measurement_inspection_evidence()
    }

    pub fn graph_closeout_report(&self) -> UiGraphCloseoutReport {
        UiGraphCloseoutReport::milestone33()
    }

    /// Inspect milestone-closeout metadata owned by the declaration boundary.
    pub fn declaration_closeout_report(&self) -> UiDeclarationCloseoutReport {
        UiDeclarationCloseoutReport::milestone32()
    }

    pub fn obligation_closeout_report(&self) -> UiObligationCloseoutReport {
        UiObligationCloseoutReport::milestone34()
    }

    /// Enter the runtime-owned inspection surface through one formal facade lane.
    pub fn inspect(&self, query: UiInspectionQuery) -> UiInspectionReceipt {
        route_inspection(self, query)
    }

    pub fn expand_evidence_ref(
        &self,
        evidence_ref: UiEvidenceRef,
        requested_richness: worth_ui_inspection::UiEvidenceRichness,
    ) -> UiEvidenceExpansion {
        expand_inspection_evidence_ref(self, evidence_ref, requested_richness)
    }

    pub fn discard_evidence_slice(&self, slice_ref: crate::evidence::UiEvidenceSliceRef) -> bool {
        self.retained_obligations.discard_slice(slice_ref)
            || self
                .retained_allocation_planning_registry()
                .discard_slice(slice_ref)
    }

    pub fn inspection_support_report(&self, scope: UiInspectionScope) -> UiInspectionSupportReport {
        self.prepared.lifecycle().inspection_support_report(scope)
    }

    pub fn inspection_closure_report(&self) -> UiInspectionClosureReport {
        self.prepared.lifecycle().inspection_closure_report()
    }

    pub fn runtime_support_inventory(&self) -> &WorthUiRuntimeSupportInventory {
        self.prepared.lifecycle().runtime_support_inventory()
    }

    pub fn inspection_observation(&self) -> UiInspectionFacadeObservation {
        self.prepared.lifecycle().inspection_observation()
    }

    #[cfg(test)]
    pub(crate) fn rebuild_prepared_derived_indexes(&mut self) {
        self.prepared.rebuild_derived_indexes();
    }

    /// Launch the runtime whose ordinary frame boundary owns dispatcher close/pump.
    pub fn launch(
        self,
    ) -> Result<crate::facade::entry::WorthUiActiveApplicationSession, WorthUiRuntimeLaunchDenial>
    {
        self.launch_with_diagnostics(crate::runtime::WorthUiRuntimeDiagnosticPolicy::minimal())
    }

    pub(crate) fn launch_with_diagnostics(
        self,
        diagnostic_policy: crate::runtime::WorthUiRuntimeDiagnosticPolicy,
    ) -> Result<crate::facade::entry::WorthUiActiveApplicationSession, WorthUiRuntimeLaunchDenial>
    {
        let admission = self
            .prepared
            .admit_launch(self.host_session_plan.clone(), diagnostic_policy)?;
        let (runtime, host_session) = WorthUiRuntime::launch_prepared(
            admission,
            Rc::clone(&self.retained_allocation_planning_evidence),
        )?;
        crate::facade::entry::WorthUiActiveApplicationSession::new(self, runtime, host_session)
    }

    /// Certification-only launch seam for subsystem tests that construct a
    /// deliberately synthetic artifact. Ordinary callers cannot access it.
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn launch_runtime(
        &self,
        launch: WorthUiRuntimeLaunch,
    ) -> Result<WorthUiRuntime, WorthUiRuntimeLaunchDenial> {
        let artifact_digest = launch.candidate_artifact_digest.unwrap_or_else(|| {
            crate::source::WorthUiArtifactDigestor::digest(
                launch.artifact.as_ref(),
                crate::source::WorthUiArtifactEquivalenceBasis::semantic(),
            )
        });
        let lowering_authority = self
            .prepared
            .lowering_authority()
            .synthetic_launch_for_certification(Rc::clone(&launch.artifact), artifact_digest);
        let initial_allocation_commit = self.prepared.initial_allocation_commit(artifact_digest)?;
        let host_session = crate::facade::WorthUiHostSessionAuthority::activate(
            &self.host_session_plan,
        )
        .map_err(|denial| match denial {
            crate::facade::WorthUiHostSessionActivationDenial::IdentityExhausted => {
                WorthUiRuntimeLaunchDenial::HostSessionIdentityExhausted
            }
            crate::facade::WorthUiHostSessionActivationDenial::Protocol(denial) => {
                WorthUiRuntimeLaunchDenial::HostProtocol(denial)
            }
            crate::facade::WorthUiHostSessionActivationDenial::MountedPresentationLease(_) => {
                WorthUiRuntimeLaunchDenial::HostMountedPresentationLease
            }
            crate::facade::WorthUiHostSessionActivationDenial::ObservationSession(denial) => {
                WorthUiRuntimeLaunchDenial::HostObservationSession(denial)
            }
        })?;
        let runtime = WorthUiRuntime::launch(
            launch,
            crate::runtime::WorthUiRuntimeLaunchAuthority {
                lowering_authority,
                initial_allocation_commit,
                snapshot_digest: self.prepared.capabilities().digest(),
                retained_allocation_planning_evidence: Rc::clone(
                    &self.retained_allocation_planning_evidence,
                ),
                query_binding: self
                    .prepared
                    .query_binding_plan()
                    .prepare_downstream_state(),
                host_plan_binding: host_session.plan_binding(),
                change_profile: self.prepared.change_profile(),
            },
        )?;
        Ok(runtime)
    }

    pub(crate) fn retained_obligation_registry(&self) -> &WorthUiRetainedObligationRegistry {
        &self.retained_obligations
    }

    pub(crate) fn retained_allocation_planning_registry(
        &self,
    ) -> &WorthUiRetainedAllocationPlanningEvidenceRegistry {
        self.retained_allocation_planning_evidence.as_ref()
    }

    pub(crate) fn retained_planning_authority(
        &self,
    ) -> &Rc<WorthUiRetainedAllocationPlanningEvidenceRegistry> {
        &self.retained_allocation_planning_evidence
    }
}
