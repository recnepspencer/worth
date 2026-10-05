#[cfg(test)]
use crate::data::output::CanonicalChangedRegions;
use crate::data::trace::{
    assemble_historical_artifact_record, assemble_trace_summary_with_execution, ArtifactWriteDelta,
    CausalityMetadata, ExecutionTraceStamp, HistoricalArtifactRecord, RetainedDiagnosticArtifact,
    RuntimeArtifactState, TraceSummary,
};

#[cfg(test)]
use crate::data::trace::{RuntimeArtifactHot, RuntimeArtifactWarm};

use super::NodeEntry;

#[cfg_attr(not(test), allow(dead_code))]
impl NodeEntry {
    /// The last operational artifact state.
    pub fn get_runtime_artifact_state(&self) -> Option<&RuntimeArtifactState> {
        self.warm.runtime_artifact_state.as_deref()
    }

    /// Set or clear the runtime artifact state.
    pub fn set_runtime_artifact_state(&mut self, state: Option<RuntimeArtifactState>) {
        self.warm.runtime_artifact_state = state.map(std::sync::Arc::new);
    }

    /// Mutably access the runtime artifact state when an operation needs to
    /// update warm metadata in place without rebuilding the whole carrier.
    #[allow(dead_code)]
    pub fn runtime_artifact_state_mut(&mut self) -> Option<&mut RuntimeArtifactState> {
        self.warm
            .runtime_artifact_state
            .as_mut()
            .map(std::sync::Arc::make_mut)
    }

    /// Retained diagnostic artifact payload, if any.
    pub fn retained_diagnostic_artifact(&self) -> Option<&RetainedDiagnosticArtifact> {
        self.cold.as_ref()?.retained_artifact.as_ref()
    }

    /// Cold retained artifact record, if any.
    pub fn cold_artifact_record(&self) -> Option<&RetainedDiagnosticArtifact> {
        self.retained_diagnostic_artifact()
    }

    /// Assemble a cold historical artifact record from the published hot/cold
    /// facades for this node entry.
    pub fn historical_artifact_record(
        &self,
        node: crate::data::handle::NodeId,
    ) -> Option<HistoricalArtifactRecord> {
        assemble_historical_artifact_record(
            node,
            self.get_runtime_artifact_state(),
            self.cold_artifact_record(),
            self.get_causality(),
        )
    }

    /// Assemble a trace summary from the published hot/cold facades for this
    /// node entry.
    pub fn trace_summary(&self) -> Option<TraceSummary> {
        assemble_trace_summary_with_execution(
            self.get_runtime_artifact_state(),
            self.cold_artifact_record(),
            self.execution_trace_stamp(),
        )
    }

    /// Set or clear the retained diagnostic artifact payload.
    pub fn set_retained_diagnostic_artifact(
        &mut self,
        artifact: Option<RetainedDiagnosticArtifact>,
    ) {
        self.cold_mut().retained_artifact = artifact;
        self.trim_cold_if_empty();
    }

    /// Cold execution/segment stamp, if any.
    pub fn execution_trace_stamp(&self) -> Option<ExecutionTraceStamp> {
        self.cold.as_ref()?.execution_trace
    }

    /// Set or clear the cold execution/segment stamp.
    pub fn set_execution_trace_stamp(&mut self, stamp: Option<ExecutionTraceStamp>) {
        self.cold_mut().execution_trace = stamp;
        self.trim_cold_if_empty();
    }

    /// Apply explicit hot/cold artifact lane updates without implying that the
    /// lanes are a single ambient payload.
    #[allow(dead_code)]
    pub fn apply_artifact_write_delta(&mut self, delta: ArtifactWriteDelta) {
        self.set_runtime_artifact_state(delta.runtime);
        self.set_retained_diagnostic_artifact(delta.retained);
    }

    /// Split a materialized trace summary back into runtime and retained
    /// storage lanes.
    #[cfg(test)]
    pub fn set_trace_summary(&mut self, summary: Option<TraceSummary>) {
        match summary {
            Some(summary) => {
                let retained_changed_regions =
                    CanonicalChangedRegions::from(summary.changed_regions.clone());
                self.warm.runtime_artifact_state =
                    Some(std::sync::Arc::new(RuntimeArtifactState::new(
                        RuntimeArtifactHot {
                            output_hash: summary.output_hash,
                            output_change: summary.output_change,
                            recomputed: summary.recomputed,
                            dependency_count: summary.dependency_count,
                            meaningful_input_changes: summary.meaningful_input_changes,
                            changed_partition_count: summary.changed_partition_count,
                            propagation_suppressed: summary.propagation_suppressed,
                            changed_scopes: crate::data::trace::CompactChangedScopeProof::new(
                                crate::data::proof::PartitionScopeSet::from_changed_regions(
                                    &retained_changed_regions,
                                ),
                            ),
                        },
                        RuntimeArtifactWarm {
                            output_identity: summary.output_identity,
                            continuity_token: crate::data::trace::ContinuityAuthorityToken::new(
                                summary.continuity_token,
                            ),
                            memoized_origin: summary.memoized_origin,
                            reuse_basis: crate::data::trace::ReuseOperationalBasis::new(
                                summary.reuse_basis,
                            ),
                            reuse_origin: summary.reuse_origin,
                            reuse_boundary_authority: summary
                                .reuse_boundary_context
                                .as_ref()
                                .map(|context| context.authority()),
                            lineage_artifact_id: crate::data::trace::ArtifactTransitionKey::new(
                                summary.lineage_artifact_id,
                            ),
                            merge_authority: crate::data::trace::ArtifactMergeAuthority::default(),
                        },
                    )));
                self.set_execution_trace_stamp(Some(ExecutionTraceStamp {
                    execution_record_id: summary.execution_record_id,
                    semantic_segment_id: summary.semantic_segment_id,
                }));
                let retained = RetainedDiagnosticArtifact {
                    changed_regions: retained_changed_regions,
                    labels: summary.labels,
                    keyed_family: summary.keyed_family,
                    keyed_key: summary.keyed_key,
                    reuse_certification: None,
                    reuse_boundary_context: summary.reuse_boundary_context,
                };
                if retained.changed_regions.is_empty()
                    && retained.labels.is_empty()
                    && retained.keyed_family.is_none()
                    && retained.keyed_key.is_none()
                    && retained.reuse_certification.is_none()
                    && retained.reuse_boundary_context.is_none()
                {
                    self.set_retained_diagnostic_artifact(None);
                } else {
                    self.set_retained_diagnostic_artifact(Some(retained));
                }
            }
            None => {
                self.warm.runtime_artifact_state = None;
                self.set_retained_diagnostic_artifact(None);
                self.set_execution_trace_stamp(None);
            }
        }
    }

    /// Optional host-provided causality payload.
    pub fn get_causality(&self) -> Option<&CausalityMetadata> {
        self.cold.as_ref()?.causality.as_ref()
    }

    /// Set or clear the causality payload.
    pub fn set_causality(&mut self, causality: Option<CausalityMetadata>) {
        self.cold_mut().causality = causality;
        self.trim_cold_if_empty();
    }

    fn cold_mut(&mut self) -> &mut super::layout::NodeColdData {
        self.cold
            .get_or_insert_with(|| Box::new(super::layout::NodeColdData::default()))
            .as_mut()
    }

    fn trim_cold_if_empty(&mut self) {
        if self.cold.as_ref().is_some_and(|cold| {
            cold.retained_artifact.is_none()
                && cold.causality.is_none()
                && cold.execution_trace.is_none()
        }) {
            self.cold = None;
        }
    }
}
