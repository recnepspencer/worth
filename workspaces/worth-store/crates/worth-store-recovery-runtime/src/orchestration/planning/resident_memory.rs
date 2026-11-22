//! Actual live recovery backing at planning admission boundaries.
//! Owned buffers are counted once; immutable shared history uses pointer identity.

use super::super::{RecoveryCoordination, ResidentSourceSelection};
use super::{context::PlanningContext, resolved_basis::ResolvedPlanningState};
use crate::entry::PhysicalRecoverySourceDenial;
use crate::progression::RecoveryIntegrityEvidence;
use std::sync::Arc;

pub(super) fn selected_live_bytes(
    coordination: &RecoveryCoordination,
    selection: &ResidentSourceSelection,
    integrity: &RecoveryIntegrityEvidence,
    root_protocol_denials: &Vec<PhysicalRecoverySourceDenial>,
    integrity_trace: &crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Option<u64> {
    u64::try_from(std::mem::size_of::<PlanningContext>())
        .ok()?
        .checked_add(selection.owned_heap_bytes()?)?
        .checked_add(integrity.owned_heap_bytes()?)?
        .checked_add(integrity_trace.owned_heap_bytes()?)?
        .checked_add(coordination.owner().owned_recovery_heap_bytes()?)?
        .checked_add(vector_bytes(root_protocol_denials)?)?
        .checked_add(
            root_protocol_denials
                .iter()
                .try_fold(0_u64, |total, denial| {
                    total.checked_add(denial.owned_heap_bytes()?)
                })?,
        )
}

pub(super) fn context_live_bytes(context: &PlanningContext) -> Option<u64> {
    selected_live_bytes(
        &context.coordination,
        &context.selection,
        &context.integrity,
        &context.root_protocol_denials,
        &context.integrity_trace,
    )?
    .checked_add(u64::try_from(std::mem::size_of_val(&**context)).ok()?)
}

pub(super) fn live_bytes(context: &PlanningContext, basis: &ResolvedPlanningState) -> Option<u64> {
    let mut bytes = context_live_bytes(context)?
        .checked_add(u64::try_from(std::mem::size_of::<ResolvedPlanningState>()).ok()?)?
        .checked_add(basis.unused_backing_bytes())?
        .checked_add(std::mem::size_of::<super::resolved_basis::ResolvedPlanningBasis>() as u64)?
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
    match &basis.custody {
        crate::progression::PlanningCustody::Unresolved
        | crate::progression::PlanningCustody::NoCheckpoint => {}
        crate::progression::PlanningCustody::NoRelease(no_release) => {
            bytes = bytes.checked_add(u64::try_from(std::mem::size_of_val(&**no_release)).ok()?)?;
        }
        crate::progression::PlanningCustody::SourceHeads(heads) => {
            bytes = bytes
                .checked_add(u64::try_from(std::mem::size_of_val(&**heads)).ok()?)?
                .checked_add(heads.owned_heap_bytes()?)?;
        }
        crate::progression::PlanningCustody::PendingPrepared { claim, replay } => {
            bytes = bytes
                .checked_add(u64::try_from(std::mem::size_of_val(&**claim)).ok()?)?
                .checked_add(u64::try_from(std::mem::size_of_val(&**replay)).ok()?)?
                .checked_add(replay.owned_heap_bytes()?)?;
        }
        crate::progression::PlanningCustody::OrderedCompleted {
            claim,
            effective_heads,
        } => {
            bytes = bytes
                .checked_add(u64::try_from(std::mem::size_of_val(&**claim)).ok()?)?
                .checked_add(u64::try_from(std::mem::size_of_val(&**effective_heads)).ok()?)?;
        }
    }
    for (index, drop) in basis.observed_pages.historical_drops.iter().enumerate() {
        bytes = bytes
            .checked_add(u64::try_from(std::mem::size_of_val(drop.manifest.dropped())).ok()?)?;
        let history = &drop.ordered_history;
        let already_counted = basis.observed_pages.historical_drops[..index]
            .iter()
            .any(|prior| Arc::ptr_eq(&prior.ordered_history, history));
        if !already_counted {
            bytes = bytes
                .checked_add(history.owned_heap_bytes()?)?
                .checked_add(u64::try_from(std::mem::size_of_val(&**history)).ok()?)?
                .checked_add(2 * std::mem::size_of::<usize>() as u64)?;
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
fn final_custody_bytes(basis: &ResolvedPlanningState) -> Option<u64> {
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
            .map(|drop| &*drop.ordered_history)
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
