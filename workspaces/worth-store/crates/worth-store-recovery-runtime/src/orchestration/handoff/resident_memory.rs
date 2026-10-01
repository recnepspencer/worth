//! Live recovery backing retained across the Runtime-to-Store construction call.
//! This measures ownership, not historical I/O or a new admission authority.

use crate::handoff::RecoveryCleanupPosture;
use crate::progression::ReopenedPhysicalRecovery;
use worth_store_physical_integrity::VerifiedCheckpointStream;
use worth_store_recovery_physics::VerifiedOrderedRootHistory;

/// Heap-only retained bytes. The enclosing inline values are reported by
/// `inline_bytes`; Arc-backed checkpoint/history allocations are counted once
/// by pointer identity even when several typed claims refer to them.
pub(super) fn retained_bytes(
    reopened: &ReopenedPhysicalRecovery,
    cleanup: &RecoveryCleanupPosture,
) -> Option<u64> {
    let state = &reopened.state;
    let mut bytes = state
        .selection
        .owned_heap_bytes()?
        .checked_add(state.integrity.owned_heap_bytes()?)?
        .checked_add(state.integrity_trace.owned_heap_bytes()?)?
        .checked_add(state.freshness.owned_heap_bytes()?)?
        .checked_add(state.fates.owned_heap_bytes()?)?
        .checked_add(state.base.owned_heap_bytes()?)?
        .checked_add(reopened.expectation.owned_heap_bytes()?)?
        .checked_add(state.staging_settlements.owned_heap_bytes()?)?
        .checked_add(reopened.publication_settlement.owned_heap_bytes()?)?
        .checked_add(cleanup.owned_heap_bytes()?)?
        .checked_add(state.coordination.owner().owned_recovery_heap_bytes()?)?
        .checked_add(vector_bytes(&state.root_protocol_denials)?)?;
    for denial in &state.root_protocol_denials {
        bytes = bytes.checked_add(denial.owned_heap_bytes()?)?;
    }
    if let Some(fresh) = &reopened.reopened {
        bytes = bytes.checked_add(fresh.owned_heap_bytes()?)?;
    }
    if let Some(claim) = &state.verified_selected_checkpoint_custody {
        bytes = bytes.checked_add(claim.owned_heap_bytes()?)?;
    }
    if let Some(claim) = &state.verified_selected_head_custody_v2 {
        bytes = bytes.checked_add(claim.owned_heap_bytes()?)?;
    }
    if let Some(claim) = &state.verified_pending_wal_release_custody {
        bytes = bytes.checked_add(claim.owned_heap_bytes()?)?;
    }
    if let Some(claim) = &state.verified_ordered_historical_release_custody {
        bytes = bytes.checked_add(claim.owned_heap_bytes()?)?;
    }
    if let Some(heads) = &state.verified_effective_release_heads_v14 {
        bytes = bytes.checked_add(heads.owned_heap_bytes()?)?;
    }
    bytes = bytes.checked_add(shared_checkpoint_bytes(reopened)?)?;
    bytes.checked_add(shared_history_bytes(reopened)?)
}

/// Inline footprint is separate from the heap measure and must be added only
/// once by the enclosing owner, not again for each nested typed field.
pub(super) fn inline_bytes(
    _: &ReopenedPhysicalRecovery,
    _: &RecoveryCleanupPosture,
) -> Option<u64> {
    u64::try_from(std::mem::size_of::<ReopenedPhysicalRecovery>())
        .ok()?
        .checked_add(u64::try_from(std::mem::size_of::<RecoveryCleanupPosture>()).ok()?)
}

fn vector_bytes<T>(values: &Vec<T>) -> Option<u64> {
    u64::try_from(values.capacity())
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}

fn shared_checkpoint_bytes(reopened: &ReopenedPhysicalRecovery) -> Option<u64> {
    let state = &reopened.state;
    let checkpoints: [Option<&VerifiedCheckpointStream>; 11] = [
        state.selection.checkpoint().map(|base| base.checkpoint()),
        state
            .verified_selected_checkpoint_custody
            .as_ref()
            .map(|claim| claim.checkpoint()),
        state
            .verified_selected_head_custody_v2
            .as_ref()
            .map(|claim| claim.checkpoint()),
        state
            .verified_selected_no_release_custody
            .as_ref()
            .map(|claim| claim.checkpoint()),
        state
            .verified_pending_wal_release_custody
            .as_ref()
            .map(|claim| claim.checkpoint()),
        state
            .verified_ordered_historical_release_custody
            .as_ref()
            .map(|claim| claim.checkpoint()),
        state
            .verified_selected_tier_custody
            .as_ref()
            .map(|claim| claim.checkpoint()),
        state
            .verified_pending_wal_release_custody
            .as_ref()
            .and_then(|claim| claim.selected_release())
            .map(|base| base.checkpoint()),
        state
            .verified_pending_wal_release_custody
            .as_ref()
            .and_then(|claim| claim.addressed_release_base())
            .map(|base| base.checkpoint()),
        state
            .verified_pending_wal_release_custody
            .as_ref()
            .and_then(|claim| claim.selected_head_v2())
            .map(|base| base.checkpoint()),
        state
            .verified_ordered_historical_release_custody
            .as_ref()
            .and_then(|claim| claim.selected_head_v2())
            .map(|base| base.checkpoint()),
    ];
    let mut bytes = 0_u64;
    for (index, checkpoint) in checkpoints.into_iter().enumerate() {
        let Some(checkpoint) = checkpoint else {
            continue;
        };
        if checkpoints[..index]
            .iter()
            .flatten()
            .any(|prior| std::ptr::eq(*prior, checkpoint))
        {
            continue;
        }
        bytes = bytes
            .checked_add(checkpoint.owned_heap_bytes()?)?
            .checked_add(u64::try_from(std::mem::size_of_val(checkpoint)).ok()?)?
            .checked_add(u64::try_from(2 * std::mem::size_of::<usize>()).ok()?)?;
    }
    Some(bytes)
}

fn shared_history_bytes(reopened: &ReopenedPhysicalRecovery) -> Option<u64> {
    let state = &reopened.state;
    let histories: [Option<&VerifiedOrderedRootHistory>; 2] = [
        state
            .verified_pending_wal_release_custody
            .as_ref()
            .and_then(|claim| claim.ordered_history()),
        state
            .verified_ordered_historical_release_custody
            .as_ref()
            .map(|claim| claim.history()),
    ];
    let mut bytes = 0_u64;
    for (index, history) in histories.into_iter().enumerate() {
        let Some(history) = history else { continue };
        if histories[..index]
            .iter()
            .flatten()
            .any(|prior| std::ptr::eq(*prior, history))
        {
            continue;
        }
        bytes = bytes
            .checked_add(history.owned_heap_bytes()?)?
            .checked_add(u64::try_from(std::mem::size_of_val(history)).ok()?)?
            .checked_add(u64::try_from(2 * std::mem::size_of::<usize>()).ok()?)?;
    }
    Some(bytes)
}
