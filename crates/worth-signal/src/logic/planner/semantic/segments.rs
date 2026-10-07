use crate::data::handle::NodeId;

use crate::data::node::NodeState;

use crate::data::output::MemoizedResultOrigin;

use crate::data::reuse::ReuseBasis;

use crate::data::temporal::LoweredTemporalEligibility;

use crate::data::trace::RuntimeArtifactFinalizeImage;

use crate::logic::evaluation::EvaluationVerdict;

use crate::logic::explain::RewiringSummary;

use super::super::types::SemanticTaskRange;
use super::super::types::{ExecutionRecordId, SemanticSegmentId};

#[derive(Debug, Clone, Copy)]
pub(in crate::logic::planner) struct StageSemanticIdentity {
    pub record_id: ExecutionRecordId,
    pub segment_id: SemanticSegmentId,
}

#[derive(Debug, Clone)]
pub(in crate::logic::planner) struct SemanticTaskUpdate {
    task_index: usize,
    node: NodeId,
    identity: StageSemanticIdentity,
    before_state: NodeState,
    before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
    after_state: NodeState,
    dependency_updates: u32,
    recomputed: bool,
    partition_aware: bool,
    temporal_eligibility: Option<LoweredTemporalEligibility>,
    rewiring: Option<RewiringSummary>,
    verdict: EvaluationVerdict,
    memoized_origin: MemoizedResultOrigin,
    reuse_basis: ReuseBasis,
    prepared_artifacts: Option<crate::data::graph::PreparedSemanticArtifactImage>,
}

#[derive(Debug, Clone)]
pub(in crate::logic::planner) struct SemanticSegment {
    id: SemanticSegmentId,
    task_range: SemanticTaskRange,
    updates: Vec<SemanticTaskUpdate>,
}

#[derive(Debug, Clone, Default)]
pub(in crate::logic::planner) struct StageSemanticBatch {
    segments: Vec<SemanticSegment>,
}

impl StageSemanticBatch {
    pub(in crate::logic::planner) fn push_segment(&mut self, segment: SemanticSegment) {
        self.segments.push(segment);
    }

    pub(in crate::logic::planner) fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub(in crate::logic::planner) fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub(super) fn into_segments(self) -> Vec<SemanticSegment> {
        self.segments
    }
}

impl SemanticTaskUpdate {
    pub(in crate::logic::planner) fn new(
        task_index: usize,
        node: NodeId,
        identity: StageSemanticIdentity,
        before_state: NodeState,
        before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
        after_state: NodeState,
        dependency_updates: u32,
        recomputed: bool,
        partition_aware: bool,
        temporal_eligibility: Option<LoweredTemporalEligibility>,
        rewiring: Option<RewiringSummary>,
        verdict: EvaluationVerdict,
        memoized_origin: MemoizedResultOrigin,
        reuse_basis: ReuseBasis,
    ) -> Self {
        Self {
            task_index,
            node,
            identity,
            before_state,
            before_artifact_state,
            after_state,
            dependency_updates,
            recomputed,
            partition_aware,
            temporal_eligibility,
            rewiring,
            verdict,
            memoized_origin,
            reuse_basis,
            prepared_artifacts: None,
        }
    }

    pub(in crate::logic::planner) fn with_prepared_artifacts(
        mut self,
        artifacts: crate::data::graph::PreparedSemanticArtifactImage,
    ) -> Self {
        self.prepared_artifacts = Some(artifacts);
        self
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        usize,
        NodeId,
        StageSemanticIdentity,
        NodeState,
        Option<RuntimeArtifactFinalizeImage>,
        NodeState,
        u32,
        bool,
        bool,
        Option<LoweredTemporalEligibility>,
        Option<RewiringSummary>,
        EvaluationVerdict,
        MemoizedResultOrigin,
        ReuseBasis,
        Option<crate::data::graph::PreparedSemanticArtifactImage>,
    ) {
        (
            self.task_index,
            self.node,
            self.identity,
            self.before_state,
            self.before_artifact_state,
            self.after_state,
            self.dependency_updates,
            self.recomputed,
            self.partition_aware,
            self.temporal_eligibility,
            self.rewiring,
            self.verdict,
            self.memoized_origin,
            self.reuse_basis,
            self.prepared_artifacts,
        )
    }
}

impl SemanticSegment {
    fn single(update: SemanticTaskUpdate) -> Self {
        Self {
            id: update.identity.segment_id,
            task_range: SemanticTaskRange {
                start: update.identity.record_id,
                end: update.identity.record_id,
            },
            updates: vec![update],
        }
    }

    pub(super) fn id(&self) -> SemanticSegmentId {
        self.id
    }

    pub(super) fn task_range(&self) -> SemanticTaskRange {
        self.task_range
    }

    pub(super) fn into_updates(self) -> Vec<SemanticTaskUpdate> {
        self.updates
    }
}

pub(in crate::logic::planner) fn segment_for_single_update(
    update: SemanticTaskUpdate,
) -> SemanticSegment {
    SemanticSegment::single(update)
}

pub(in crate::logic::planner) fn reserve_stage_identities(
    next_record_id: &mut u64,
    next_segment_id: &mut u64,
    task_count: usize,
) -> Vec<StageSemanticIdentity> {
    let first_record = *next_record_id;
    let first_segment = *next_segment_id;
    *next_record_id += task_count as u64;
    *next_segment_id += task_count as u64;
    (0..task_count)
        .map(|offset| StageSemanticIdentity {
            record_id: ExecutionRecordId(first_record + offset as u64),
            segment_id: SemanticSegmentId(first_segment + offset as u64),
        })
        .collect()
}
