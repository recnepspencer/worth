use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::graph::signal_graph::{stale_error, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::node::{
    CheckpointNodeImage, CheckpointNodeImageParts, NodeColdData, NodeDefinitionData,
    NodeEvaluationConfig, NodeHotData, NodeState, NodeWarmData,
};
use crate::data::output::PartitionSubscription;
use crate::data::trace::{
    RuntimeArtifactFinalizeImage, RuntimeArtifactHot, RuntimeArtifactOperationalSummary,
    RuntimeArtifactReuseBoundarySnapshot, RuntimeArtifactWarm,
};

/// Persisted invalidation cache facts used to validate a checkpoint before its
/// causes are readmitted. This view is deliberately separate from guarded
/// operational readers: validation must inspect the quarantined bytes before
/// it can prove that they are safe to use.
pub(crate) struct NodeInvalidationConsistencyView {
    dirty_aspects: crate::data::aspect::AspectMask,
    dirty_partition_scopes: Vec<(crate::data::aspect::Aspect, PartitionSubscription)>,
}

impl NodeInvalidationConsistencyView {
    pub(crate) const fn dirty_aspects(&self) -> crate::data::aspect::AspectMask {
        self.dirty_aspects
    }

    pub(crate) fn dirty_partition_scopes(
        &self,
    ) -> &[(crate::data::aspect::Aspect, PartitionSubscription)] {
        &self.dirty_partition_scopes
    }
}

impl SignalGraph {
    pub fn get_state(&self, id: NodeId) -> Result<NodeState, SignalError> {
        self.validate_handle(id)?;
        self.arena
            .hot
            .get(id.index() as usize)
            .and_then(Option::as_ref)
            .map(|hot| hot.state)
            .ok_or_else(|| stale_error(id, id.generation()))
    }

    pub(crate) fn node_dependency_ids(
        &self,
        id: NodeId,
    ) -> Result<
        (
            crate::data::graph::DependencySetId,
            crate::data::dependency::DependencySnapshotId,
        ),
        SignalError,
    > {
        let hot = self.hot_ref(id)?;
        Ok((hot.dependencies_id, hot.dep_snapshot_id))
    }

    pub(crate) fn node_subscribers_id(
        &self,
        id: NodeId,
    ) -> Result<crate::data::graph::SubscriberSetId, SignalError> {
        Ok(self.hot_ref(id)?.subscribers_id)
    }

    pub(crate) fn node_eval_config(
        &self,
        id: NodeId,
    ) -> Result<&NodeEvaluationConfig, SignalError> {
        Ok(&self.definition_ref(id)?.eval_config)
    }

    pub(crate) fn node_invalidation_consistency_view(
        &self,
        id: NodeId,
    ) -> Result<NodeInvalidationConsistencyView, SignalError> {
        let hot = self.hot_ref(id)?;
        let warm = self.warm_ref(id)?;
        Ok(NodeInvalidationConsistencyView {
            dirty_aspects: hot.dirty_aspects,
            dirty_partition_scopes: warm.dirty_partition_scope_payload.to_vec(),
        })
    }

    pub(crate) fn node_dirty_aspects(
        &self,
        id: NodeId,
    ) -> Result<crate::data::aspect::AspectMask, SignalError> {
        self.ensure_cause_readmission_complete()?;
        Ok(self.hot_ref(id)?.dirty_aspects)
    }

    #[cfg(test)]
    pub(crate) fn node_dirty_scoped_aspects(
        &self,
        id: NodeId,
    ) -> Result<Vec<(crate::data::aspect::Aspect, PartitionSubscription)>, SignalError> {
        self.ensure_cause_readmission_complete()?;
        Ok(self.warm_ref(id)?.dirty_partition_scope_payload.to_vec())
    }

    pub(crate) fn node_dirty_partition_scopes_present(
        &self,
        id: NodeId,
    ) -> Result<bool, SignalError> {
        self.ensure_cause_readmission_complete()?;
        Ok(!self.hot_ref(id)?.dirty_partition_scope_aspects.is_empty())
    }

    pub(crate) fn node_state(&self, id: NodeId) -> Result<NodeState, SignalError> {
        Ok(self.hot_ref(id)?.state)
    }

    pub(crate) fn node_runtime_artifact_hot(
        &self,
        id: NodeId,
    ) -> Result<Option<&RuntimeArtifactHot>, SignalError> {
        Ok(self
            .warm_ref(id)?
            .runtime_artifact_state
            .as_ref()
            .map(|state| state.hot()))
    }

    pub(crate) fn node_runtime_artifact_warm(
        &self,
        id: NodeId,
    ) -> Result<Option<&RuntimeArtifactWarm>, SignalError> {
        crate::data::access_counters::note_runtime_artifact_warm_read();
        Ok(self
            .warm_ref(id)?
            .runtime_artifact_state
            .as_ref()
            .map(|state| state.warm()))
    }

    /// Narrow borrowed output projection. Like the owned reuse-boundary
    /// snapshot below, this does not expose the full warm artifact companion.
    pub(crate) fn node_runtime_artifact_output_tokens(
        &self,
        id: NodeId,
    ) -> Result<
        (
            Option<&crate::data::output::OutputIdentity>,
            Option<&crate::data::output::ArtifactContinuityToken>,
        ),
        SignalError,
    > {
        Ok(self
            .warm_ref(id)?
            .runtime_artifact_state
            .as_ref()
            .map_or((None, None), |state| {
                (
                    state.warm().output_identity.as_ref(),
                    state.warm().continuity_token.as_ref(),
                )
            }))
    }

    pub(crate) fn node_runtime_artifact_reuse_boundary_snapshot(
        &self,
        id: NodeId,
    ) -> Result<Option<RuntimeArtifactReuseBoundarySnapshot>, SignalError> {
        Ok(self
            .warm_ref(id)?
            .runtime_artifact_state
            .as_ref()
            .map(crate::data::trace::RuntimeArtifactState::reuse_boundary_snapshot))
    }

    pub(crate) fn node_runtime_artifact_operational_summary(
        &self,
        id: NodeId,
    ) -> Result<Option<RuntimeArtifactOperationalSummary>, SignalError> {
        Ok(self
            .warm_ref(id)?
            .runtime_artifact_state
            .as_ref()
            .map(crate::data::trace::RuntimeArtifactState::operational_summary))
    }

    pub(crate) fn node_runtime_artifact_state(
        &self,
        id: NodeId,
    ) -> Result<Option<&crate::data::trace::RuntimeArtifactState>, SignalError> {
        crate::data::access_counters::note_runtime_artifact_state_read();
        Ok(self.warm_ref(id)?.runtime_artifact_state.as_ref())
    }

    pub(crate) fn node_runtime_artifact_finalize_image(
        &self,
        id: NodeId,
    ) -> Result<Option<RuntimeArtifactFinalizeImage>, SignalError> {
        Ok(self
            .warm_ref(id)?
            .runtime_artifact_state
            .as_ref()
            .map(RuntimeArtifactFinalizeImage::from_runtime_state))
    }

    pub(crate) fn node_runtime_artifact_state_present(
        &self,
        id: NodeId,
    ) -> Result<bool, SignalError> {
        Ok(self.warm_ref(id)?.runtime_artifact_state.is_some())
    }

    pub(crate) fn node_checkpoint_image(
        &self,
        id: NodeId,
    ) -> Result<CheckpointNodeImage, SignalError> {
        let hot = self.hot_ref(id)?;
        let warm = self.warm_ref(id)?;
        let cold = self.cold_ref(id)?;
        Ok(CheckpointNodeImage::from_parts(CheckpointNodeImageParts {
            state: hot.state,
            dirty_aspects: hot.dirty_aspects,
            dirty_partition_scopes: warm.dirty_partition_scope_payload.iter().cloned().collect(),
            aspect_versions: crate::data::aspect::PartitionVersionMap::from_storage_parts(
                hot.aspect_version_header,
                warm.aspect_version_overrides.clone(),
            ),
            dependencies_id: hot.dependencies_id,
            subscribers_id: hot.subscribers_id,
            dep_snapshot_id: hot.dep_snapshot_id,
            pending_cause_set_id: hot.pending_cause_set_id,
            dependency_revision: hot.dependency_revision,
            pending_dependency_revalidation: warm.pending_dependency_revalidation.clone(),
            direct_invalidation_basis: warm.direct_invalidation_basis.clone(),
            direct_invalidation_generation: warm.direct_invalidation_generation,
            tombstoned: self.definition_ref(id)?.tombstoned,
            conditional_contract_generation: self
                .definition_ref(id)?
                .conditional_contract_generation,
            conditional_contract_occurrence: self
                .definition_ref(id)?
                .conditional_contract_occurrence,
            runtime_artifact_state: warm.runtime_artifact_state.clone(),
            retained_artifact: cold.and_then(|cold| cold.retained_artifact.clone()),
            causality: cold.and_then(|cold| cold.causality.clone()),
            execution_trace: cold.and_then(|cold| cold.execution_trace),
            eval_config: self.definition_ref(id)?.eval_config.clone(),
        }))
    }

    pub(crate) fn node_condition(
        &self,
        id: NodeId,
    ) -> Result<crate::data::node::EvaluationCondition, SignalError> {
        Ok(self.definition_ref(id)?.eval_config.condition.clone())
    }

    pub fn node_aspect_version(
        &self,
        id: NodeId,
    ) -> Result<crate::data::aspect::AspectVersion, SignalError> {
        Ok(self.hot_ref(id)?.aspect_version_header.global())
    }

    pub(crate) fn node_partitioned_aspect_version(
        &self,
        id: NodeId,
        scope: &PartitionSubscription,
    ) -> Result<AspectVersion, SignalError> {
        let hot = self.hot_ref(id)?;
        let warm = self.warm_ref(id)?;
        Ok(warm
            .aspect_version_overrides
            .scoped_or_global(scope, hot.aspect_version_header.global()))
    }

    pub(crate) fn node_version_for_scope(
        &self,
        id: NodeId,
        aspect: crate::data::aspect::Aspect,
        scope: Option<&PartitionSubscription>,
    ) -> Result<u64, SignalError> {
        let hot = self.hot_ref(id)?;
        let warm = self.warm_ref(id)?;
        Ok(warm.aspect_version_overrides.version_for_scope(
            aspect,
            scope,
            hot.aspect_version_header.global(),
        ))
    }

    pub(super) fn hot_ref(&self, id: NodeId) -> Result<&NodeHotData, SignalError> {
        self.validate_handle(id)?;
        self.arena
            .hot
            .get(id.index() as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| stale_error(id, id.generation()))
    }

    pub(super) fn definition_ref(&self, id: NodeId) -> Result<&NodeDefinitionData, SignalError> {
        self.validate_handle(id)?;
        self.arena
            .definitions
            .get(id.index() as usize)
            .ok_or_else(|| stale_error(id, id.generation()))
    }

    pub(super) fn warm_ref(&self, id: NodeId) -> Result<&NodeWarmData, SignalError> {
        self.validate_handle(id)?;
        self.arena
            .warm
            .get(id.index() as usize)
            .ok_or_else(|| stale_error(id, id.generation()))
    }

    pub(super) fn cold_ref(&self, id: NodeId) -> Result<Option<&NodeColdData>, SignalError> {
        self.validate_handle(id)?;
        Ok(self
            .arena
            .cold
            .get(id.index() as usize)
            .and_then(|cold| cold.as_deref()))
    }

    pub(super) fn hot_mut(&mut self, id: NodeId) -> Result<&mut NodeHotData, SignalError> {
        self.validate_handle(id)?;
        self.arena
            .hot
            .get_mut(id.index() as usize)
            .and_then(Option::as_mut)
            .ok_or_else(|| stale_error(id, id.generation()))
    }

    pub(super) fn warm_mut(&mut self, id: NodeId) -> Result<&mut NodeWarmData, SignalError> {
        self.validate_handle(id)?;
        self.arena
            .warm
            .get_mut(id.index() as usize)
            .ok_or_else(|| stale_error(id, id.generation()))
    }

    #[cfg(test)]
    pub(super) fn cold_mut(&mut self, id: NodeId) -> Result<&mut NodeColdData, SignalError> {
        self.validate_handle(id)?;
        Ok(self.arena.cold[id.index() as usize]
            .get_or_insert_with(|| Box::new(NodeColdData::default()))
            .as_mut())
    }

    pub(super) fn set_dep_snapshot_id_direct(
        &mut self,
        id: NodeId,
        snapshot_id: crate::data::dependency::DependencySnapshotId,
    ) -> Result<(), SignalError> {
        self.hot_mut(id)?.dep_snapshot_id = snapshot_id;
        Ok(())
    }

    pub(crate) fn set_dependencies_id_direct(
        &mut self,
        id: NodeId,
        dependencies_id: crate::data::graph::DependencySetId,
    ) -> Result<(), SignalError> {
        self.hot_mut(id)?.dependencies_id = dependencies_id;
        Ok(())
    }

    pub(crate) fn set_subscribers_id_direct(
        &mut self,
        id: NodeId,
        subscribers_id: crate::data::graph::SubscriberSetId,
    ) -> Result<(), SignalError> {
        self.hot_mut(id)?.subscribers_id = subscribers_id;
        Ok(())
    }
}
