use serde::{Deserialize, Serialize};

use crate::data::core_profile::StableHashValue;
use crate::data::handle::NodeId;
use crate::data::output::{
    ArtifactContinuityToken, ChangedRegion, MemoizedResultOrigin, OutputChange, OutputIdentity,
    PartitionSubscription,
};
use crate::data::reuse::{ReuseBasis, ReuseBoundaryContext, ReuseOrigin};
use crate::diagnostics::lineage::LineageArtifactId;

use super::records::{ExecutionTraceStamp, HistoricalArtifactRecord};
use super::tiers::RuntimeArtifactState;
use super::writes::ColdArtifactRecord;

/// Materialized trace view assembled for explanation, comparison, and
/// historical reporting surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TraceSummary {
    pub output_hash: StableHashValue,
    #[serde(default)]
    pub output_identity: Option<OutputIdentity>,
    #[serde(default)]
    pub continuity_token: Option<ArtifactContinuityToken>,
    #[serde(default)]
    pub output_change: OutputChange,
    #[serde(default)]
    pub recomputed: bool,
    #[serde(default)]
    pub dependency_count: u32,
    #[serde(default)]
    pub meaningful_input_changes: u32,
    #[serde(default)]
    pub changed_partition_count: u32,
    #[serde(default)]
    pub propagation_suppressed: bool,
    #[serde(default)]
    pub changed_regions: Vec<ChangedRegion>,
    #[serde(default)]
    pub keyed_family: Option<String>,
    #[serde(default)]
    pub keyed_key: Option<String>,
    #[serde(default)]
    pub memoized_origin: MemoizedResultOrigin,
    #[serde(default)]
    pub reuse_basis: ReuseBasis,
    #[serde(default)]
    pub reuse_origin: ReuseOrigin,
    #[serde(default)]
    pub reuse_boundary_context: Option<ReuseBoundaryContext>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub execution_record_id: Option<u64>,
    #[serde(default)]
    pub semantic_segment_id: Option<u64>,
    #[serde(default)]
    pub lineage_artifact_id: Option<LineageArtifactId>,
}

impl TraceSummary {
    pub fn from_parts(
        runtime: &RuntimeArtifactState,
        retained: Option<&ColdArtifactRecord>,
        execution: Option<ExecutionTraceStamp>,
    ) -> Self {
        Self {
            output_hash: runtime.output_hash(),
            output_identity: runtime.output_identity().cloned(),
            continuity_token: runtime.continuity_token_authority().clone_inner(),
            output_change: runtime.output_change(),
            recomputed: runtime.recomputed(),
            dependency_count: runtime.dependency_count(),
            meaningful_input_changes: runtime.meaningful_input_changes(),
            changed_partition_count: runtime.changed_partition_count(),
            propagation_suppressed: runtime.propagation_suppressed(),
            changed_regions: retained
                .map(|artifact| artifact.changed_regions.as_slice().to_vec())
                .unwrap_or_else(|| {
                    scopes_to_regions_from_slice(runtime.changed_scopes().as_slice())
                }),
            keyed_family: retained.and_then(|artifact| artifact.keyed_family.clone()),
            keyed_key: retained.and_then(|artifact| artifact.keyed_key.clone()),
            memoized_origin: runtime.memoized_origin(),
            reuse_basis: runtime.reuse_basis().clone_inner(),
            reuse_origin: runtime.reuse_origin(),
            reuse_boundary_context: retained
                .and_then(|artifact| artifact.reuse_boundary_context.clone()),
            labels: retained
                .map(|artifact| artifact.labels.clone())
                .unwrap_or_default(),
            execution_record_id: execution.and_then(|stamp| stamp.execution_record_id),
            semantic_segment_id: execution.and_then(|stamp| stamp.semantic_segment_id),
            lineage_artifact_id: runtime.lineage_artifact_id().get(),
        }
    }

    pub fn from_record(record: &HistoricalArtifactRecord) -> Self {
        Self::from_parts(&record.runtime, record.retained.as_ref(), None)
    }
}

pub fn assemble_historical_artifact_record(
    node: NodeId,
    runtime: Option<&RuntimeArtifactState>,
    retained: Option<&ColdArtifactRecord>,
    causality: Option<&super::evidence::CausalityMetadata>,
) -> Option<HistoricalArtifactRecord> {
    Some(HistoricalArtifactRecord {
        node,
        runtime: runtime?.clone(),
        retained: retained.cloned(),
        causality: causality.cloned(),
    })
}

#[allow(dead_code)]
pub fn assemble_trace_summary(
    runtime: Option<&RuntimeArtifactState>,
    retained: Option<&ColdArtifactRecord>,
) -> Option<TraceSummary> {
    assemble_trace_summary_with_execution(runtime, retained, None)
}

pub fn assemble_trace_summary_with_execution(
    runtime: Option<&RuntimeArtifactState>,
    retained: Option<&ColdArtifactRecord>,
    execution: Option<ExecutionTraceStamp>,
) -> Option<TraceSummary> {
    Some(TraceSummary::from_parts(runtime?, retained, execution))
}

pub(super) fn scopes_to_regions_from_slice(scopes: &[PartitionSubscription]) -> Vec<ChangedRegion> {
    scopes
        .iter()
        .map(|scope| match scope.coverage() {
            crate::data::output::ScopeCoverage::Exact => ChangedRegion::exact(scope.path().clone()),
            crate::data::output::ScopeCoverage::Subtree => {
                ChangedRegion::subtree(scope.path().clone())
            }
        })
        .collect()
}
