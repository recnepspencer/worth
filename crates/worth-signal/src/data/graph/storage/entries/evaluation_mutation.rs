//! One node's evaluation writes, with installed and draft storage sharing semantics.
use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::{NodeColdData, NodeHotData, NodeState, NodeWarmData};
use crate::data::output::ChangedRegion;
use crate::data::persistent_paged_vector::PersistentPagedVector;
use crate::data::trace::ArtifactWriteDelta;

pub(in crate::data::graph) struct NodeEvaluationMutation<'a> {
    target: EvaluationMutationTarget<'a>,
}

enum EvaluationMutationTarget<'a> {
    Installed {
        index: usize,
        hot: &'a mut PersistentPagedVector<Option<NodeHotData>>,
        warm: &'a mut PersistentPagedVector<NodeWarmData>,
        cold: &'a mut PersistentPagedVector<Option<Box<NodeColdData>>>,
    },
    Draft {
        hot: &'a mut NodeHotData,
        warm: &'a mut NodeWarmData,
        cold: &'a mut Option<Box<NodeColdData>>,
    },
}

impl SignalGraph {
    pub(in crate::data::graph) fn node_evaluation_mutation(
        &mut self,
        node: NodeId,
    ) -> Result<NodeEvaluationMutation<'_>, SignalError> {
        self.hot_ref(node)?;
        Ok(NodeEvaluationMutation::installed(
            &mut self.arena,
            node.index() as usize,
        ))
    }
}

impl<'a> NodeEvaluationMutation<'a> {
    pub(in crate::data::graph) fn installed(
        arena: &'a mut crate::data::graph::runtime::graph::NodeArena,
        index: usize,
    ) -> Self {
        Self {
            target: EvaluationMutationTarget::Installed {
                index,
                hot: &mut arena.hot,
                warm: &mut arena.warm,
                cold: &mut arena.cold,
            },
        }
    }

    pub(in crate::data::graph) fn replace_dependency_topology(
        &mut self,
        dependencies: crate::data::graph::DependencySetId,
        pending: crate::data::proof::invalidation::binding::PendingDependencyRevalidation,
    ) {
        let (hot, warm) = self.hot_warm();
        super::evaluation_payload::replace_dependency_topology(hot, warm, dependencies, pending);
    }

    pub(in crate::data::graph) fn draft(
        hot: &'a mut NodeHotData,
        warm: &'a mut NodeWarmData,
        cold: &'a mut Option<Box<NodeColdData>>,
    ) -> Self {
        Self {
            target: EvaluationMutationTarget::Draft { hot, warm, cold },
        }
    }

    pub(in crate::data::graph) fn apply_aspect_version(
        &mut self,
        version: AspectVersion,
        regions: &[ChangedRegion],
    ) {
        let (hot, warm) = self.hot_warm();
        super::evaluation_payload::apply_aspect_version(hot, warm, version, regions);
    }

    pub(in crate::data::graph) fn apply_artifact_write(&mut self, delta: ArtifactWriteDelta) {
        let (warm, cold) = match &mut self.target {
            EvaluationMutationTarget::Installed {
                index, warm, cold, ..
            } => (&mut warm[*index], &mut cold[*index]),
            EvaluationMutationTarget::Draft { warm, cold, .. } => (&mut **warm, &mut **cold),
        };
        super::evaluation_payload::apply_artifact_write(warm, cold, delta);
    }

    pub(in crate::data::graph) fn transition_clean(&mut self) {
        let (hot, warm) = self.hot_warm();
        super::evaluation_payload::transition_clean(hot, warm);
    }

    pub(in crate::data::graph) fn set_snapshot_id(
        &mut self,
        snapshot: crate::data::dependency::DependencySnapshotId,
    ) {
        self.hot().dep_snapshot_id = snapshot;
    }

    pub(in crate::data::graph) fn set_causality(
        &mut self,
        causality: Option<crate::data::trace::CausalityMetadata>,
    ) {
        let cold = match &mut self.target {
            EvaluationMutationTarget::Installed { index, cold, .. } => &mut cold[*index],
            EvaluationMutationTarget::Draft { cold, .. } => &mut **cold,
        };
        super::evaluation_payload::set_causality(cold, causality);
    }

    pub(in crate::data::graph) fn stamp_lineage_and_execution(
        &mut self,
        artifact_id: crate::diagnostics::lineage::LineageArtifactId,
        execution_record_id: crate::logic::planner::ExecutionRecordId,
        semantic_segment_id: crate::logic::planner::SemanticSegmentId,
    ) {
        let (warm, cold) = match &mut self.target {
            EvaluationMutationTarget::Installed {
                index, warm, cold, ..
            } => (&mut warm[*index], &mut cold[*index]),
            EvaluationMutationTarget::Draft { warm, cold, .. } => (&mut **warm, &mut **cold),
        };
        if let Some(runtime) = warm.runtime_artifact_state.as_mut() {
            super::evaluation_payload::stamp_lineage_and_execution(
                runtime,
                cold,
                artifact_id,
                execution_record_id,
                semantic_segment_id,
            );
        }
    }

    pub(in crate::data::graph) fn apply_cause_resolution(
        &mut self,
        cause_set: super::super::invalidation_causes::PendingCauseSetId,
        cache: super::PreparedInvalidationCache,
        projected: crate::data::graph::PendingRevalidationNodeProjection,
    ) {
        let (hot, warm) = self.hot_warm();
        hot.pending_cause_set_id = cause_set;
        cache.install_payload(hot, warm);
        super::evaluation_payload::install_revalidation_resolution(hot, warm, projected);
    }

    pub(in crate::data::graph) fn apply_revalidation_resolution(
        &mut self,
        projected: crate::data::graph::PendingRevalidationNodeProjection,
    ) {
        let (hot, warm) = self.hot_warm();
        super::evaluation_payload::install_revalidation_resolution(hot, warm, projected);
    }

    pub(in crate::data::graph) fn set_state(&mut self, state: NodeState) {
        self.hot().state = state;
    }

    fn hot(&mut self) -> &mut NodeHotData {
        match &mut self.target {
            EvaluationMutationTarget::Installed { index, hot, .. } => hot[*index]
                .as_mut()
                .expect("validated live node retains hot storage"),
            EvaluationMutationTarget::Draft { hot, .. } => &mut **hot,
        }
    }

    fn hot_warm(&mut self) -> (&mut NodeHotData, &mut NodeWarmData) {
        match &mut self.target {
            EvaluationMutationTarget::Installed {
                index, hot, warm, ..
            } => (
                hot[*index]
                    .as_mut()
                    .expect("validated live node retains hot storage"),
                &mut warm[*index],
            ),
            EvaluationMutationTarget::Draft { hot, warm, .. } => (&mut **hot, &mut **warm),
        }
    }
}
