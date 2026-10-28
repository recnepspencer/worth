//! Full-history admission and cache publication under the caller's request.
use super::*;

pub(crate) fn ensure_historical_state(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: HistoricalVisibilityBasis,
    recent_candidate: bool,
) -> Result<SnapshotState, crate::execution::RelationalExecutionDenialCause> {
    crate::execution::check_request(execution)?;
    if let Some(root) = basis.root() {
        root.demand_all_full(execution, &runtime.services.symbols)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
    }
    let key = basis_key(&basis);
    if let Some(state) = runtime.visibility.cache.state(&key) {
        runtime
            .services
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return Ok(state);
    }
    runtime
        .services
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
                runtime,
                basis.clone(),
                SnapshotId(0),
                SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
                &mut |units| context.checkpoint(units),
                memory,
            )
        },
        |answer| answer.as_ref().map_or(0, historical_pin_bytes),
    )??;
    insert_state(runtime, state.clone());
    if recent_candidate {
        mark_recent_state_key(runtime, &key);
    }
    Ok(state)
}

pub(crate) fn materialize_historical_visibility(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    version_id: crate::identity::data::VersionId,
    allow_recent_admission: bool,
) -> Result<Option<SnapshotState>, crate::execution::RelationalExecutionDenialCause> {
    crate::execution::check_request(execution)?;
    let Ok(basis) = historical_basis_for_retained_version(runtime, version_id) else {
        return Ok(None);
    };
    if let Some(root) = basis.root() {
        root.demand_all_full(execution, &runtime.services.symbols)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
    }
    let key = basis_key(&basis);
    if let Some(state) = runtime.visibility.cache.state(&key) {
        runtime
            .services
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return Ok(Some(state));
    }
    let recent_candidate = allow_recent_admission
        && runtime.config.visibility.cache_policy.enabled
        && runtime.visibility.cache.recent_window() > 0
        && !is_key_protected(runtime, &key);
    if recent_candidate || is_key_protected(runtime, &key) {
        return ensure_historical_state(execution, runtime, basis, recent_candidate).map(Some);
    }
    runtime
        .services
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
                runtime,
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
