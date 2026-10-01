//! Structural slots and selected payload copy charges for one graph epoch.
use super::{
    semantic_order, EpochNodeChange, EpochPublishedProducer, SelectedNodeDraft, SelectedNodeRole,
};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::retained_storage::{
    arc_allocation_charge, btree_structure_charge, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    SignalConditionalRetentionReservation,
};

impl SignalGraph {
    pub(crate) fn epoch_node_edit_structure_bound(
        &self,
        selected_nodes: usize,
        selected_vector_capacity: usize,
        producers: usize,
    ) -> Result<u64, SignalError> {
        Charge::capacity::<(usize, SelectedNodeRole)>(selected_vector_capacity)
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<EpochPublishedProducer>(producers)?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<(usize, SelectedNodeDraft)>(
                    selected_vector_capacity,
                )?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<semantic_order::PlanSemanticSlot>(
                    producers,
                )?)
            })
            .and_then(|charge| {
                charge.checked_add(btree_structure_charge::<NodeId, EpochNodeChange>(
                    selected_nodes,
                )?)
            })
            .and_then(|charge| {
                if self.arena.retained_node_ledger.is_some() {
                    charge.checked_add(
                        arc_allocation_charge::<SignalConditionalRetentionReservation>()?
                            .checked_mul(selected_nodes)?,
                    )
                } else {
                    Ok(charge)
                }
            })
            .map(Charge::bytes)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }

    /// Full producer draft copy, including any warm companion that may detach.
    pub(crate) fn epoch_node_clone_charge(
        &self,
        node: NodeId,
        work: &mut Work<'_>,
    ) -> Result<Charge, SignalError> {
        self.get_state(node)?;
        let index = node.index() as usize;
        let hot = self.arena.hot[index]
            .as_ref()
            .expect("validated epoch node");
        hot.retained_heap_charge(work)
            .and_then(|charge| {
                charge.checked_add(self.arena.warm[index].retained_heap_charge(work)?)
            })
            .and_then(|charge| {
                charge.checked_add(self.arena.cold[index].retained_heap_charge(work)?)
            })
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }

    /// Cold storage and immutable warm companions stay shared for a consumer.
    pub(crate) fn epoch_consumer_clone_charge(
        &self,
        node: NodeId,
        work: &mut Work<'_>,
    ) -> Result<Charge, SignalError> {
        self.get_state(node)?;
        let warm = &self.arena.warm[node.index() as usize];
        warm.pending_dependency_revalidation
            .retained_heap_charge(work)
            .and_then(|charge| {
                charge.checked_add(
                    warm.dirty_partition_scope_payload
                        .retained_heap_charge(work)?,
                )
            })
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }

    /// The producer draft copies cold ownership and operational warm fields;
    /// warm Arc companions retain their existing allocation until a checked
    /// mutation admits a specific detach.
    pub(crate) fn epoch_producer_draft_clone_charge(
        &self,
        node: NodeId,
        work: &mut Work<'_>,
    ) -> Result<Charge, SignalError> {
        let warm = self.epoch_consumer_clone_charge(node, work)?;
        let cold = self.arena.cold[node.index() as usize]
            .retained_heap_charge(work)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        warm.checked_add(cold)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }
}
