mod change_classification;
mod framework_turn;
mod inspection;
mod mounted_allocation;
#[path = "application_state/service_proposal.rs"]
mod service_proposal;
pub(crate) use service_proposal::{
    UiIndeterminatePortalProposalTransaction, UiPortalProposalPreparationDenial,
    UiStagedPortalProposalTransaction,
};
mod appearance_consumers;
#[cfg(any(test, feature = "certification-support"))]
mod planning;
mod rebind_planning;
mod rebind_publication;
mod replacement;
mod selection_mapping;

pub(crate) use selection_mapping::UiDeclaredSelectionMappingDenial;

pub(crate) use replacement::WorthUiRuntimePublicationBasis;

use crate::facade::prepared_application_authority::{
    WorthUiPreparedApplicationGenerationIdentity, WorthUiPreparedApplicationGraphSuccessor,
};
use crate::facade::registry::snapshot::CapabilitySnapshot;
use crate::facade::WorthUiApp;
use crate::graph::{UiGraphAuthority, UiGraphNodeIdentity};
use crate::runtime::{
    WorthUiRuntime, WorthUiRuntimeShutdownReceipt, WorthUiSourceEventIngress, WorthUiSourceProvider,
};

/// Application/runtime authority retained by one active session.
///
/// The active-session composition root delegates named application operations
/// here instead of lending the complete app or runtime to sibling subsystems.
pub(crate) struct WorthUiApplicationSessionState {
    app: WorthUiApp,
    runtime: WorthUiRuntime,
}

impl WorthUiApplicationSessionState {
    pub(crate) fn new(app: WorthUiApp, runtime: WorthUiRuntime) -> Self {
        Self { app, runtime }
    }

    pub(crate) fn generation_identity(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        self.app.generation_identity()
    }

    pub(crate) fn authored_overlay_material(
        &self,
    ) -> &crate::runtime::WorthUiAuthoredOverlayMaterial {
        self.app.prepared_authority().authored_overlay_material()
    }

    pub(crate) fn host_session_plan(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiHostSessionPlan {
        self.app.host_session_plan()
    }

    pub(crate) fn font_collection(&self) -> &std::sync::Arc<worth_ui_text::UiGlobalFontCollection> {
        self.app.font_collection()
    }

    pub(crate) fn capabilities(&self) -> &CapabilitySnapshot {
        self.app.capabilities()
    }

    pub(crate) fn intent_application_fact_plan(
        &self,
    ) -> &crate::declaration::UiIntentApplicationFactPlan {
        self.app.prepared_authority().intent_application_fact_plan()
    }

    pub(crate) fn graph(&self) -> UiGraphAuthority<'_> {
        self.app.graph()
    }

    pub(crate) fn graph_snapshot(&self) -> &crate::graph::UiGraphSnapshot {
        self.app.graph_snapshot()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn lookup_consumed_fact(
        &self,
        fact: &crate::fact_contract::UiProducedFact,
    ) -> Result<crate::graph::UiGraphFactLookupReceipt, crate::graph::UiGraphFactLookupDenial> {
        let prepared = self.app.prepared_authority();
        let index = prepared.consumed_fact_index();
        index.lookup(index.basis(), fact)
    }

    pub(crate) fn source_event_ingress(
        &self,
        provider: WorthUiSourceProvider,
    ) -> WorthUiSourceEventIngress {
        self.runtime.source_event_ingress(provider)
    }

    pub(crate) fn begin_observation_turn<'state>(
        &'state mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        appearance_close: Option<
            crate::runtime::observation::UiAppearanceObservationCloseInput<'state>,
        >,
    ) -> Result<
        crate::facade::observation::UiObservationTurn<'state>,
        crate::facade::observation::UiObservationTurnDenial,
    > {
        let source_basis = self.app.capabilities().digest().as_u64();
        self.runtime.begin_observation_turn_with_appearance_close(
            session,
            source_basis,
            appearance_close,
        )
    }

    pub(crate) fn observation_resource_snapshot(
        &self,
    ) -> crate::runtime::observation::UiObservationResourceSnapshot {
        self.runtime.observation.resource_snapshot()
    }

    pub(crate) fn retire_observation_resources(
        &mut self,
        cause: crate::runtime::observation::UiObservationResourceRetirementCause,
    ) -> crate::runtime::observation::UiObservationResourceRetirementReport {
        self.runtime.observation.retire_resources(cause)
    }

    pub(crate) fn commit_prepared_observation_progress(
        &mut self,
        commit: crate::runtime::observation::UiPreparedObservationProgressCommit,
    ) {
        self.runtime.commit_prepared_observation_progress(commit);
    }

    pub(crate) fn prepare_exact_query_change_publication(
        &mut self,
        reference: &worth_ui_query_binding::WorthUiInstalledQueryBindingReference,
    ) -> Result<
        worth_ui_query_binding::WorthUiAdmittedCollectionChangePublication,
        crate::runtime::intent_execution::UiIntentConsequenceStopReason,
    > {
        let consequence = self
            .runtime
            .query_binding
            .retry_operation_live_change_handoff(reference)
            .map_err(
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::QueryHandoff,
            )?;
        self.runtime
            .query_binding
            .admit_operation_live_change_for_publication(consequence)
            .map_err(|stop| {
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::QueryAdmission(
                    stop.denial(),
                )
            })
    }

    pub(crate) fn publish_exact_query_change(
        &mut self,
        admission: worth_ui_query_binding::WorthUiAdmittedCollectionChangePublication,
    ) -> Result<
        worth_ui_query_binding::WorthUiCollectionChangePublicationReceipt,
        worth_ui_query_binding::WorthUiCollectionChangePublicationStop,
    > {
        self.runtime
            .query_binding
            .publish_admitted_operation_live_change(admission)
    }

    pub(crate) fn withdraw_exact_query_change(
        &mut self,
        admission: worth_ui_query_binding::WorthUiAdmittedCollectionChangePublication,
    ) -> Result<
        worth_ui_query_binding::WorthUiCollectionChangeConsequence,
        worth_ui_query_binding::WorthUiCollectionChangePublicationStop,
    > {
        self.runtime
            .query_binding
            .withdraw_admitted_operation_live_change(admission)
    }

    pub(crate) fn admission(&self) -> crate::admission::UiAdmissionBoundary<'_> {
        self.app.admission()
    }

    pub(crate) fn try_allocation_touch_for_node(
        &self,
        graph_node_identity: UiGraphNodeIdentity,
    ) -> Result<
        crate::obligations::touch::UiGraphTouchDescriptor,
        crate::obligations::touch::UiGraphTouchDenial,
    > {
        self.app.try_allocation_touch_for_node(graph_node_identity)
    }

    pub(crate) fn allocation_truth_revision(&self) -> crate::runtime::UiAllocationTruthRevision {
        self.runtime.allocation_truth_revision()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn refresh_query_change_for_certification(
        &mut self,
        request: worth_ui_query_binding::WorthUiOperationLiveRefreshRequest<'_>,
    ) -> Result<
        worth_ui_query_binding::WorthUiOperationLiveRefreshOutcome,
        worth_ui_query_binding::WorthUiOperationLiveRefreshError,
    > {
        self.runtime.query_binding.refresh_operation_live(request)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn query_change_state_for_certification(
        &self,
        reference: &worth_ui_query_binding::WorthUiInstalledQueryBindingReference,
    ) -> Result<
        worth_ui_query_binding::WorthUiOperationLiveChangeObservation,
        worth_ui_query_binding::WorthUiQueryViewExecutionEvidenceDenial,
    > {
        self.runtime
            .query_binding
            .operation_live_change_observation_for(reference)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn measurement_basis_sources_for_certification(
        &self,
    ) -> Box<[crate::declaration::UiDeclaredMeasurementBasisSource]> {
        self.app
            .declaration_artifacts()
            .iter()
            .filter_map(|artifact| artifact.graph_handoff().ok())
            .filter_map(|handoff| {
                handoff
                    .measurement_policy()
                    .admitted()
                    .and_then(|policy| policy.basis_source())
            })
            .collect()
    }

    pub(crate) fn host_measurement_collector(
        &self,
    ) -> crate::host::WorthUiHostMeasurementCollector {
        self.runtime.host_measurement_collector()
    }

    pub(crate) fn viewport_measurement_witnesses(
        &self,
    ) -> Box<[crate::evidence::UiHostMeasurementAuthorityWitness]> {
        self.runtime
            .allocation_invalidation_index
            .borrow()
            .viewport_measurement_witnesses()
    }

    pub(crate) fn prepare_graph_successor(
        &self,
        commit: crate::graph::UiGraphMutationCommitResult,
    ) -> Result<
        WorthUiPreparedApplicationGraphSuccessor,
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGraphSuccessorDenial,
    >{
        self.app.prepare_graph_successor(commit)
    }

    pub(crate) fn shutdown(self) -> WorthUiRuntimeShutdownReceipt {
        self.runtime
            .shutdown()
            .unwrap_or_else(|recovery| panic!("runtime shutdown blocked: {:?}", recovery.blocker()))
    }

    #[cfg(test)]
    pub(crate) fn into_runtime_for_test(self) -> WorthUiRuntime {
        self.runtime
    }
}
