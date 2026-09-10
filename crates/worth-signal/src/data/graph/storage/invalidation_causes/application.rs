use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::{DependencyRevision, ResolvedDependencyCause};
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;

use crate::data::graph::SignalGraph;

impl SignalGraph {
    pub(crate) fn begin_retained_cause_store_publication(
        &self,
        ledger: &std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionLedger>,
        maximum: crate::data::retained_storage::RetainedStorageCharge,
    ) -> Result<super::RetainedCauseStorePublicationDraft, SignalError> {
        self.cause_sets.begin_retained_publication(ledger, maximum)
    }

    pub(crate) fn install_retained_cause_store_publication(
        &mut self,
        prepared: super::PreparedRetainedCauseStorePublication,
    ) {
        prepared.install(&mut self.cause_sets);
    }

    #[cfg(test)]
    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    pub(crate) fn published_output_commit_order_for_test(&self) -> Vec<(u64, NodeId)> {
        self.cause_sets
            .published_order_probe
            .iter()
            .copied()
            .collect()
    }

    pub(crate) fn dependency_revision(
        &self,
        node: NodeId,
    ) -> Result<DependencyRevision, SignalError> {
        self.node_dependency_revision(node)
    }

    pub(crate) fn pending_causes(
        &self,
        node: NodeId,
    ) -> Result<&[ResolvedDependencyCause], SignalError> {
        self.ensure_cause_readmission_complete()?;
        let id = self.node_pending_cause_set_id(node)?;
        self.cause_sets.get(id)
    }

    pub(crate) fn pending_cause_set_id(
        &self,
        node: NodeId,
    ) -> Result<PendingCauseSetId, SignalError> {
        self.ensure_cause_readmission_complete()?;
        self.node_pending_cause_set_id(node)
    }

    pub(crate) fn replace_pending_causes(
        &mut self,
        node: NodeId,
        causes: impl IntoIterator<Item = ResolvedDependencyCause>,
    ) -> Result<PendingCauseSetId, SignalError> {
        let revision = self.dependency_revision(node)?;
        let graph_instance = self.runtime_instance_id();
        let causes = causes.into_iter().collect::<Vec<_>>();
        self.validate_pending_causes(node, &causes)?;
        for cause in &causes {
            if cause.key.graph_instance != graph_instance
                || cause.key.consumer != node
                || cause.key.dependency_revision != revision
                || cause.binding_axes.graph_instance != graph_instance
                || cause.binding_axes.consumer != node
                || cause.binding_axes.dependency_revision != revision
            {
                return Err(SignalError::invalid_input(
                    "pending dependency cause is stale or bound to another consumer",
                ));
            }
        }
        let current = self.node_pending_cause_set_id(node)?;
        let id = self.cause_sets.replace_set(current, causes)?;
        self.set_node_pending_cause_set_id(node, id)?;
        self.rebuild_dirty_caches_from_pending_causes(node)?;
        self.node_pending_cause_set_id(node)
    }

    pub(crate) fn publish_prepared_cause_storage(
        &mut self,
        node: NodeId,
        causes: super::NormalizedCauseSet,
        delta: Option<&ProducedAspectDelta>,
        slot: super::PreparedCauseSlot,
    ) -> Result<PendingCauseSetId, SignalError> {
        if let Some(delta) = delta {
            self.validate_prepared_pending_causes(node, &causes, delta)?;
        } else {
            self.validate_pending_causes(node, &causes)?;
        }
        let current = self.node_pending_cause_set_id(node)?;
        if current != slot.handle() {
            return Err(SignalError::invalid_input(
                "prepared cause handle was not installed by node publication",
            ));
        }
        let id = slot.handle();
        self.cause_sets.publish_prepared_cause_slot(slot, causes)?;
        Ok(id)
    }

    pub(crate) fn merge_pending_causes(
        &mut self,
        node: NodeId,
        causes: impl IntoIterator<Item = ResolvedDependencyCause>,
    ) -> Result<PendingCauseSetId, SignalError> {
        let revision = self.dependency_revision(node)?;
        let graph_instance = self.runtime_instance_id();
        let causes = causes.into_iter().collect::<Vec<_>>();
        self.validate_pending_causes(node, &causes)?;
        for cause in &causes {
            if cause.key.graph_instance != graph_instance
                || cause.key.consumer != node
                || cause.key.dependency_revision != revision
            {
                return Err(SignalError::invalid_input(
                    "pending dependency cause is stale or bound to another consumer",
                ));
            }
        }
        let current = self.node_pending_cause_set_id(node)?;
        let id = self.cause_sets.replace_set(current, causes)?;
        self.set_node_pending_cause_set_id(node, id)?;
        self.rebuild_dirty_caches_from_pending_causes(node)?;
        self.node_pending_cause_set_id(node)
    }

    pub(crate) fn rebuild_dirty_caches_from_pending_causes(
        &mut self,
        node: NodeId,
    ) -> Result<(), SignalError> {
        let causes = self.pending_causes(node)?.to_vec();
        let direct = self.node_direct_invalidation_basis(node)?.cloned();
        let mut dirty_aspects = crate::data::aspect::AspectMask::EMPTY;
        if let Some(direct) = &direct {
            dirty_aspects = direct.dirty_aspects();
        }
        for cause in &causes {
            dirty_aspects.insert(cause.key.aspect);
        }
        let direct_scopes = direct.iter().flat_map(|basis| {
            basis
                .scoped_aspects()
                .iter()
                .map(|(aspect, scope)| (*aspect, scope.clone()))
        });
        let cause_scopes = causes.iter().flat_map(|cause| {
            cause
                .changed_scopes
                .as_slice()
                .iter()
                .cloned()
                .map(|scope| (cause.key.aspect, scope))
        });
        self.replace_node_invalidation_cache(node, dirty_aspects, direct_scopes.chain(cause_scopes))
    }

    pub(crate) fn release_pending_causes(&mut self, node: NodeId) -> Result<(), SignalError> {
        let current = self.node_pending_cause_set_id(node)?;
        if current == PendingCauseSetId::EMPTY {
            return Ok(());
        }
        self.cause_sets.release(current)?;
        self.set_node_pending_cause_set_id(node, PendingCauseSetId::EMPTY)?;
        self.replace_node_invalidation_cache(
            node,
            crate::data::aspect::AspectMask::EMPTY,
            std::iter::empty(),
        )?;
        Ok(())
    }
}
