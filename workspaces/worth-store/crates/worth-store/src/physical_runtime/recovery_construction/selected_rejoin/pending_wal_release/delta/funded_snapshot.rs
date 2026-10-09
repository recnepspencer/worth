//! Funded Store snapshot of routed and membership state for a historical edge.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PhysicalInventoryTranscriptBuilderV1, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::{Snapshot, MAX_TRANSITION_ENTRIES};
use crate::physical_runtime::recovery_construction::selected_rejoin::{
    control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint},
    pending_wal_release::topology,
    tier::routes::{self, RouteWalkStorage},
    SelectedMediaRejoinDenial as Denial,
};

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn snapshot_with_storage<
    S: RouteWalkStorage,
>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    storage: &mut S,
) -> Result<Snapshot, Denial> {
    if root.record_count() > MAX_TRANSITION_ENTRIES || free.entry_count() > MAX_TRANSITION_ENTRIES {
        return Err(Denial::BoundExceeded);
    }
    let root_frame = storage.reserve_vec::<u8>(root.encoded_frame_bytes())?;
    let free_frame = storage.reserve_vec::<u8>(free.encoded_frame_bytes())?;
    let consumed_bytes = u64::try_from(root_frame.capacity())
        .ok()
        .and_then(|bytes| bytes.checked_add(u64::try_from(free_frame.capacity()).ok()?))
        .ok_or(Denial::BoundExceeded)?;
    let transcript = PhysicalInventoryTranscriptBuilderV1::new_in_reserved(
        root,
        free,
        format,
        MAX_TRANSITION_ENTRIES,
        root_frame,
        free_frame,
    );
    storage.release_consumed_bytes(consumed_bytes)?;
    let mut transcript = transcript.map_err(|_| Denial::RoutingFrame)?;
    let (routes_proof, routes) = routes::verify_snapshot_with_storage(
        discovery,
        root,
        free,
        format,
        &mut transcript,
        storage,
    )?;
    let mut fingerprint = routes_proof.into_media_fingerprint_with_storage(storage)?;
    let (segments, free_entries, membership_slices) =
        topology::observe_membership_snapshot_with_storage(
            discovery,
            root,
            free,
            format,
            &mut transcript,
            MAX_TRANSITION_ENTRIES,
            storage,
        )?;
    fingerprint.extend_with_storage(
        SelectedControlMediaFingerprint::observed(membership_slices),
        storage,
    )?;
    let headers = canonical_header_slices(root, free, format, storage)?;
    fingerprint.extend_with_storage(SelectedControlMediaFingerprint::observed(headers), storage)?;
    Ok(Snapshot {
        routes,
        segments,
        free_entries,
        transcript: transcript.finish().map_err(|_| Denial::RoutingFrame)?,
        fingerprint,
    })
}

fn canonical_header_slices<S: RouteWalkStorage>(
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    storage: &mut S,
) -> Result<Vec<SelectedArtifactSlice>, Denial> {
    let generation = root.generation();
    let mut slices = storage.reserve_vec(2)?;
    let root_buffer = storage.reserve_vec::<u8>(root.encoded_frame_bytes())?;
    let root_buffer = root
        .encode_in_reserved(format, root_buffer)
        .ok_or(Denial::RootBinding)?;
    slices.push(
        SelectedArtifactSlice::observed(
            RecordArtifactFile::RootManifest { generation },
            0,
            &root_buffer,
            true,
        )
        .ok_or(Denial::BoundExceeded)?,
    );
    storage.discard_vec(root_buffer)?;
    let free_buffer = storage.reserve_vec::<u8>(free.encoded_frame_bytes())?;
    let free_buffer = free
        .encode_in_reserved(format, free_buffer)
        .ok_or(Denial::RootBinding)?;
    slices.push(
        SelectedArtifactSlice::observed(
            RecordArtifactFile::FreeSpaceManifest { generation },
            0,
            &free_buffer,
            true,
        )
        .ok_or(Denial::BoundExceeded)?,
    );
    storage.discard_vec(free_buffer)?;
    Ok(slices)
}
