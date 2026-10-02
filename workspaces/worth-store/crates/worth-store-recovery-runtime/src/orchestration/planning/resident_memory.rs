//! Actual live recovery backing at planning admission boundaries.
//! Owned buffers are counted once; immutable shared history uses pointer identity.

use super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use std::sync::Arc;

pub(super) fn live_bytes(context: &PlanningContext, basis: &ResolvedPlanningBasis) -> Option<u64> {
    let mut bytes = u64::try_from(std::mem::size_of::<PlanningContext>())
        .ok()?
        .checked_add(u64::try_from(std::mem::size_of::<ResolvedPlanningBasis>()).ok()?)?
        .checked_add(context.selection.owned_heap_bytes()?)?
        .checked_add(context.integrity.owned_heap_bytes()?)?
        .checked_add(context.integrity_trace.owned_heap_bytes()?)?
        .checked_add(context.coordination.owner().owned_recovery_heap_bytes()?)?
        .checked_add(vector_bytes(&context.root_protocol_denials)?)?
        .checked_add(
            context
                .root_protocol_denials
                .iter()
                .try_fold(0_u64, |total, denial| {
                    total.checked_add(denial.owned_heap_bytes()?)
                })?,
        )?
        .checked_add(basis.sample.owned_heap_bytes()?)?
        .checked_add(basis.fates.owned_heap_bytes()?)?
        .checked_add(basis.redo.owned_heap_bytes()?)?
        .checked_add(u64::try_from(std::mem::size_of_val(&*basis.targets)).ok()?)?
        .checked_add(basis.observed_pages.selected_source.owned_heap_bytes()?)?
        .checked_add(vector_bytes(&basis.observed_pages.observations)?)?
        .checked_add(vector_bytes(&basis.observed_pages.historical_drops)?)?
        .checked_add(vector_bytes(&basis.verified_drops)?)?
        .checked_add(vector_bytes(&basis.verified_historical_release_operations)?)?
        .checked_add(vector_bytes(&basis.verified_historical_release_sources)?)?;
    if let Some(consumed) = &basis.historical_consumed {
        bytes = bytes.checked_add(consumed.owned_heap_bytes()?)?;
    }
    if let crate::progression::PlanningCustody::SourceHeads(heads) = &basis.custody {
        bytes = bytes.checked_add(heads.owned_heap_bytes()?)?;
    }
    if let crate::progression::PlanningCustody::PendingPrepared { replay, .. } = &basis.custody {
        bytes = bytes.checked_add(replay.owned_heap_bytes()?)?;
    }
    for (index, drop) in basis.observed_pages.historical_drops.iter().enumerate() {
        bytes = bytes
            .checked_add(u64::try_from(std::mem::size_of_val(drop.manifest.dropped())).ok()?)?;
        if let Some(chain) = &drop.chain {
            bytes = bytes.checked_add(chain.owned_heap_bytes()?)?;
        }
        if let Some(history) = &drop.ordered_history {
            let already_counted = basis.observed_pages.historical_drops[..index]
                .iter()
                .filter_map(|prior| prior.ordered_history.as_ref())
                .any(|prior| Arc::ptr_eq(prior, history));
            if !already_counted {
                bytes = bytes
                    .checked_add(history.owned_heap_bytes()?)?
                    .checked_add(u64::try_from(std::mem::size_of_val(&**history)).ok()?)?
                    .checked_add(2 * std::mem::size_of::<usize>() as u64)?;
            }
        }
    }
    if let Some(ordered) = &basis.observed_pages.ordered_releases {
        bytes = bytes.checked_add(vector_bytes(ordered)?)?;
        for released in ordered {
            bytes = bytes
                .checked_add(
                    u64::try_from(std::mem::size_of_val(released.manifest.dropped())).ok()?,
                )?
                .checked_add(released.descriptor_frame.owned_heap_bytes()?)?
                .checked_add(released.reservation_frame.owned_heap_bytes()?)?
                .checked_add(released.manifest_frame.owned_heap_bytes()?)?
                .checked_add(released.head_replay.owned_heap_bytes()?)?;
        }
    }
    bytes = bytes.checked_add(final_custody_bytes(basis)?)?;
    Some(bytes)
}

fn vector_bytes<T>(values: &Vec<T>) -> Option<u64> {
    u64::try_from(values.capacity())
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}

/// Final custody owns distinct control/replay/roster buffers, even when they
/// contain bytes equal to earlier observations. Only shared history is deduped.
fn final_custody_bytes(basis: &ResolvedPlanningBasis) -> Option<u64> {
    let (pending_history, completed_history, mut bytes) = match &basis.custody {
        crate::progression::PlanningCustody::Unresolved
        | crate::progression::PlanningCustody::NoCheckpoint
        | crate::progression::PlanningCustody::NoRelease(_)
        | crate::progression::PlanningCustody::SourceHeads(_) => (None, None, 0),
        crate::progression::PlanningCustody::PendingPrepared { claim, .. } => {
            (claim.ordered_history(), None, claim.owned_heap_bytes()?)
        }
        crate::progression::PlanningCustody::OrderedCompleted {
            claim,
            effective_heads,
        } => (
            None,
            Some(claim.history()),
            claim
                .owned_heap_bytes()?
                .checked_add(effective_heads.owned_heap_bytes()?)?,
        ),
    };
    for (index, history) in [pending_history, completed_history].into_iter().enumerate() {
        let Some(history) = history else { continue };
        let observed = basis
            .observed_pages
            .historical_drops
            .iter()
            .filter_map(|drop| drop.ordered_history.as_deref())
            .any(|other| std::ptr::eq(other, history));
        let same_pending =
            index == 1 && pending_history.is_some_and(|other| std::ptr::eq(other, history));
        if !observed && !same_pending {
            bytes = bytes
                .checked_add(history.owned_heap_bytes()?)?
                .checked_add(u64::try_from(std::mem::size_of_val(history)).ok()?)?
                .checked_add(2 * std::mem::size_of::<usize>() as u64)?;
        }
    }
    Some(bytes)
}
