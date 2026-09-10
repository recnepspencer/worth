use super::GraphObserver;
use crate::data::graph::EvaluationStrategy;
use crate::data::node::{node_hot_inline_size_bytes, node_warm_inline_size_bytes};
#[cfg(test)]
use crate::data::proof::FrontierDiagnosticsSidecar;
use crate::data::proof::{InvalidationPlanningEstimate, InvalidationTraceRecord};
use crate::data::trace::{ColdArtifactRecord, RuntimeArtifactHot, RuntimeArtifactWarm};
use crate::diagnostics::access::GraphDiagnostics;
use crate::diagnostics::history::ExecutionInspector;
use crate::diagnostics::policy::OrdinaryAccessLane;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::{ExecutionHistorySummary, GraphSummary};
use crate::diagnostics::{FailureSummary, RetainedFlowSummaryView, RollbackDiagnostic};
use crate::logic::transaction::ObservationBoundarySummary;
use crate::presentation::metrics::GraphMetrics;
use crate::runtime_policy::SignalRuntimePolicy;

impl<'a> GraphObserver<'a> {
    pub fn telemetry(&self) -> &'a crate::data::telemetry::RuntimeTelemetry {
        &self.graph.observation.telemetry
    }

    pub fn metrics(&self) -> GraphMetrics {
        let mut metrics = GraphMetrics::from_runtime_telemetry(
            self.telemetry(),
            self.graph.observation.partition_interner.token_count(),
        );
        metrics.storage.hot_path_artifact_reconstruction_count =
            self.graph.hot_path_artifact_reconstruction_count();
        metrics.storage.explicit_cold_materialization_request_count =
            self.graph.explicit_cold_materialization_request_count();
        metrics.storage.retained_forensic_read_count = self.graph.retained_forensic_read_count();
        metrics.storage.cold_explanation_reconstruction_count =
            self.graph.cold_explanation_reconstruction_count();
        metrics.storage.cold_provenance_reconstruction_count =
            self.graph.cold_provenance_reconstruction_count();
        metrics.storage.retained_artifact_read_count = self.graph.retained_artifact_read_count();
        metrics.storage.reconstructed_artifact_read_count =
            self.graph.reconstructed_artifact_read_count();
        metrics.storage.denied_reconstruction_by_budget_count =
            self.graph.denied_reconstruction_by_budget_count();
        metrics.storage.denied_reconstruction_by_tier_count =
            self.graph.denied_reconstruction_by_tier_count();
        metrics.storage.denied_reconstruction_explanation_api_count =
            self.graph.denied_reconstruction_explanation_api_count();
        metrics.storage.denied_reconstruction_provenance_api_count =
            self.graph.denied_reconstruction_provenance_api_count();
        metrics.storage.hot_node_inline_size_bytes = node_hot_inline_size_bytes();
        metrics.storage.warm_node_inline_size_bytes = node_warm_inline_size_bytes();
        metrics.storage.definition_node_inline_size_bytes =
            std::mem::size_of::<crate::data::node::NodeDefinitionData>() as u64;
        metrics.storage.hot_runtime_artifact_inline_size_bytes =
            std::mem::size_of::<RuntimeArtifactHot>() as u64;
        metrics.storage.warm_runtime_artifact_inline_size_bytes =
            std::mem::size_of::<RuntimeArtifactWarm>() as u64;
        metrics.storage.cold_artifact_record_inline_size_bytes =
            std::mem::size_of::<ColdArtifactRecord>() as u64;
        metrics
    }

    pub fn diagnostics_profile(&self) -> DiagnosticsTier {
        self.graph.observation.diagnostics.tier()
    }

    pub fn evaluation_strategy(&self) -> EvaluationStrategy {
        self.graph.derive_evaluation_strategy()
    }

    pub fn runtime_policy(&self) -> SignalRuntimePolicy {
        self.graph.runtime_policy()
    }

    pub fn diagnostics_summary(&self, profile: DiagnosticsTier) -> GraphSummary {
        if self.graph.diagnostics_state().has_pending_change_input() {
            if let Some(summary) = self.graph.diagnostics_state().pending_graph_summary() {
                return summary.with_profile(profile);
            }
        } else if let Some(summary) = self.graph.diagnostics_state().latest_graph_summary() {
            return summary.with_profile(profile);
        }
        GraphSummary::from_graph(
            self.graph,
            profile,
            self.graph
                .installed_runtime_policy()
                .retention_budget()
                .detail_limit,
            OrdinaryAccessLane,
        )
    }

    pub fn diagnostics(&self) -> GraphDiagnostics<'a> {
        GraphDiagnostics::new(self.graph)
    }

    pub fn execution_history_summary(&self, profile: DiagnosticsTier) -> ExecutionHistorySummary {
        let retention_budget = self.graph.installed_runtime_policy().retention_budget();
        if let Some(summary) = self.graph.diagnostics_state().recent_history().back() {
            if !retention_budget.retain_history_details || !summary.nodes.is_empty() {
                return summary.with_profile(profile);
            }
        }
        ExecutionHistorySummary::from_graph(
            self.graph,
            profile,
            retention_budget.detail_limit,
            retention_budget.retain_history_details,
            OrdinaryAccessLane,
        )
    }

    pub fn inspect_execution(&self) -> ExecutionInspector<'a> {
        ExecutionInspector { graph: self.graph }
    }

    pub fn latest_flow_diagnostics(&self) -> Option<RetainedFlowSummaryView<'a>> {
        self.graph.observation.diagnostics.latest_flow()
    }

    pub fn latest_failure_diagnostics(&self) -> Option<&'a FailureSummary> {
        self.graph.observation.diagnostics.latest_failure()
    }

    pub fn latest_rollback_diagnostics(&self) -> Option<&'a RollbackDiagnostic> {
        self.graph.observation.diagnostics.latest_rollback()
    }

    pub fn latest_observation_summary(&self) -> Option<&'a ObservationBoundarySummary> {
        self.graph.observation.diagnostics.latest_observation()
    }

    #[cfg(test)]
    pub(crate) fn latest_frontier_execution_summary(
        &self,
    ) -> Option<&'a FrontierDiagnosticsSidecar> {
        self.graph
            .observation
            .diagnostics
            .latest_frontier_execution()
    }

    pub fn latest_invalidation_planning_estimate(
        &self,
    ) -> Option<&'a InvalidationPlanningEstimate> {
        self.graph
            .observation
            .diagnostics
            .latest_invalidation_planning_estimate()
    }

    pub fn latest_invalidation_trace_records(&self) -> &'a [InvalidationTraceRecord] {
        self.graph
            .observation
            .diagnostics
            .latest_invalidation_trace_records()
    }

    pub fn recent_execution_history_diagnostics(
        &self,
    ) -> crate::diagnostics::summary::RetainedExecutionHistoryView<'a> {
        crate::diagnostics::summary::RetainedExecutionHistoryView::new(
            self.graph.observation.diagnostics.recent_history(),
        )
    }
}
