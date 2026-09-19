//! Node-local diagnostic payload writes, without branch-history publication.
use crate::data::node::NodeColdData;
use crate::data::trace::{CausalityMetadata, ExecutionTraceStamp, RuntimeArtifactState};
use crate::diagnostics::lineage::LineageArtifactId;
use crate::logic::planner::{ExecutionRecordId, SemanticSegmentId};

pub(in crate::data::graph) fn set_causality(
    cold: &mut Option<Box<NodeColdData>>,
    causality: Option<CausalityMetadata>,
) {
    if causality.is_some() {
        cold.get_or_insert_with(|| Box::new(NodeColdData::default()))
            .causality = causality;
    } else if let Some(cold) = cold.as_mut() {
        cold.causality = None;
    }
    super::artifacts::trim_empty_cold(cold);
}

pub(in crate::data::graph) fn stamp_lineage_and_execution(
    runtime: &mut RuntimeArtifactState,
    cold: &mut Option<Box<NodeColdData>>,
    artifact_id: LineageArtifactId,
    execution_record_id: ExecutionRecordId,
    semantic_segment_id: SemanticSegmentId,
) {
    runtime.set_lineage_artifact_id(Some(artifact_id));
    cold.get_or_insert_with(|| Box::new(NodeColdData::default()))
        .execution_trace = Some(ExecutionTraceStamp {
        execution_record_id: Some(execution_record_id.0),
        semantic_segment_id: Some(semantic_segment_id.0),
    });
}
