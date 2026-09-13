//! Bounded waiter discovery against the output packet's projected node state.
use super::preparation_work;
use std::collections::BTreeMap;

use crate::data::aspect::AspectMask;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::proof::invalidation::binding::PendingDependencyRevalidation;
use crate::data::retained_storage::{
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial,
};

/// Descriptive future state supplied by the packet's producer/cause owners.
/// This projection cannot authorize installation or substitute for node proof.
#[derive(Debug, Clone)]
pub(crate) struct PendingRevalidationNodeProjection {
    pub(crate) state: NodeState,
    pub(crate) pending: Option<PendingDependencyRevalidation>,
    pub(crate) has_pending_causes: bool,
    pub(crate) dirty_aspects: AspectMask,
    pub(crate) has_direct_basis: bool,
}

#[derive(Debug)]
pub(crate) enum PendingRevalidationPreparationDenial {
    Storage(RetainedStoragePreparationDenial),
    Graph(SignalError),
}

impl From<RetainedStoragePreparationDenial> for PendingRevalidationPreparationDenial {
    fn from(value: RetainedStoragePreparationDenial) -> Self {
        Self::Storage(value)
    }
}

impl From<SignalError> for PendingRevalidationPreparationDenial {
    fn from(value: SignalError) -> Self {
        Self::Graph(value)
    }
}

impl PendingRevalidationPreparationDenial {
    pub(crate) fn into_signal_error(self) -> SignalError {
        match self {
            PendingRevalidationPreparationDenial::Graph(error) => error,
            PendingRevalidationPreparationDenial::Storage(
                RetainedStoragePreparationDenial::WorkExhausted { maximum_visits },
            ) => SignalError::WaiterResolutionWorkExhausted { maximum_visits },
            PendingRevalidationPreparationDenial::Storage(error) => {
                SignalError::internal(format!("waiter resolution accounting failed: {error:?}"))
            }
        }
    }
}

impl PendingRevalidationNodeProjection {
    pub(crate) fn capture(
        graph: &SignalGraph,
        node: NodeId,
        work: &mut Work,
    ) -> Result<Self, PendingRevalidationPreparationDenial> {
        work.visit()?;
        let pending = graph.node_pending_revalidation(node)?;
        if let Some(pending) = pending {
            pending.retained_heap_charge(work)?;
            work.reserve_visits(pending.unresolved_producers().len())?;
        }
        Ok(Self {
            state: graph.get_state(node)?,
            pending: pending.cloned(),
            has_pending_causes: !graph.pending_causes(node)?.is_empty(),
            dirty_aspects: graph.node_dirty_aspects(node)?,
            has_direct_basis: graph.node_direct_invalidation_basis(node)?.is_some(),
        })
    }
}

/// Semantic changes only. The publication owner must still stage, charge, and
/// validate storage for these changes together with every other packet owner.
#[derive(Debug)]
pub(crate) struct PreparedPendingRevalidationResolution {
    pub(crate) initial_waiter_count: usize,
    pub(crate) nodes: BTreeMap<NodeId, PendingRevalidationNodeProjection>,
    pub(crate) buckets: BTreeMap<NodeId, im::OrdSet<NodeId>>,
}

struct ResolutionPreparation<'a> {
    graph: &'a SignalGraph,
    work: &'a mut Work,
    draft: PreparedPendingRevalidationResolution,
}

impl SignalGraph {
    /// Input projections already include the producer transition and direct
    /// consumer replacements. Untouched nodes are loaded from this graph once.
    /// Neither stale-index pruning nor recursive stability propagation writes
    /// live storage. Draft bytes require separate caller custody reservation.
    pub(crate) fn prepare_pending_revalidation_resolution(
        &self,
        producer: NodeId,
        projections: BTreeMap<NodeId, PendingRevalidationNodeProjection>,
        work: &mut Work,
    ) -> Result<PreparedPendingRevalidationResolution, PendingRevalidationPreparationDenial> {
        self.validate_handle(producer)?;
        preparation_work::reserve(
            work,
            projections
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(4)),
        )?;
        for &node in projections.keys() {
            work.visit()?;
            self.validate_handle(node)?;
        }
        let mut preparation = ResolutionPreparation {
            graph: self,
            work,
            draft: PreparedPendingRevalidationResolution {
                initial_waiter_count: 0,
                nodes: projections,
                buckets: BTreeMap::new(),
            },
        };
        // The old outer loop visits initial consumers in ascending order;
        // recursive extensions use the existing ascending-push/LIFO order.
        let initial = preparation.current_waiters(producer)?;
        preparation.draft.initial_waiter_count = initial.len();
        preparation_work::sequence_growth(preparation.work, 0, initial.len())?;
        let mut resolutions = initial
            .into_iter()
            .rev()
            .map(|consumer| (consumer, producer))
            .collect::<Vec<_>>();
        while let Some((consumer, resolved_producer)) = resolutions.pop() {
            preparation.work.visit()?;
            preparation_work::map_lookup(preparation.work, preparation.draft.buckets.len())?;
            let bucket = preparation
                .draft
                .buckets
                .get_mut(&resolved_producer)
                .expect("resolution came from a prepared bucket");
            preparation_work::bucket_edit(preparation.work, bucket.len())?;
            bucket.remove(&consumer);
            if preparation.resolve(consumer, resolved_producer)? {
                let next = preparation.current_waiters(consumer)?;
                preparation_work::sequence_growth(preparation.work, resolutions.len(), next.len())?;
                resolutions.reserve_exact(next.len());
                resolutions.extend(next.into_iter().map(|subscriber| (subscriber, consumer)));
            }
        }
        Ok(preparation.draft)
    }
}

impl ResolutionPreparation<'_> {
    fn load_node(&mut self, node: NodeId) -> Result<(), PendingRevalidationPreparationDenial> {
        preparation_work::map_lookup(self.work, self.draft.nodes.len())?;
        if self.draft.nodes.contains_key(&node) {
            return Ok(());
        }
        preparation_work::map_insert(self.work, self.draft.nodes.len())?;
        self.draft.nodes.insert(
            node,
            PendingRevalidationNodeProjection::capture(self.graph, node, self.work)?,
        );
        Ok(())
    }

    fn current_waiters(
        &mut self,
        producer: NodeId,
    ) -> Result<Vec<NodeId>, PendingRevalidationPreparationDenial> {
        self.work.visit()?;
        preparation_work::map_lookup(self.work, self.draft.buckets.len())?;
        if !self.draft.buckets.contains_key(&producer) {
            preparation_work::reserve(
                self.work,
                Some(
                    4 * self
                        .graph
                        .topology
                        .pending_revalidation_waiters
                        .lookup_steps(),
                ),
            )?;
            let Some(bucket) = self
                .graph
                .topology
                .pending_revalidation_waiters
                .get(&producer)
            else {
                return Ok(Vec::new());
            };
            preparation_work::map_insert(self.work, self.draft.buckets.len())?;
            self.draft.buckets.insert(producer, bucket.clone());
        }
        preparation_work::map_lookup(self.work, self.draft.buckets.len())?;
        let candidates = self.draft.buckets[&producer].clone();
        preparation_work::reserve(
            self.work,
            candidates
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(128)),
        )?;
        preparation_work::sequence_growth(self.work, 0, candidates.len())?;
        let mut current = Vec::with_capacity(candidates.len());
        for &consumer in &candidates {
            self.work.visit()?;
            let waiting = if self.graph.is_alive(consumer) {
                self.load_node(consumer)?;
                preparation_work::map_lookup(self.work, self.draft.nodes.len())?;
                let pending = self.draft.nodes[&consumer].pending.as_ref();
                self.work
                    .reserve_visits(pending.map_or(0, |p| p.unresolved_producers().len()))?;
                pending.is_some_and(|pending| pending.unresolved_producers().contains(&producer))
            } else {
                false
            };
            if waiting {
                current.push(consumer);
            } else {
                preparation_work::map_lookup(self.work, self.draft.buckets.len())?;
                let bucket = self.draft.buckets.get_mut(&producer).unwrap();
                preparation_work::bucket_edit(self.work, bucket.len())?;
                bucket.remove(&consumer);
            }
        }
        Ok(current)
    }

    fn resolve(
        &mut self,
        consumer: NodeId,
        producer: NodeId,
    ) -> Result<bool, PendingRevalidationPreparationDenial> {
        self.load_node(consumer)?;
        preparation_work::map_lookup(self.work, self.draft.nodes.len())?;
        let node = self.draft.nodes.get_mut(&consumer).unwrap();
        let Some(pending) = node.pending.as_mut() else {
            return Ok(false);
        };
        self.work
            .reserve_visits(pending.unresolved_producers().len().checked_mul(2).ok_or(
                RetainedStoragePreparationDenial::WorkExhausted {
                    maximum_visits: self.work.maximum_visits(),
                },
            )?)?;
        pending.resolve_producer(producer);
        if !pending.is_resolved() || pending.requires_structural_recompute() {
            return Ok(false);
        }
        node.pending = None;
        let became_stable = node.state == NodeState::MaybeStale
            && !node.has_pending_causes
            && node.dirty_aspects.is_empty();
        if became_stable {
            node.state = NodeState::Clean;
        }
        Ok(became_stable)
    }
}

#[cfg(test)]
#[path = "resolution_preparation_tests.rs"]
mod tests;
