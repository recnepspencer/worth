//! Cumulative selected roots, indexes and subscriber-store shape.
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::retained_storage::btree_structure_charge;

use super::{overflow, ResourceAdmission};

pub(super) struct CandidateShape {
    pub next_width: usize,
    pub owner_shape: u64,
    pub owner_shape_growth: u64,
    pub selected_index: u64,
    pub producer_index: u64,
    pub consumer_index: u64,
    pub waiter_index: u64,
    pub waiter_scratch: u64,
    pub source_index: u64,
    pub basis_growth_capacity: usize,
    pub cause_transition_count: usize,
}

impl CandidateShape {
    pub(super) fn index_scratch(&self) -> Result<u64, SignalError> {
        self.selected_index
            .checked_add(self.source_index)
            .and_then(|bytes| bytes.checked_add(self.consumer_index))
            .and_then(|bytes| bytes.checked_add(self.producer_index))
            .and_then(|bytes| bytes.checked_add(self.waiter_index))
            .ok_or_else(overflow)
    }

    pub(super) fn measure(
        admission: &ResourceAdmission<'_, '_>,
        graph: &SignalGraph,
        producer: NodeId,
        nodes: &[NodeId],
        sources: &[NodeId],
        fanout: &[NodeId],
        new_waiter_count: usize,
    ) -> Result<Self, SignalError> {
        let selected_new = nodes
            .iter()
            .filter(|node| !admission.selected.contains(node))
            .count();
        let next_width = admission.width.checked_add(1).ok_or_else(overflow)?;
        let cause_transition_count = admission
            .cause_transition_count
            .checked_add(fanout.len())
            .ok_or_else(overflow)?;
        let next_selected = admission
            .selected
            .len()
            .checked_add(selected_new)
            .ok_or_else(overflow)?;
        let new_consumers = fanout
            .iter()
            .filter(|consumer| !admission.consumers.contains(consumer))
            .count();
        let next_consumers = admission
            .consumers
            .len()
            .checked_add(new_consumers)
            .ok_or_else(overflow)?;
        let next_waiters = admission
            .waiters
            .len()
            .checked_add(new_waiter_count)
            .ok_or_else(overflow)?;
        // Epoch drafts are allocated once from the deduplicated selected map.
        let selected_vector_capacity = next_selected;
        let new_sources = sources
            .iter()
            .filter(|source| !admission.sources.contains_key(source))
            .count();
        let next_source_count = admission
            .sources
            .len()
            .checked_add(new_sources)
            .ok_or_else(overflow)?;
        let owner_shape = SignalGraph::epoch_publication_shape_bound(next_width)?
            .checked_add(graph.epoch_node_edit_structure_bound(
                next_selected,
                selected_vector_capacity,
                next_width,
            )?)
            .ok_or_else(overflow)?
            .checked_add(SignalGraph::epoch_cause_shape_bound(
                next_width,
                next_consumers,
                next_waiters,
                cause_transition_count,
            )?)
            .ok_or_else(overflow)?
            .checked_add(graph.epoch_cause_store_fork_growth_bound()?)
            .ok_or_else(overflow)?
            .checked_add(graph.epoch_snapshot_store_fork_growth_bound()?)
            .ok_or_else(overflow)?
            .checked_add(
                graph
                    .diagnostics_state()
                    .epoch_preparation_capacity_bound(next_width)?,
            )
            .ok_or_else(overflow)?
            .checked_add(graph.epoch_dependency_batch_structure_capacity_bound(next_width)?)
            .ok_or_else(overflow)?
            .checked_add(graph.epoch_subscriber_batch_structure_capacity_bound(next_source_count)?)
            .ok_or_else(overflow)?;
        let owner_shape_growth = owner_shape
            .checked_sub(admission.owner_shape)
            .ok_or_else(overflow)?;
        let charge = |count| {
            btree_structure_charge::<NodeId, ()>(count)
                .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)
                .map(|charge| charge.bytes())
        };
        let selected_index = charge(selected_new)?;
        let producer_index = charge(usize::from(!admission.producers.contains(&producer)))?;
        let consumer_index = charge(new_consumers)?;
        let waiter_index = charge(new_waiter_count)?;
        let waiter_scratch = btree_structure_charge::<NodeId, ()>(1)
            .and_then(|one| one.checked_mul(2))
            .and_then(|both| both.checked_mul(new_waiter_count))
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?
            .bytes();
        let source_index = btree_structure_charge::<NodeId, (usize, u64)>(new_sources)
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?
            .bytes();
        let basis_growth_capacity =
            if admission.apply_bases.len() == admission.apply_bases.capacity() {
                admission.apply_bases.capacity().saturating_mul(2).max(4)
            } else {
                0
            };
        Ok(Self {
            next_width,
            owner_shape,
            owner_shape_growth,
            selected_index,
            producer_index,
            consumer_index,
            waiter_index,
            waiter_scratch,
            source_index,
            basis_growth_capacity,
            cause_transition_count,
        })
    }
}
