use super::{
    ArtifactTransitionKind, InvalidationCause, LineageArtifactId, LineageRecord, LineageRecordKind,
};
use crate::data::handle::NodeId;
use crate::logic::planner::{ExecutionRecordId, SemanticSegmentId};

use crate::state::SignalBranchId;

impl LineageRecord {
    pub fn artifact_transition(
        sequence: u64,
        emitted_on_branch_id: SignalBranchId,
        node: NodeId,
        artifact_id: LineageArtifactId,
        parent_artifact_id: Option<LineageArtifactId>,
        execution_record_id: ExecutionRecordId,
        semantic_segment_id: SemanticSegmentId,
        transition: ArtifactTransitionKind,
    ) -> Self {
        Self::new(
            sequence,
            emitted_on_branch_id,
            LineageRecordKind::ArtifactTransition {
                node,
                artifact_id,
                parent_artifact_id,
                execution_record_id,
                semantic_segment_id,
                transition,
            },
        )
    }

    pub fn invalidation(
        sequence: u64,
        emitted_on_branch_id: SignalBranchId,
        node: NodeId,
        artifact_id: LineageArtifactId,
        cause: InvalidationCause,
    ) -> Self {
        Self::new(
            sequence,
            emitted_on_branch_id,
            LineageRecordKind::Invalidation {
                node,
                artifact_id,
                cause,
            },
        )
    }
}
