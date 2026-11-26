//! Full-history admission and cache publication under the caller's request.
use super::*;
use crate::snapshots::data::{SnapshotId, SnapshotReadPolicy};

pub fn ensure_historical_state<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    basis: HistoricalVisibilityBasis,
    recent_candidate: bool,
) -> Result<SnapshotState, crate::execution::denial_cause::RelationalExecutionDenialCause> {
    let inputs = inputs.into();
    crate::execution::request_custody::check_request(execution)?;
    if let Some(root) = basis.root() {
        root.demand_all_full(execution, inputs.symbols)
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
    }
    let key = basis_key(&basis);
    if let Some(state) = inputs.cache.state(&key) {
        inputs
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return Ok(state.within_historical(basis));
    }
    inputs
        .instrumentation
        .count(|counters| counters.visibility_cache_miss_reconstructions += 1);
    let state = crate::visibility::materialization::read_records::read_work::metered_with_memory(
        execution,
        |context, memory| {
            crate::visibility::materialization::read_records::read_work::record_memory::claim(
                memory,
                basis.branch_id().0.len() as u64,
            )?;
            crate::visibility::snapshot_states::build_historical_visibility_state(
                inputs.records,
                inputs.runtime_identity,
                basis.clone(),
                SnapshotId(0),
                SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
                &mut |units| context.checkpoint(units),
                memory,
            )
        },
        |answer| answer.as_ref().map_or(0, historical_pin_bytes),
    )??;
    insert_state(inputs, state.clone());
    if recent_candidate {
        mark_recent_state_key(inputs, &key);
    }
    Ok(state)
}

pub fn materialize_historical_visibility<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    version_id: crate::identity::data::VersionId,
    allow_recent_admission: bool,
) -> Result<Option<SnapshotState>, crate::execution::denial_cause::RelationalExecutionDenialCause> {
    let inputs = inputs.into();
    crate::execution::request_custody::check_request(execution)?;
    let Ok(basis) = historical_basis_for_retained_version(inputs, version_id) else {
        return Ok(None);
    };
    if let Some(root) = basis.root() {
        root.demand_all_full(execution, inputs.symbols)
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
    }
    let key = basis_key(&basis);
    if let Some(state) = inputs.cache.state(&key) {
        inputs
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return Ok(Some(state.within_historical(basis)));
    }
    let recent_candidate = allow_recent_admission
        && inputs.policy.enabled
        && inputs.cache.recent_window() > 0
        && !is_key_protected(inputs, &key);
    if recent_candidate || is_key_protected(inputs, &key) {
        return ensure_historical_state(execution, inputs, basis, recent_candidate).map(Some);
    }
    inputs
        .instrumentation
        .count(|counters| counters.visibility_cache_miss_reconstructions += 1);
    crate::visibility::materialization::read_records::read_work::metered_with_memory(
        execution,
        |context, memory| {
            crate::visibility::materialization::read_records::read_work::record_memory::claim(
                memory,
                basis.branch_id().0.len() as u64,
            )?;
            crate::visibility::snapshot_states::build_historical_visibility_state(
                inputs.records,
                inputs.runtime_identity,
                basis.clone(),
                SnapshotId(0),
                SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
                &mut |units| context.checkpoint(units),
                memory,
            )
        },
        |answer| answer.as_ref().map_or(0, historical_pin_bytes),
    )?
    .map(Some)
}

fn historical_pin_bytes(state: &SnapshotState) -> u64 {
    struct Bytes(u64);
    impl crate::storage::substrate::StorageAllocationVisitor for Bytes {
        fn visit(
            &mut self,
            allocation: crate::storage::substrate::StorageAllocationObservation,
        ) -> bool {
            self.0 = self.0.saturating_add(allocation.bytes);
            true
        }
    }
    let mut bytes = Bytes(0);
    let empty = crate::storage::partition::DenseSlotBitSet::with_capacity(0);
    for pins in state.pinned_partitions.values() {
        pins.entity_slots.visit_new_allocations(&empty, &mut bytes);
        pins.relation_slots
            .visit_new_allocations(&empty, &mut bytes);
        pins.retained_relation_slots
            .visit_new_allocations(&empty, &mut bytes);
    }
    // std B-tree nodes hold at most eleven pairs, twelve edges and a header;
    // twice the entry count bounds even a minimally occupied tree.
    let entries = state.pinned_partitions.len() as u64;
    let node_bytes = 11
        * std::mem::size_of::<(
            crate::identity::data::PartitionId,
            crate::storage::overlay::SnapshotPartitionPins,
        )>() as u64
        + 15 * std::mem::size_of::<usize>() as u64;
    bytes
        .0
        .saturating_add(entries.saturating_mul(2).saturating_mul(node_bytes))
        .saturating_add(state.handle.branch_id.0.capacity() as u64)
        .saturating_add(state.basis.branch_id().0.capacity() as u64)
}
