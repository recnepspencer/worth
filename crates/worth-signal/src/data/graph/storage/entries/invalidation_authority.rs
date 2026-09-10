use crate::data::aspect::{Aspect, AspectMask};
use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, PendingDependencyRevalidation,
};
use crate::data::proof::invalidation::source_seed::DirectInvalidationBasis;

impl SignalGraph {
    pub(crate) fn admit_pending_cause_handle_reads(
        &self,
        count: usize,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(
            self.arena
                .nodes
                .lookup_steps()
                .checked_add(self.arena.hot.lookup_steps())
                .and_then(|n| n.checked_add(4))
                .and_then(|n| n.checked_mul(count)),
        )
    }

    pub(crate) fn node_dependency_revision(
        &self,
        node: NodeId,
    ) -> Result<DependencyRevision, SignalError> {
        Ok(self.hot_ref(node)?.dependency_revision)
    }

    pub(crate) fn node_pending_cause_set_id(
        &self,
        node: NodeId,
    ) -> Result<PendingCauseSetId, SignalError> {
        Ok(self.hot_ref(node)?.pending_cause_set_id)
    }

    pub(crate) fn node_direct_invalidation_basis(
        &self,
        node: NodeId,
    ) -> Result<Option<&DirectInvalidationBasis>, SignalError> {
        Ok(self.warm_ref(node)?.direct_invalidation_basis.as_ref())
    }

    pub(crate) fn node_direct_invalidation_generation(
        &self,
        node: NodeId,
    ) -> Result<u64, SignalError> {
        Ok(self.warm_ref(node)?.direct_invalidation_generation)
    }

    pub(crate) fn node_dirty_partition_scope_payload(
        &self,
        node: NodeId,
    ) -> Result<&[(Aspect, PartitionSubscription)], SignalError> {
        Ok(self
            .warm_ref(node)?
            .dirty_partition_scope_payload
            .as_slice())
    }

    pub(crate) fn node_pending_revalidation(
        &self,
        node: NodeId,
    ) -> Result<Option<&PendingDependencyRevalidation>, SignalError> {
        Ok(self
            .warm_ref(node)?
            .pending_dependency_revalidation
            .as_ref())
    }

    pub(crate) fn set_node_pending_cause_set_id(
        &mut self,
        node: NodeId,
        id: PendingCauseSetId,
    ) -> Result<(), SignalError> {
        self.hot_mut(node)?.pending_cause_set_id = id;
        Ok(())
    }

    pub(crate) fn advance_node_dependency_revision(
        &mut self,
        node: NodeId,
    ) -> Result<(), SignalError> {
        let invalidates_dependency_causes = {
            let hot = self.hot_mut(node)?;
            let invalidates = hot.pending_cause_set_id != PendingCauseSetId::EMPTY;
            hot.dependency_revision.0 = hot
                .dependency_revision
                .0
                .checked_add(1)
                .expect("dependency revision overflow");
            hot.pending_cause_set_id = PendingCauseSetId::EMPTY;
            if invalidates {
                hot.state = crate::data::node::NodeState::MaybeStale;
                hot.dirty_aspects = AspectMask::EMPTY;
                hot.dirty_partition_scope_aspects = AspectMask::EMPTY;
            }
            invalidates
        };
        let warm = self.warm_mut(node)?;
        warm.pending_dependency_revalidation = None;
        if invalidates_dependency_causes {
            warm.dirty_partition_scope_payload.clear();
        }
        Ok(())
    }

    pub(crate) fn replace_node_invalidation_cache(
        &mut self,
        node: NodeId,
        dirty_aspects: AspectMask,
        scopes: impl IntoIterator<Item = (Aspect, PartitionSubscription)>,
    ) -> Result<(), SignalError> {
        let scope_aspects = {
            let payload = &mut self.warm_mut(node)?.dirty_partition_scope_payload;
            payload.clear();
            for (aspect, scope) in scopes {
                if !payload.iter().any(|(candidate_aspect, candidate_scope)| {
                    candidate_aspect == &aspect && candidate_scope == &scope
                }) {
                    payload.push((aspect, scope));
                }
            }
            payload.sort_unstable();
            let mut aspects = AspectMask::EMPTY;
            for (aspect, _) in payload.iter() {
                aspects.insert(*aspect);
            }
            aspects
        };
        let hot = self.hot_mut(node)?;
        hot.dirty_aspects = dirty_aspects;
        hot.dirty_partition_scope_aspects = scope_aspects;
        Ok(())
    }

    pub(crate) fn install_node_dependency_revalidation(
        &mut self,
        node: NodeId,
        producers: impl IntoIterator<Item = NodeId>,
        requires_structural_recompute: bool,
    ) -> Result<(), SignalError> {
        let hot = self.hot_mut(node)?;
        let revision = hot.dependency_revision;
        if matches!(hot.state, crate::data::node::NodeState::Clean) {
            hot.state = crate::data::node::NodeState::MaybeStale;
        }
        self.warm_mut(node)?.pending_dependency_revalidation =
            Some(if requires_structural_recompute {
                PendingDependencyRevalidation::structural(revision, producers)
            } else {
                PendingDependencyRevalidation::new(revision, producers)
            });
        Ok(())
    }

    pub(crate) fn publish_node_revalidation_resolution(
        &mut self,
        node: NodeId,
        projected: crate::data::graph::PendingRevalidationNodeProjection,
    ) -> Result<(), SignalError> {
        self.node_evaluation_mutation(node)?
            .apply_revalidation_resolution(projected);
        Ok(())
    }

    pub(crate) fn publish_node_cause_resolution(
        &mut self,
        node: NodeId,
        cause_set: PendingCauseSetId,
        cache: super::PreparedInvalidationCache,
        projected: crate::data::graph::PendingRevalidationNodeProjection,
    ) -> Result<(), SignalError> {
        self.node_evaluation_mutation(node)?
            .apply_cause_resolution(cause_set, cache, projected);
        Ok(())
    }
}
