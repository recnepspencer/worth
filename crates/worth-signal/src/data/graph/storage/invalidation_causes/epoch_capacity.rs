//! Selected cause roots and paths that a checked epoch can detach.
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

/// Read-only selected consumer facts for one prospective producer.
pub(crate) struct ObservedCauseScope {
    pub(crate) consumer: NodeId,
    pub(crate) old_count: usize,
    pub(crate) incoming_count: usize,
    pub(crate) incoming_heap: u64,
    pub(crate) old_scopes: usize,
    pub(crate) incoming_scopes: usize,
}

impl SignalGraph {
    /// Scoped edge payload read from the selected consumers. A new cause owns
    /// the scope in changed scopes, key and binding, then the invalidation
    /// cache copies it once. Previous epoch causes are copied through the
    /// pending reducer and cache on each subsequent producer.
    pub(crate) fn epoch_selected_cause_scope_payload_bound(
        &self,
        producer: NodeId,
        consumers: &[NodeId],
        work: &mut Work,
    ) -> Result<Vec<ObservedCauseScope>, SignalError> {
        let mut payloads = Vec::with_capacity(consumers.len());
        for &consumer in consumers {
            let edges = self.current_runtime_dependencies_of(consumer)?;
            work.reserve_visits(edges.len()).map_err(accounting)?;
            let mut bytes = 0_u64;
            let mut incoming = 0usize;
            let mut scoped_incoming = 0usize;
            for edge in edges.iter().filter(|edge| edge.source() == producer) {
                incoming = incoming.checked_add(1).ok_or_else(overflow)?;
                if let Some(scope) = edge.scope_ref() {
                    scoped_incoming = scoped_incoming.checked_add(1).ok_or_else(overflow)?;
                    bytes = bytes
                        .checked_add(
                            scope
                                .retained_heap_charge(work)
                                .map_err(accounting)?
                                .bytes(),
                        )
                        .ok_or_else(overflow)?;
                }
            }
            let pending = self.pending_causes(consumer)?;
            work.reserve_visits(pending.len()).map_err(accounting)?;
            let old_scopes = pending
                .iter()
                .try_fold(0usize, |count, cause| {
                    count.checked_add(cause.changed_scopes.len())
                })
                .ok_or_else(overflow)?;
            payloads.push(ObservedCauseScope {
                consumer,
                old_count: pending.len(),
                incoming_count: incoming,
                incoming_heap: bytes,
                old_scopes,
                incoming_scopes: scoped_incoming,
            });
        }
        Ok(payloads)
    }

    /// Worst selected cause edits for one prospective producer and its direct
    /// consumers. Existing historical roots remain shared; only selected
    /// cause sets, their page paths and referenced ordinal paths are measured.
    pub(crate) fn epoch_selected_cause_growth_bound(
        &self,
        producer: NodeId,
        consumers: &[NodeId],
        work: &mut Work,
        budget: &mut SignalPreparationBudget,
    ) -> Result<u64, SignalError> {
        let mut old_causes = 0usize;
        let mut empty_slots = 0usize;
        for node in std::iter::once(&producer).chain(consumers.iter()) {
            work.visit().map_err(accounting)?;
            old_causes = old_causes
                .checked_add(self.pending_causes(*node)?.len())
                .ok_or_else(overflow)?;
            empty_slots = empty_slots
                .checked_add(usize::from(
                    self.pending_cause_set_id(*node)?.index.is_none(),
                ))
                .ok_or_else(overflow)?;
        }
        let mark = budget.checkpoint();
        budget.claim_vec::<u64>(old_causes)?;
        let mut old_ordinals = Vec::with_capacity(old_causes);
        let selected_count = consumers.len().checked_add(1).ok_or_else(overflow)?;
        let free_count = self.cause_sets.free_indices.len();
        let reusable = free_count.min(empty_slots);
        let set_capacity = selected_count
            .checked_add(reusable)
            .and_then(|n| n.checked_add(empty_slots))
            .ok_or_else(overflow)?;
        let generation_capacity = selected_count
            .checked_add(reusable)
            .and_then(|n| n.checked_add(empty_slots))
            .ok_or_else(overflow)?;
        let free_capacity = empty_slots
            .checked_add(selected_count)
            .ok_or_else(overflow)?;
        budget.claim_vec::<usize>(set_capacity)?;
        budget.claim_vec::<usize>(generation_capacity)?;
        budget.claim_vec::<usize>(free_capacity)?;
        let mut set_indices = Vec::with_capacity(set_capacity);
        let mut generation_indices = Vec::with_capacity(generation_capacity);
        let mut free_indices = Vec::with_capacity(free_capacity);
        let mut prospective = 0usize;
        let mut old_payloads = Charge::ZERO;
        let slot_count = self.cause_sets.sets.len();
        for node in std::iter::once(&producer).chain(consumers.iter()) {
            let id = self.pending_cause_set_id(*node)?;
            if let Some(index) = id.index {
                let index = index.get() as usize - 1;
                set_indices.push(index);
                generation_indices.push(index);
            }
            let old = self.pending_causes(*node)?;
            work.reserve_visits(old.len()).map_err(accounting)?;
            old_ordinals.extend(
                old.iter()
                    .map(|cause| cause.binding_axes.output_commit_ordinal.0),
            );
            let edge_count = if *node == producer {
                0
            } else {
                let edges = self.current_runtime_dependencies_of(*node)?;
                work.reserve_visits(edges.len()).map_err(accounting)?;
                edges
                    .iter()
                    .filter(|edge| edge.source() == producer)
                    .count()
            };
            prospective = prospective.checked_add(edge_count).ok_or_else(overflow)?;
            let mut old_payload =
                Charge::capacity::<ResolvedDependencyCause>(old.len()).map_err(accounting)?;
            for cause in old {
                old_payload = old_payload
                    .checked_add(cause.retained_heap_charge(work).map_err(accounting)?)
                    .map_err(accounting)?;
            }
            // The reducer copies the pending slice; selected page edits and
            // the invalidation cache may retain the old payload or its scopes
            // alongside that replacement until the epoch is published.
            old_payloads = old_payloads
                .checked_add(old_payload.checked_mul(3).map_err(accounting)?)
                .map_err(accounting)?;
        }
        // A prepared slot may reuse a selected free suffix or append at the
        // successive real indices below. The old single-slot placeholder was
        // not a valid page forecast for several new consumers.
        for offset in 0..reusable {
            work.reserve_visits(self.cause_sets.free_indices.lookup_steps())
                .map_err(accounting)?;
            let index = free_count - 1 - offset;
            let slot = *self
                .cause_sets
                .free_indices
                .get(index)
                .ok_or_else(overflow)? as usize;
            set_indices.push(slot);
            generation_indices.push(slot);
        }
        work.reserve_visits(empty_slots.checked_mul(2).ok_or_else(overflow)?)
            .map_err(accounting)?;
        for offset in 0..empty_slots {
            let index = slot_count.checked_add(offset).ok_or_else(overflow)?;
            set_indices.push(index);
            generation_indices.push(index);
        }
        let free_start = free_count.saturating_sub(empty_slots);
        // A virtual slot may be allocated and released again by successive
        // selected heads, so even initially empty nodes can push the tail.
        let free_end = free_count
            .checked_add(selected_count)
            .ok_or_else(overflow)?;
        work.reserve_visits(free_end - free_start)
            .map_err(accounting)?;
        free_indices.extend(free_start..free_end);
        sort_selected_indices(&mut set_indices, work)?;
        sort_selected_indices(&mut generation_indices, work)?;
        sort_selected_indices(&mut free_indices, work)?;
        let mut growth = old_payloads;
        for vector in [
            self.cause_sets.sets.selected_batch_request_growth_bound(
                &set_indices,
                empty_slots != 0,
                work,
            ),
            self.cause_sets
                .slot_generations
                .selected_batch_request_growth_bound(&generation_indices, empty_slots != 0, work),
            self.cause_sets
                .free_indices
                .selected_batch_request_growth_bound(&free_indices, !free_indices.is_empty(), work),
        ] {
            growth = growth
                .checked_add(vector.map_err(accounting)?)
                .map_err(accounting)?;
        }
        // Rust's unstable sort has O(n log n) worst-case work. Reserve a
        // conservative comparison-layer bound before sorting the observed
        // scalar keys; deduplication is one further linear pass.
        let layers = usize::BITS as usize - old_causes.saturating_sub(1).leading_zeros() as usize;
        let sort_visits = old_causes
            .checked_mul(
                layers
                    .checked_mul(8)
                    .and_then(|n| n.checked_add(1))
                    .ok_or_else(overflow)?,
            )
            .ok_or_else(overflow)?;
        work.reserve_visits(sort_visits).map_err(accounting)?;
        old_ordinals.sort_unstable();
        old_ordinals.dedup();
        let edits = old_causes.checked_add(prospective).ok_or_else(overflow)?;
        // Old causes may refer to different committed ordinals. Every new
        // cause selected for this one producer refers to its same new ordinal.
        // Reference-count edits mutate one draft in sequence: replacing that
        // key's COW path drops the prior draft path. Retain one path per
        // possibly distinct ordinal, including the in-flight edit path.
        let reference_paths = old_ordinals
            .len()
            .checked_add(usize::from(prospective != 0))
            .ok_or_else(overflow)?;
        let reference_path = self
            .cause_sets
            .output_commit_reference_counts
            .selected_edit_growth_bound_with_growth(&0, edits, work)
            .map_err(accounting)?;
        growth = growth
            .checked_add(
                reference_path
                    .checked_mul(reference_paths)
                    .map_err(accounting)?,
            )
            .map_err(accounting)?;
        // An old cause can retire its output commit. Only reachable ordinals
        // from selected old cause sets are inspected, never the whole store.
        for &ordinal in &old_ordinals {
            growth = growth
                .checked_add(
                    self.cause_sets
                        .published_output_commits
                        .selected_edit_growth_bound_with_growth(&ordinal, edits, work)
                        .map_err(accounting)?,
                )
                .map_err(accounting)?;
        }
        let next_ordinal = self
            .cause_sets
            .next_output_commit_ordinal
            .checked_add(1)
            .ok_or_else(overflow)?;
        growth = growth
            .checked_add(
                self.cause_sets
                    .published_output_commits
                    .selected_edit_growth_bound_with_growth(&next_ordinal, edits, work)
                    .map_err(accounting)?,
            )
            .map_err(accounting)?;
        // Prospective inline cause arrays are priced by the admission owner
        // together with prior causes from earlier producers in this epoch.
        drop(old_ordinals);
        budget.release(mark);
        Ok(growth.bytes())
    }
}

fn sort_selected_indices(indices: &mut Vec<usize>, work: &mut Work) -> Result<(), SignalError> {
    let count = indices.len();
    let layers = usize::BITS as usize - count.saturating_sub(1).leading_zeros() as usize;
    let visits = count
        .checked_mul(
            layers
                .checked_mul(8)
                .and_then(|n| n.checked_add(1))
                .ok_or_else(overflow)?,
        )
        .ok_or_else(overflow)?;
    work.reserve_visits(visits).map_err(accounting)?;
    indices.sort_unstable();
    indices.dedup();
    Ok(())
}

fn accounting(_: Denial) -> SignalError {
    SignalError::EvaluationStorageCapacityExhausted
}

fn overflow() -> SignalError {
    SignalError::invalid_input("selected cause growth overflow")
}
