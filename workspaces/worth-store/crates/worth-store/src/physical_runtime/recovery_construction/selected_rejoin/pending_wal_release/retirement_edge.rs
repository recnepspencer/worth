//! Store's rejoin of the per-edge basis decision and of a checkpoint-covered
//! retirement edge. The decision is the same Physics function recovery
//! planning used, fed with Store's own sampled release intents; the edge is
//! then rechecked over Store's own reads of both addressed roots.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::PhysicalRecordFormatDeclaration;
use worth_store_recovery_physics::{
    decide_ordered_root_step_basis, is_retirement_prefix, CheckpointRetiredReleaseIntent,
    OrderedRootStepBasis, ReleasedInventoryView, RetirementReleaseIntent, VerifiedOrderedRootEdge,
    VerifiedRetirementRootEdge,
};

use super::super::{
    completed_history::{
        native_storage::HistoricalWalkStorage, CompletedHistoryHeadFold, CompletedHistoryHeadStep,
    },
    control_frames::{FundedCompletedHistoricalRawSlices, SelectedControlMediaFingerprint},
    resident::StoreRejoinResidentLedger,
    SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use super::{addressed_root, delta};
use crate::physical_runtime::{
    PhysicalRecoveryReadAllocation, StoreRecoveryBindingFreshnessSample,
};

/// Every edge must carry the basis Store's own evidence decides for it: a
/// member edge where no checkpoint-covered intent competes, and a retirement
/// edge only for the exact intent Store sampled at or below the cutoff, after
/// Store's own reread of the sampled members finds none leaving the source.
/// A member edge's member is matched by its own edge rewalk.
pub(super) fn verify_basis(
    edges: &[VerifiedOrderedRootEdge],
    edge_index: usize,
    source_generation: u64,
    cutoff: u64,
    sample: &StoreRecoveryBindingFreshnessSample,
    member_leaves_source: impl FnOnce() -> Result<bool, Denial>,
) -> Result<(), Denial> {
    let (prior, edge) = edges.split_at(edge_index);
    let planned = match edge.first().ok_or(Denial::CertificateRoster)? {
        VerifiedOrderedRootEdge::Retirement(edge) => Some(edge.basis()),
        VerifiedOrderedRootEdge::Ordinary(_) | VerifiedOrderedRootEdge::Released(_) => None,
    };
    check_basis(
        planned,
        is_retirement_prefix(prior),
        source_generation,
        cutoff,
        sample.release_intents(),
        member_leaves_source,
    )
}

fn check_basis(
    planned: Option<CheckpointRetiredReleaseIntent>,
    retirement_prefix: bool,
    source_generation: u64,
    cutoff: u64,
    intents: &[RetirementReleaseIntent],
    member_leaves_source: impl FnOnce() -> Result<bool, Denial>,
) -> Result<(), Denial> {
    let member_at_source = match planned {
        Some(_) => member_leaves_source()?,
        None => true,
    };
    let decided = decide_ordered_root_step_basis(
        source_generation,
        retirement_prefix,
        cutoff,
        member_at_source,
        intents,
    )
    .map_err(|_| Denial::WalFate)?;
    match (decided, planned) {
        (OrderedRootStepBasis::WalMember, None) => Ok(()),
        (OrderedRootStepBasis::RetirementIntent(decided), Some(planned)) if decided == planned => {
            Ok(())
        }
        _ => Err(Denial::WalFate),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    media: AdmittedRecoveryFilesystemMedia,
    edge: &VerifiedRetirementRootEdge,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
    retained_peak_bytes: u64,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let maximum_fingerprint = delta::MAX_TRANSITION_MEMORY
        .checked_sub(retained_peak_bytes)
        .ok_or(Denial::BoundExceeded)?;
    let mut remaining = maximum_fingerprint
        .checked_sub(super::historical_chain::DISCOVERY_HEADROOM)
        .ok_or(Denial::BoundExceeded)?;
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let (source, result) = observe_roots(&mut discovery, edge, generation, format, node_capacity)?;
    let source_snapshot = delta::snapshot(
        &mut discovery,
        &source.root,
        &source.free,
        format,
        &mut remaining,
    )?;
    let result_snapshot = delta::snapshot(
        &mut discovery,
        &result.root,
        &result.free,
        format,
        &mut remaining,
    )?;
    recheck(
        edge,
        &source,
        &source_snapshot,
        &result,
        &result_snapshot,
        format,
    )?;
    let mut fingerprint = source_snapshot.fingerprint;
    fingerprint.try_extend_bounded(result_snapshot.fingerprint, maximum_fingerprint)?;
    Ok((discovery.finish(), fingerprint))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn verify_completed(
    media: AdmittedRecoveryFilesystemMedia,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
    raw: &mut FundedCompletedHistoricalRawSlices,
    fold: &mut CompletedHistoryHeadFold,
    edge: &VerifiedRetirementRootEdge,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let mut storage = HistoricalWalkStorage::new(window, resident, raw);
    let source = addressed_root::observe_with_storage(
        &mut discovery,
        generation,
        node_capacity,
        format,
        edge.source_topology(),
        None,
        &mut storage,
    )?;
    let result = addressed_root::observe_with_storage(
        &mut discovery,
        generation.checked_add(1).ok_or(Denial::BoundExceeded)?,
        node_capacity,
        format,
        edge.result_topology(),
        Some(edge.basis().intent().candidate_root_sha256()),
        &mut storage,
    )?;
    let source_snapshot = delta::snapshot_with_storage(
        &mut discovery,
        &source.root,
        &source.free,
        format,
        &mut storage,
    )?;
    let result_snapshot = delta::snapshot_with_storage(
        &mut discovery,
        &result.root,
        &result.free,
        format,
        &mut storage,
    )?;
    recheck(
        edge,
        &source,
        &source_snapshot,
        &result,
        &result_snapshot,
        format,
    )?;
    // A release publishes free-range truth only; the custody head is unchanged.
    fold.apply(CompletedHistoryHeadStep::Ordinary {
        source: &source.root,
        result: &result.root,
    })?;
    let mut fingerprint = source.fingerprint;
    fingerprint.extend_with_storage(result.fingerprint, &mut storage)?;
    fingerprint.extend_with_storage(source_snapshot.fingerprint, &mut storage)?;
    fingerprint.extend_with_storage(result_snapshot.fingerprint, &mut storage)?;
    for snapshot in [source_snapshot.routes, result_snapshot.routes] {
        storage.discard_vec(snapshot)?;
    }
    for snapshot in [source_snapshot.segments, result_snapshot.segments] {
        storage.discard_vec(snapshot)?;
    }
    for snapshot in [source_snapshot.free_entries, result_snapshot.free_entries] {
        storage.discard_vec(snapshot)?;
    }
    drop(storage);
    Ok((discovery.finish(), fingerprint))
}

fn observe_roots(
    discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
    edge: &VerifiedRetirementRootEdge,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
) -> Result<(addressed_root::AddressedRoot, addressed_root::AddressedRoot), Denial> {
    let source = addressed_root::observe(
        discovery,
        generation,
        node_capacity,
        format,
        edge.source_topology(),
        None,
    )?;
    let result = addressed_root::observe(
        discovery,
        generation.checked_add(1).ok_or(Denial::BoundExceeded)?,
        node_capacity,
        format,
        edge.result_topology(),
        Some(edge.basis().intent().candidate_root_sha256()),
    )?;
    Ok((source, result))
}

fn recheck(
    edge: &VerifiedRetirementRootEdge,
    source: &addressed_root::AddressedRoot,
    source_snapshot: &delta::Snapshot,
    result: &addressed_root::AddressedRoot,
    result_snapshot: &delta::Snapshot,
    format: PhysicalRecordFormatDeclaration,
) -> Result<(), Denial> {
    if source_snapshot.transcript != edge.source_topology()
        || result_snapshot.transcript != edge.result_topology()
    {
        return Err(Denial::RoutingFrame);
    }
    edge.recheck_actual_media(
        ReleasedInventoryView::new(
            &source.root,
            &source.free,
            &source_snapshot.routes,
            &source_snapshot.segments,
            &source_snapshot.free_entries,
        ),
        ReleasedInventoryView::new(
            &result.root,
            &result.free,
            &result_snapshot.routes,
            &result_snapshot.segments,
            &result_snapshot.free_entries,
        ),
        edge.basis(),
        format,
        delta::MAX_TRANSITION_ENTRIES,
    )
    .map_err(|denial| delta::replay_denial(denial.exceeded_bound()))
}

#[cfg(test)]
#[path = "retirement_edge_tests.rs"]
mod tests;
