//! Store's independent actual-media join for one addressed V3 root edge.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedPhysicalRecoveryProjection, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    ReleasedInventoryView, VerifiedOrderedPendingWalReleaseBatch,
    VerifiedOrderedReleasedHeadReplayV14, VerifiedReleasedRootEdge,
    VerifiedReleasedV3InventoryTransition,
};

use super::super::tier::routes::RouteWalkStorage;
use super::super::{
    control_frames::SelectedControlMediaFingerprint, SelectedMediaRejoinDenial as Denial,
    MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use super::{addressed_root, delta, directory_media, head_effect_media};
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, StoreRecoveryBindingFreshnessSample,
};

#[path = "ordered_released/completed.rs"]
mod completed;
#[path = "ordered_released/controls.rs"]
mod controls;
#[path = "ordered_released/member_projection.rs"]
mod member_projection;
pub(super) use completed::verify_completed;

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    media: AdmittedRecoveryFilesystemMedia,
    window: &mut crate::physical_runtime::PhysicalRecoveryReadAllocation<'_>,
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
    replay: &VerifiedOrderedReleasedHeadReplayV14,
    source_generation: u64,
    sample: &StoreRecoveryBindingFreshnessSample,
    frames: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
    format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
    retained_peak: u64,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let projection = member_projection::matched_projection(
        edge,
        batch,
        replay,
        source_generation,
        sample,
        frames,
        cutoff,
        retained_peak,
        format,
    )?;
    let mut remaining =
        delta::MAX_TRANSITION_MEMORY
            .checked_sub(retained_peak)
            .and_then(|bytes| bytes.checked_sub(delta::projection_memory(&projection)))
            .and_then(|bytes| {
                bytes.checked_sub((projection.placements().len() as u64).checked_mul(
                    4 * std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64,
                )?)
            })
            .ok_or(Denial::BoundExceeded)?;
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let source = addressed_root::observe(
        &mut discovery,
        source_generation,
        node_capacity,
        format,
        edge.transition().source_topology(),
        Some(edge.source_root_frame_sha256()),
    )?;
    if <[u8; 32]>::from(Sha256::digest(source.free.encode(format)))
        != edge.source_free_space_frame_sha256()
    {
        return Err(Denial::RootBinding);
    }
    let result = addressed_root::observe(
        &mut discovery,
        source_generation
            .checked_add(1)
            .ok_or(Denial::BoundExceeded)?,
        node_capacity,
        format,
        edge.transition().result_topology(),
        Some(edge.result_root_frame_sha256()),
    )?;
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
    if source_snapshot.transcript != edge.transition().source_topology()
        || result_snapshot.transcript != edge.transition().result_topology()
    {
        return Err(Denial::RoutingFrame);
    }
    let observed_head = head_effect_media::observe(
        &mut discovery,
        window,
        replay.replay(),
        &source.root,
        &result.root,
        format,
        remaining,
    )?;
    remaining = remaining
        .checked_sub(
            observed_head
                .owned_heap_bytes()
                .ok_or(Denial::BoundExceeded)?,
        )
        .ok_or(Denial::BoundExceeded)?;
    let controls = controls::verify(
        &mut discovery,
        &source_snapshot,
        &result_snapshot,
        edge,
        batch,
        format,
    )?;
    remaining = remaining
        .checked_sub(controls.retained_memory_bytes())
        .ok_or(Denial::BoundExceeded)?;
    verify_delta(
        &source,
        &result,
        &source_snapshot,
        &result_snapshot,
        edge,
        batch,
        &projection,
        observed_head.replay(),
        format,
        remaining,
    )?;
    let directory = directory_media::verify_edge_with_storage(
        &mut discovery,
        &source.root,
        &source_snapshot,
        &result.root,
        &result_snapshot,
        edge.transition(),
        format,
        &mut (),
    )?;
    let maximum = delta::MAX_TRANSITION_MEMORY
        .checked_sub(retained_peak)
        .ok_or(Denial::BoundExceeded)?;
    let mut fingerprint = source_snapshot.fingerprint;
    fingerprint.try_extend_bounded(result_snapshot.fingerprint, maximum)?;
    if let Some(directory) = directory {
        fingerprint.try_extend_bounded(directory, maximum)?;
    }
    fingerprint.try_extend_bounded(observed_head.into_fingerprint(), maximum)?;
    fingerprint.try_extend_bounded(controls, maximum)?;
    // Both snapshot fingerprints already include their canonical root and
    // free headers as well as every routed/membership page. Do not retain a
    // second copy of those slices across an arbitrarily long edge sequence.
    Ok((discovery.finish(), fingerprint))
}

#[allow(clippy::too_many_arguments)]
fn verify_delta(
    source: &addressed_root::AddressedRoot,
    result: &addressed_root::AddressedRoot,
    source_snapshot: &delta::Snapshot,
    result_snapshot: &delta::Snapshot,
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
    projection: &PersistedPhysicalRecoveryProjection,
    replay: &worth_store_recovery_physics::VerifiedSelectedReleaseHeadReplayV14,
    format: PhysicalRecordFormatDeclaration,
    maximum_scratch: u64,
) -> Result<(), Denial> {
    verify_delta_with_storage(
        source,
        result,
        source_snapshot,
        result_snapshot,
        edge,
        batch,
        projection,
        replay,
        format,
        maximum_scratch,
        &mut (),
    )
}

#[allow(clippy::too_many_arguments)]
fn verify_delta_with_storage<S: RouteWalkStorage>(
    source: &addressed_root::AddressedRoot,
    result: &addressed_root::AddressedRoot,
    source_snapshot: &delta::Snapshot,
    result_snapshot: &delta::Snapshot,
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
    projection: &PersistedPhysicalRecoveryProjection,
    replay: &worth_store_recovery_physics::VerifiedSelectedReleaseHeadReplayV14,
    format: PhysicalRecordFormatDeclaration,
    maximum_scratch: u64,
    storage: &mut S,
) -> Result<(), Denial> {
    if projection.placements() != edge.transition().projected()
        || !projection.segment_updates().is_empty()
        || !projection.root_state().inline_allocations().is_empty()
        || projection.root_state().last_inline_record().is_some()
        || projection.root_state().last_inline_segment().is_some()
    {
        return Err(Denial::RoutingFrame);
    }
    let retirement_dropped = match projection.operation() {
        worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
            retirement: Some(retirement),
            ..
        } => retirement.dropped_records(),
        _ => &[],
    };
    let dropped_count = batch
        .manifest()
        .dropped()
        .len()
        .checked_add(retirement_dropped.len())
        .ok_or(Denial::BoundExceeded)?;
    let dropped_bytes = (dropped_count as u64)
        .checked_mul(4 * std::mem::size_of::<PersistedRecordIdentity>() as u64)
        .ok_or(Denial::BoundExceeded)?;
    if dropped_count as u64 > delta::MAX_TRANSITION_ENTRIES || dropped_bytes > maximum_scratch {
        return Err(Denial::BoundExceeded);
    }
    let mut dropped = storage.reserve_vec(dropped_count)?;
    dropped.extend_from_slice(batch.manifest().dropped());
    dropped.extend_from_slice(retirement_dropped);
    dropped.sort_unstable();
    if dropped.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Denial::RoutingFrame);
    }
    let actual = VerifiedReleasedV3InventoryTransition::admit_with_head_replay(
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
        &dropped,
        projection.placements(),
        replay,
        format,
        delta::MAX_TRANSITION_ENTRIES,
        maximum_scratch
            .checked_sub(dropped_bytes)
            .ok_or(Denial::BoundExceeded)?,
        edge.transition().directory_replacement(),
    )
    .map_err(|_| Denial::RoutingFrame)?;
    if &actual != edge.transition() {
        return Err(Denial::RoutingFrame);
    }
    storage.discard_vec(dropped)?;
    Ok(())
}
