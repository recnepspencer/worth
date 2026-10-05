//! Independent first V3 edge of an ordered post-NoRelease WAL custody chain.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    PersistedPhysicalRecoveryProjection, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_recovery_physics::{
    ReleasedInventoryView, VerifiedHistoricalPendingWalBatch, VerifiedReleasedV3InventoryTransition,
};

use super::super::{
    control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint},
    SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use super::{addressed_root, delta, directory_media};
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, StoreRecoveryBindingFreshnessSample,
};

#[path = "historical_first/controls.rs"]
mod controls;
#[path = "historical_first/wal.rs"]
mod wal;

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    media: AdmittedRecoveryFilesystemMedia,
    batch: &VerifiedHistoricalPendingWalBatch,
    sample: &StoreRecoveryBindingFreshnessSample,
    selected_wal: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
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
    if retained_peak_bytes >= delta::MAX_TRANSITION_MEMORY {
        return Err(Denial::BoundExceeded);
    }
    let chain = batch.chain();
    let projection = wal::match_batch(
        batch,
        sample,
        selected_wal,
        cutoff,
        delta::MAX_TRANSITION_MEMORY - retained_peak_bytes,
        format,
    )?;
    let format_charge = (projection.placements().len() as u64)
        .checked_mul(
            4 * std::mem::size_of::<worth_store_physical_format::CurrentPhysicalRecordPlacement>()
                as u64,
        )
        .ok_or(Denial::BoundExceeded)?;
    let mut remaining = delta::MAX_TRANSITION_MEMORY
        .checked_sub(retained_peak_bytes)
        .and_then(|bytes| bytes.checked_sub(format_charge))
        .ok_or(Denial::BoundExceeded)?;
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let source = addressed_root::observe(
        &mut discovery,
        chain.source_root_generation(),
        node_capacity,
        format,
        chain.source_topology(),
        Some(chain.source_root_frame_sha256()),
    )?;
    let result = addressed_root::observe(
        &mut discovery,
        chain.first_result_generation(),
        node_capacity,
        format,
        chain.first_result_topology(),
        Some(chain.first_result_root_frame_sha256()),
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
    if source_snapshot.transcript != chain.source_topology()
        || result_snapshot.transcript != chain.first_result_topology()
    {
        return Err(Denial::RoutingFrame);
    }
    let controls_fingerprint = controls::verify(
        &mut discovery,
        &source_snapshot,
        &result_snapshot,
        batch,
        format,
    )?;
    verify_delta(
        &source,
        &result,
        &source_snapshot,
        &result_snapshot,
        batch,
        &projection,
        format,
        remaining,
    )?;
    let directory = directory_media::verify_edge_with_storage(
        &mut discovery,
        &source.root,
        &source_snapshot,
        &result.root,
        &result_snapshot,
        chain.first_transition(),
        format,
        &mut (),
    )?;
    let maximum_fingerprint = delta::MAX_TRANSITION_MEMORY
        .checked_sub(retained_peak_bytes)
        .ok_or(Denial::BoundExceeded)?;
    let mut fingerprint = source_snapshot.fingerprint;
    fingerprint.try_extend_bounded(result_snapshot.fingerprint, maximum_fingerprint)?;
    fingerprint.try_extend_bounded(controls_fingerprint, maximum_fingerprint)?;
    if let Some(directory) = directory {
        fingerprint.try_extend_bounded(directory, maximum_fingerprint)?;
    }
    for (artifact, bytes) in [
        (
            RecordArtifactFile::RootManifest {
                generation: source.root.generation(),
            },
            source.root.encode(format),
        ),
        (
            RecordArtifactFile::FreeSpaceManifest {
                generation: source.free.generation(),
            },
            source.free.encode(format),
        ),
        (
            RecordArtifactFile::RootManifest {
                generation: result.root.generation(),
            },
            result.root.encode(format),
        ),
        (
            RecordArtifactFile::FreeSpaceManifest {
                generation: result.free.generation(),
            },
            result.free.encode(format),
        ),
    ] {
        fingerprint.try_extend_bounded(
            SelectedControlMediaFingerprint::observed(vec![SelectedArtifactSlice::observed(
                artifact, 0, &bytes, true,
            )
            .ok_or(Denial::BoundExceeded)?]),
            maximum_fingerprint,
        )?;
    }
    Ok((discovery.finish(), fingerprint))
}

#[allow(clippy::too_many_arguments)]
fn verify_delta(
    source: &addressed_root::AddressedRoot,
    result: &addressed_root::AddressedRoot,
    source_snapshot: &delta::Snapshot,
    result_snapshot: &delta::Snapshot,
    batch: &VerifiedHistoricalPendingWalBatch,
    projection: &PersistedPhysicalRecoveryProjection,
    format: PhysicalRecordFormatDeclaration,
    maximum_scratch: u64,
) -> Result<(), Denial> {
    let expected = batch.chain().first_transition();
    if projection.placements() != expected.projected()
        || !projection.segment_updates().is_empty()
        || !projection.root_state().inline_allocations().is_empty()
        || projection.root_state().last_inline_record().is_some()
        || projection.root_state().last_inline_segment().is_some()
    {
        return Err(Denial::RoutingFrame);
    }
    let mut dropped = batch.manifest().dropped().to_vec();
    if let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
        retirement: Some(retirement),
        ..
    } = projection.operation()
    {
        dropped.extend_from_slice(retirement.dropped_records());
    }
    dropped.sort_unstable();
    if dropped.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Denial::RoutingFrame);
    }
    let actual = VerifiedReleasedV3InventoryTransition::admit(
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
        format,
        delta::MAX_TRANSITION_ENTRIES,
        maximum_scratch,
        expected.directory_replacement(),
    )
    .map_err(|denial| delta::replay_denial(denial.exceeded_bound()))?;
    if &actual != expected {
        return Err(Denial::RoutingFrame);
    }
    Ok(())
}
