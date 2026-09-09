use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::{NodeEvaluationConfig, NodeState};
use crate::data::trace::{
    assemble_historical_artifact_record, assemble_trace_summary_with_execution, CausalityMetadata,
    ColdArtifactRecord, ExecutionTraceStamp, HistoricalArtifactRecord, RetainedDiagnosticArtifact,
    RuntimeArtifactState, TraceSummary,
};

use super::NodeReplayProjection;

pub(crate) struct NodeExplanationStorageView<'a> {
    node: NodeId,
    state: NodeState,
    dirty_aspects: crate::data::aspect::AspectMask,
    evaluation_config: &'a NodeEvaluationConfig,
    runtime: Option<&'a RuntimeArtifactState>,
    retained: Option<&'a ColdArtifactRecord>,
    execution: Option<ExecutionTraceStamp>,
    causality: Option<&'a CausalityMetadata>,
}

impl NodeExplanationStorageView<'_> {
    pub(crate) fn state(&self) -> NodeState {
        self.state
    }

    pub(crate) fn dirty_aspects(&self) -> crate::data::aspect::AspectMask {
        self.dirty_aspects
    }

    pub(crate) fn evaluation_config(&self) -> &NodeEvaluationConfig {
        self.evaluation_config
    }

    pub(crate) fn historical_artifact_record(&self) -> Option<HistoricalArtifactRecord> {
        assemble_historical_artifact_record(self.node, self.runtime, self.retained, self.causality)
    }

    pub(crate) fn trace_summary(&self) -> Option<TraceSummary> {
        assemble_trace_summary_with_execution(self.runtime, self.retained, self.execution)
    }

    pub(crate) fn causality(&self) -> Option<&CausalityMetadata> {
        self.causality
    }

    pub(crate) fn matches_historical_artifact_record(
        &self,
        expected: Option<&crate::data::trace::HistoricalArtifactRecord>,
    ) -> bool {
        match (self.runtime, expected) {
            (None, None) => true,
            (Some(runtime), Some(expected)) => {
                expected.node == self.node
                    && &expected.runtime == runtime
                    && expected.retained.as_ref() == self.retained
                    && expected.causality.as_ref() == self.causality
            }
            _ => false,
        }
    }
}

impl SignalGraph {
    pub(crate) fn node_explanation_storage_view(
        &self,
        node: NodeId,
    ) -> Result<NodeExplanationStorageView<'_>, SignalError> {
        self.validate_handle(node)?;
        let index = node.index() as usize;
        let hot = self.arena.hot[index]
            .as_ref()
            .expect("validated live node must retain hot storage");
        let warm = &self.arena.warm[index];
        let cold = self.arena.cold[index].as_deref();
        Ok(NodeExplanationStorageView {
            node,
            state: hot.state,
            dirty_aspects: hot.dirty_aspects,
            evaluation_config: &self.arena.definitions[index].eval_config,
            runtime: warm.runtime_artifact_state.as_ref(),
            retained: cold.and_then(|cold| cold.retained_artifact.as_ref()),
            execution: cold.and_then(|cold| cold.execution_trace),
            causality: cold.and_then(|cold| cold.causality.as_ref()),
        })
    }

    pub fn set_trace_summary(
        &mut self,
        id: NodeId,
        summary: Option<TraceSummary>,
    ) -> Result<(), SignalError> {
        let mut entry = self.get_entry_mut(id)?;
        entry.set_retained_diagnostic_artifact(summary.map(|summary| RetainedDiagnosticArtifact {
            labels: summary.labels,
            ..RetainedDiagnosticArtifact::default()
        }));
        Ok(())
    }

    pub fn causality_of(&self, node: NodeId) -> Result<Option<&CausalityMetadata>, SignalError> {
        Ok(self
            .cold_ref(node)?
            .and_then(|cold| cold.causality.as_ref()))
    }

    pub(crate) fn node_execution_trace_stamp(
        &self,
        node: NodeId,
    ) -> Result<Option<ExecutionTraceStamp>, SignalError> {
        Ok(self.cold_ref(node)?.and_then(|cold| cold.execution_trace))
    }

    pub(crate) fn node_retained_diagnostic_artifact(
        &self,
        node: NodeId,
    ) -> Result<Option<&RetainedDiagnosticArtifact>, SignalError> {
        crate::data::access_counters::note_retained_artifact_read();
        Ok(self
            .cold_ref(node)?
            .and_then(|cold| cold.retained_artifact.as_ref()))
    }

    pub(crate) fn node_cold_artifact_record(
        &self,
        node: NodeId,
    ) -> Result<Option<&ColdArtifactRecord>, SignalError> {
        Ok(self
            .cold_ref(node)?
            .and_then(|cold| cold.retained_artifact.as_ref()))
    }

    pub(crate) fn node_lineage_artifact_id(
        &self,
        node: NodeId,
    ) -> Result<Option<crate::diagnostics::lineage::LineageArtifactId>, SignalError> {
        Ok(self
            .warm_ref(node)?
            .runtime_artifact_state
            .as_ref()
            .and_then(|state| state.lineage_artifact_id().get()))
    }

    pub(crate) fn node_replay_projection(
        &self,
        node: NodeId,
    ) -> Result<NodeReplayProjection, SignalError> {
        let runtime_artifact_state = self.warm_ref(node)?.runtime_artifact_state.as_ref();
        let lineage_artifact_id =
            runtime_artifact_state.and_then(|state| state.lineage_artifact_id().get());
        let (persistent_correspondence_kind, composition_region_count) = runtime_artifact_state
            .and_then(|state| state.reuse_boundary_authority())
            .map(|authority| {
                (
                    authority.persistent_correspondence_kind(),
                    authority.composition_region_count(),
                )
            })
            .unwrap_or((None, 0));
        Ok(NodeReplayProjection {
            lineage_artifact_id,
            persistent_correspondence_kind,
            composition_region_count: (composition_region_count > 0)
                .then_some(composition_region_count),
        })
    }

    pub fn set_causality(
        &mut self,
        node: NodeId,
        causality: Option<CausalityMetadata>,
    ) -> Result<(), SignalError> {
        self.validate_handle(node)?;
        super::evaluation_payload::set_causality(
            &mut self.arena.cold[node.index() as usize],
            causality,
        );
        self.record_branch_mutation_causality(node);
        Ok(())
    }

    pub(crate) fn stamp_runtime_artifact_lineage_and_execution(
        &mut self,
        node: NodeId,
        artifact_id: crate::diagnostics::lineage::LineageArtifactId,
        execution_record_id: crate::logic::planner::ExecutionRecordId,
        semantic_segment_id: crate::logic::planner::SemanticSegmentId,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        // Finalization already selected the artifact image. The mutation owner
        // validates the handle and handles absent payloads without another read.
        let growth = crate::data::retained_storage::RetainedStorageCharge::capacity::<
            crate::data::node::NodeColdData,
        >(1)
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        self.mutate_evaluation_node(node, growth, work, |target| {
            target.stamp_lineage_and_execution(
                artifact_id,
                execution_record_id,
                semantic_segment_id,
            );
        })
    }
}
