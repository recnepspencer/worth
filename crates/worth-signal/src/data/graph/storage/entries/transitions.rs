use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::{CheckpointNodeImage, NodeEntry, NodeState};
use crate::data::output::PartitionSubscription;
use crate::data::reuse::ReuseBasis;
use crate::data::{aspect::AspectVersion, core_profile::StableHashValue, output::ChangedRegion};

impl SignalGraph {
    pub(crate) fn replace_entry(
        &mut self,
        id: NodeId,
        entry: NodeEntry,
    ) -> Result<(), SignalError> {
        let mut target = self.get_entry_mut(id)?;
        *target = entry;
        drop(target);
        self.record_branch_mutation_state(id);
        Ok(())
    }

    pub(crate) fn replace_entry_from_checkpoint_image(
        &mut self,
        id: NodeId,
        image: CheckpointNodeImage,
    ) -> Result<(), SignalError> {
        self.replace_entry(id, NodeEntry::from_checkpoint_image(image))
    }

    pub(crate) fn node_runtime_artifact_reuse_basis(
        &self,
        node: NodeId,
    ) -> Result<Option<&ReuseBasis>, SignalError> {
        Ok(self
            .warm_ref(node)?
            .runtime_artifact_state
            .as_ref()
            .map(|state| &**state.reuse_basis()))
    }

    pub(crate) fn node_runtime_artifact_structural_state(
        &self,
        node: NodeId,
    ) -> Result<
        (
            Option<crate::diagnostics::lineage::LineageArtifactId>,
            Option<StableHashValue>,
            Option<ReuseBasis>,
        ),
        SignalError,
    > {
        let runtime = self.warm_ref(node)?.runtime_artifact_state.as_ref();
        Ok((
            runtime.and_then(|state| state.lineage_artifact_id().get()),
            runtime.map(crate::data::trace::RuntimeArtifactState::output_hash),
            runtime.map(|state| state.reuse_basis().clone_inner()),
        ))
    }

    pub(crate) fn admit_node_aspect_evaluation_work(
        &self,
        node: NodeId,
        changed_regions: &[ChangedRegion],
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        self.warm_ref(node)?
            .aspect_version_overrides
            .admit_evaluation_work(changed_regions, work)
    }

    pub(crate) fn apply_node_aspect_version(
        &mut self,
        node: NodeId,
        version: AspectVersion,
        changed_regions: &[ChangedRegion],
    ) -> Result<(), SignalError> {
        self.validate_handle(node)?;
        let index = node.index() as usize;
        let hot = self.arena.hot[index]
            .as_mut()
            .expect("validated live node retains hot storage");
        let warm = &mut self.arena.warm[index];
        super::evaluation_payload::apply_aspect_version(hot, warm, version, changed_regions);
        Ok(())
    }

    pub(crate) fn node_partition_version_map(
        &self,
        node: NodeId,
    ) -> Result<crate::data::aspect::PartitionVersionMap, SignalError> {
        let hot = self.hot_ref(node)?;
        let warm = self.warm_ref(node)?;
        Ok(
            crate::data::aspect::PartitionVersionMap::from_storage_parts(
                hot.aspect_version_header,
                warm.aspect_version_overrides.clone(),
            ),
        )
    }

    pub(crate) fn replace_node_partition_version_map(
        &mut self,
        node: NodeId,
        versions: crate::data::aspect::PartitionVersionMap,
    ) -> Result<(), SignalError> {
        let (header, overrides) = versions.into_storage_parts();
        self.hot_mut(node)?.aspect_version_header = header;
        self.warm_mut(node)?.aspect_version_overrides = overrides;
        Ok(())
    }

    pub(crate) fn apply_node_artifact_write_delta(
        &mut self,
        node: NodeId,
        delta: crate::data::trace::ArtifactWriteDelta,
    ) -> Result<bool, SignalError> {
        self.validate_handle(node)?;
        let index = node.index() as usize;
        Ok(super::evaluation_payload::apply_artifact_write(
            &mut self.arena.warm[index],
            &mut self.arena.cold[index],
            delta,
        ))
    }

    pub(crate) fn transition_node_clean(&mut self, node: NodeId) -> Result<(), SignalError> {
        self.release_pending_causes(node)?;
        self.validate_handle(node)?;
        let index = node.index() as usize;
        let hot = self.arena.hot[index]
            .as_mut()
            .expect("validated live node retains hot storage");
        let warm = &mut self.arena.warm[index];
        super::evaluation_payload::transition_clean(hot, warm);
        Ok(())
    }

    pub(crate) fn transition_node_dirty(
        &mut self,
        node: NodeId,
        aspect: crate::data::aspect::Aspect,
        scopes: &[PartitionSubscription],
    ) -> Result<(), SignalError> {
        let hot = self.hot_ref(node)?;
        let invalidates_dependency_causes = hot.pending_cause_set_id
            != crate::data::graph::storage::invalidation_causes::PendingCauseSetId::EMPTY;
        if invalidates_dependency_causes {
            self.release_pending_causes(node)?;
        }
        self.validate_handle(node)?;
        let index = node.index() as usize;
        let hot = self.arena.hot[index]
            .as_mut()
            .expect("validated live node retains hot storage");
        let warm = &mut self.arena.warm[index];
        super::evaluation_payload::transition_dirty(
            hot,
            warm,
            aspect,
            scopes,
            invalidates_dependency_causes,
        );
        Ok(())
    }

    pub(crate) fn set_node_state(
        &mut self,
        node: NodeId,
        state: NodeState,
    ) -> Result<(), SignalError> {
        self.hot_mut(node)?.state = state;
        Ok(())
    }

    pub(crate) fn transition_node_structural_revalidation(
        &mut self,
        node: NodeId,
    ) -> Result<(), SignalError> {
        self.transition_node_revalidation(node, true)
    }

    fn transition_node_revalidation(
        &mut self,
        node: NodeId,
        requires_structural_recompute: bool,
    ) -> Result<(), SignalError> {
        let previous = self
            .pending_dependency_revalidation(node)?
            .map(|pending| pending.unresolved_producers().to_vec())
            .unwrap_or_default();
        let producers = self
            .current_runtime_dependencies_of(node)?
            .iter()
            .filter_map(|edge| {
                (!matches!(self.get_state(edge.source()), Ok(NodeState::Clean)))
                    .then_some(edge.source())
            })
            .collect::<Vec<_>>();
        self.install_node_dependency_revalidation(node, producers, requires_structural_recompute)?;
        let current = self
            .pending_dependency_revalidation(node)?
            .map(|pending| pending.unresolved_producers().to_vec())
            .unwrap_or_default();
        self.replace_pending_revalidation_waiters(node, &previous, &current);
        Ok(())
    }
}
