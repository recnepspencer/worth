//! Physical hot, warm and producer-cold root growth for one selected epoch.
use std::collections::BTreeSet;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::persistent_paged_vector::PersistentPagedVector;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl SignalGraph {
    pub(super) fn admit_ordinary_epoch_node_writes(
        &self,
        selected: usize,
        producers: usize,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let visits = [
            self.arena.hot.indexed_mutation_work_bound(selected),
            self.arena.warm.indexed_mutation_work_bound(selected),
            self.arena.cold.indexed_mutation_work_bound(producers),
        ]
        .into_iter()
        .try_fold(0_usize, |total, lane| total.checked_add(lane?))
        .ok_or_else(overflow)?;
        work.reserve_visits(visits)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }

    pub(crate) fn epoch_selected_node_root_growth_bound(
        &self,
        selected: &BTreeSet<NodeId>,
        candidate: &[NodeId],
        producers: &BTreeSet<NodeId>,
        next_producer: NodeId,
        work: &mut Work,
        budget: &mut SignalPreparationBudget,
    ) -> Result<u64, SignalError> {
        let selected_capacity = selected
            .len()
            .checked_add(candidate.len())
            .ok_or_else(overflow)?;
        let producer_capacity = producers.len().checked_add(1).ok_or_else(overflow)?;
        budget.claim_vec::<usize>(selected_capacity)?;
        budget.claim_vec::<usize>(producer_capacity)?;
        let sort_visits = selected_capacity
            .checked_mul(usize::BITS as usize)
            .and_then(|visits| {
                visits.checked_add(producer_capacity.checked_mul(usize::BITS as usize)?)
            })
            .ok_or_else(overflow)?;
        work.reserve_visits(sort_visits)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        let mut selected_indices = Vec::with_capacity(selected_capacity);
        selected_indices.extend(selected.iter().map(|node| node.index() as usize));
        selected_indices.extend(candidate.iter().map(|node| node.index() as usize));
        selected_indices.sort_unstable();
        selected_indices.dedup();
        let mut producer_indices = Vec::with_capacity(producer_capacity);
        producer_indices.extend(producers.iter().map(|node| node.index() as usize));
        producer_indices.push(next_producer.index() as usize);
        producer_indices.sort_unstable();
        producer_indices.dedup();

        let retained = self.arena.retained_node_ledger.is_some();
        let hot = lane_growth(
            &self.arena.hot,
            &selected_indices,
            retained,
            work,
            |_, _| Ok(Charge::ZERO),
        )?;
        let warm = lane_growth(
            &self.arena.warm,
            &selected_indices,
            retained,
            work,
            |warm, work| {
                warm.pending_dependency_revalidation
                    .retained_heap_charge(work)?
                    .checked_add(
                        warm.dirty_partition_scope_payload
                            .retained_heap_charge(work)?,
                    )
            },
        )?;
        let cold = lane_growth(
            &self.arena.cold,
            &producer_indices,
            retained,
            work,
            |cold, work| cold.retained_heap_charge(work),
        )?;
        hot.checked_add(warm)
            .and_then(|charge| charge.checked_add(cold))
            .map(Charge::bytes)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }
}

fn lane_growth<T: Clone + RetainedStorageMeasurement>(
    lane: &PersistentPagedVector<T>,
    indices: &[usize],
    retained: bool,
    work: &mut Work,
    payload_growth: impl FnMut(&T, &mut Work) -> Result<Charge, Denial>,
) -> Result<Charge, SignalError> {
    let measured = if retained {
        lane.selected_batch_request_growth_bound_with_payload(indices, false, work, payload_growth)
    } else {
        lane.selected_direct_replacement_request_growth_bound(indices, work, payload_growth)
    };
    measured.map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
}

fn overflow() -> SignalError {
    SignalError::invalid_input("selected node root growth overflow")
}
