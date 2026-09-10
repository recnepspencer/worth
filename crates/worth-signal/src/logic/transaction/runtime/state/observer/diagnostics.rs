use crate::data::comparator::TierPolicyResolver;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::proof::InvalidationTraceRecord;
use crate::diagnostics::access::RuntimeDiagnostics;
use crate::diagnostics::history::ExecutionInspector;
use crate::diagnostics::lineage::{LineageArtifactId, SynthesizedLineageChain};
use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::{
    ExecutionHistorySummary, GraphSummary, TemporalDiagnosticsSummary,
};
use crate::diagnostics::{
    FailureSummary, ReplayView, RetainedFlowSummaryView, RollbackDiagnostic, SynthesizedReplaySlice,
};
use crate::logic::explain::{explain_with_policy_resolver, NodeExplanation};
use crate::logic::transaction::ObservationBoundarySummary;
use crate::presentation::metrics::RuntimeMetrics;
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::super::CheckpointRecord;
use super::metrics::{merge_evaluation_telemetry, merge_invalidation_telemetry};
use super::RuntimeObserver;

impl<'a, D, I, E, Ctx, T> RuntimeObserver<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn explain(&self, node: NodeId) -> Result<NodeExplanation, SignalError> {
        self.runtime.assert_construction_graph_access();
        let resolver = TierPolicyResolver::new(
            self.runtime.config.node_meta(),
            self.runtime.config.tier_policies(),
            self.runtime.config.fallback_comparator(),
        );
        explain_with_policy_resolver(&self.runtime.graph, node, &resolver)
    }

    pub fn metrics(&self) -> RuntimeMetrics {
        let graph_metrics = self.graph().metrics();
        RuntimeMetrics {
            evaluation: merge_evaluation_telemetry(
                graph_metrics.evaluation,
                self.runtime.telemetry.evaluation,
            ),
            invalidation: merge_invalidation_telemetry(
                graph_metrics.invalidation,
                self.runtime.telemetry.invalidation,
            ),
            transaction: self.runtime.telemetry.transaction,
            planner: graph_metrics.planner,
            execution: graph_metrics.execution,
            storage: graph_metrics.storage,
            checkpoint: self.composed_checkpoint_telemetry(),
            temporal: self.runtime.telemetry.temporal,
            host_computed: graph_metrics.host_computed,
        }
    }

    pub fn checkpoint_record(&self) -> CheckpointRecord {
        self.runtime.assert_construction_state_access();
        CheckpointRecord::from_checkpoint_telemetry(self.composed_checkpoint_telemetry())
    }

    pub fn diagnostics_summary(&self, profile: DiagnosticsTier) -> GraphSummary {
        self.graph().diagnostics_summary(profile)
    }

    pub fn temporal_diagnostics_summary(
        &self,
        profile: DiagnosticsTier,
    ) -> TemporalDiagnosticsSummary {
        self.runtime.assert_construction_state_access();
        TemporalDiagnosticsSummary::from_artifact(
            profile,
            self.runtime.temporal.frontier_snapshot(),
            crate::logic::transaction::TemporalReconstructabilityArtifact::from_temporal_state(
                &self.runtime.temporal,
            ),
            self.runtime.telemetry.temporal,
        )
    }

    pub fn temporal_diagnostics_summary_now(&self) -> TemporalDiagnosticsSummary {
        self.temporal_diagnostics_summary(self.diagnostics_profile())
    }

    pub fn diagnostics(&self) -> RuntimeDiagnostics<'a> {
        self.runtime.assert_construction_graph_access();
        crate::diagnostics::access::diagnostics_for_runtime(self.runtime)
    }

    pub fn diagnostics_profile(&self) -> DiagnosticsTier {
        self.graph().diagnostics_profile()
    }

    pub fn replay_for_node(&self, node: NodeId) -> ReplayView {
        self.graph().replay_for_node(node).to_owned_slice()
    }

    pub fn replay_for_artifact(&self, artifact_id: LineageArtifactId) -> ReplayView {
        self.graph()
            .replay_for_artifact(artifact_id)
            .to_owned_slice()
    }

    pub fn replay_from_cursor(&self, start: crate::diagnostics::ReplayCursor) -> ReplayView {
        self.graph().replay_from_cursor(start).to_owned_slice()
    }

    pub fn replay_between(
        &self,
        start: crate::diagnostics::ReplayCursor,
        end: crate::diagnostics::ReplayCursor,
    ) -> ReplayView {
        self.graph().replay_between(start, end).to_owned_slice()
    }

    pub fn replay_around_snapshot(&self, snapshot_id: SignalSnapshotId) -> ReplayView {
        self.graph()
            .replay_around_snapshot(snapshot_id)
            .to_owned_slice()
    }

    pub fn replay_for_branch(&self, branch_id: SignalBranchId) -> ReplayView {
        self.runtime.assert_construction_graph_access();
        self.runtime
            .branches
            .replay_graph(
                branch_id,
                self.runtime.graph.current_branch().id,
                &self.runtime.graph,
            )
            .map(|graph| {
                graph
                    .observe()
                    .replay_for_branch(branch_id)
                    .to_owned_slice()
            })
            .unwrap_or_default()
    }

    pub fn replay_where(
        &self,
        predicate: impl FnMut(&crate::diagnostics::ReplayEvent) -> bool,
    ) -> SynthesizedReplaySlice {
        self.graph().replay_where(predicate)
    }

    pub fn current_lineage_artifact(&self, node: NodeId) -> Option<LineageArtifactId> {
        self.graph().current_lineage_artifact(node)
    }

    pub fn lineage_chain_for_node(&self, node: NodeId) -> SynthesizedLineageChain {
        self.graph().lineage_chain_for_node(node)
    }

    pub fn lineage_chain_for_artifact(
        &self,
        artifact_id: LineageArtifactId,
    ) -> SynthesizedLineageChain {
        self.graph().lineage_chain_for_artifact(artifact_id)
    }

    pub fn execution_history_summary(&self, profile: DiagnosticsTier) -> ExecutionHistorySummary {
        self.graph().execution_history_summary(profile)
    }

    pub fn inspect_execution(&self) -> ExecutionInspector<'a> {
        self.graph().inspect_execution()
    }

    pub fn latest_flow_diagnostics(&self) -> Option<RetainedFlowSummaryView<'a>> {
        self.graph().latest_flow_diagnostics()
    }

    pub fn latest_failure_diagnostics(&self) -> Option<&'a FailureSummary> {
        self.graph().latest_failure_diagnostics()
    }

    pub fn latest_rollback_diagnostics(&self) -> Option<&'a RollbackDiagnostic> {
        self.graph().latest_rollback_diagnostics()
    }

    pub fn latest_observation_summary(&self) -> Option<&'a ObservationBoundarySummary> {
        self.runtime.assert_construction_graph_access();
        self.runtime.graph.diagnostics_state().latest_observation()
    }

    pub fn latest_invalidation_trace_records(&self) -> &'a [InvalidationTraceRecord] {
        self.graph().latest_invalidation_trace_records()
    }

    pub fn recent_execution_history_diagnostics(
        &self,
    ) -> crate::diagnostics::summary::RetainedExecutionHistoryView<'a> {
        self.graph().recent_execution_history_diagnostics()
    }

    pub fn to_dot(&self) -> String {
        self.graph().to_dot()
    }
}
