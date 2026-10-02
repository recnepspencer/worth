//! Candidate-root triple and typed V3 control joins for one ordered edge.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    VerifiedOrderedPendingWalReleaseBatch, VerifiedReleasedRootEdge,
};

use super::super::super::{
    control_frames::SelectedControlMediaFingerprint, tier, SelectedMediaRejoinDenial as Denial,
    MAX_CONTROL_FRAME_BYTES,
};
use super::super::delta;
#[path = "controls/completed.rs"]
mod completed;
pub(super) use completed::verify_with_storage;

pub(super) fn verify(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    source: &delta::Snapshot,
    result: &delta::Snapshot,
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
    format: PhysicalRecordFormatDeclaration,
) -> Result<SelectedControlMediaFingerprint, Denial> {
    let records = verify_batch_binding(discovery, edge, batch)?;
    let mut slices = Vec::new();
    for (frame, expected) in [
        (batch.descriptor_frame(), batch.descriptor().encode()),
        (batch.reservation_frame(), batch.reservation().encode()),
        (batch.manifest_frame(), batch.manifest().encode()),
    ] {
        if frame.candidate_root_frame_sha256() != edge.result_root_frame_sha256()
            || frame.payload_sha256() != <[u8; 32]>::from(Sha256::digest(&expected))
            || frame.bytes() != expected
        {
            return Err(Denial::ControlFrame);
        }
        let index = result
            .routes
            .binary_search_by_key(&frame.record(), |route| route.record())
            .map_err(|_| Denial::ControlFrame)?;
        let CurrentPhysicalRecordPlacement::Extent(extent) = result.routes[index] else {
            return Err(Denial::UnsupportedSelectedPlacement);
        };
        let maximum = u64::try_from(expected.len()).map_err(|_| Denial::BoundExceeded)?;
        if maximum > MAX_CONTROL_FRAME_BYTES {
            return Err(Denial::BoundExceeded);
        }
        let bytes = tier::no_release_frame::read(discovery, format, extent, maximum, &mut slices)?;
        if bytes != expected || Sha256::digest(&bytes).as_slice() != frame.payload_sha256() {
            return Err(Denial::ControlFrame);
        }
        if !matches!(
            decode_blob_record(&bytes),
            Ok(BlobRecordV1::ReclaimDescriptorV3(_)
                | BlobRecordV1::OriginalDropReserved(_)
                | BlobRecordV1::DropSetManifestV3(_))
        ) {
            return Err(Denial::ControlFrame);
        }
    }
    verify_source_routes(source, result, records)?;
    Ok(SelectedControlMediaFingerprint::observed(slices))
}

pub(super) fn verify_batch_binding(
    discovery: &BoundedRecoveryFilesystemDiscovery,
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
) -> Result<[PersistedRecordIdentity; 3], Denial> {
    let base = batch.descriptor().base();
    let custody = batch.descriptor().custody();
    let reserved = batch.reservation();
    let manifest = batch.manifest();
    let records = [
        batch.descriptor_frame().record(),
        batch.reservation_frame().record(),
        batch.manifest_frame().record(),
    ];
    if records[0] == records[1]
        || records[0] == records[2]
        || records[1] == records[2]
        || batch.descriptor_frame().kind() != BlobRecordKind::ReclaimDescriptorV3
        || batch.reservation_frame().kind() != BlobRecordKind::OriginalDropReserved
        || batch.manifest_frame().kind() != BlobRecordKind::DropSetManifestV3
        || base.manifest_record() != records[2]
        || base.manifest_frame_sha256() != batch.manifest_frame().payload_sha256()
        || base.store() != discovery.store_identity().bytes()
        || base.source_root_generation() != edge.candidate_root_generation().saturating_sub(1)
        || base.candidate_root_generation() != edge.candidate_root_generation()
        || custody.source_root_frame_sha256() != edge.source_root_frame_sha256()
        || custody.source_free_space_frame_sha256() != edge.source_free_space_frame_sha256()
        || custody.request().idempotency() != edge.operation()
        || reserved.store() != base.store()
        || reserved.reclaim_attempt() != base.reclaim_attempt()
        || reserved.manifest_record() != records[2]
        || reserved.manifest_frame_sha256() != batch.manifest_frame().payload_sha256()
        || reserved.source_basis_digest() != base.source_basis_digest()
        || reserved.reserved_selected_generation() != base.source_root_generation()
        || reserved.request() != custody.request()
        || manifest.store() != base.store()
        || manifest.reclaim_attempt() != base.reclaim_attempt()
        || manifest.source_basis_digest() != base.source_basis_digest()
        || manifest.source_basis().digest(base.store()) != base.source_basis_digest()
        || manifest.count() != base.manifest_count()
    {
        return Err(Denial::ControlFrame);
    }
    Ok(records)
}

pub(super) fn verify_source_routes(
    source: &delta::Snapshot,
    result: &delta::Snapshot,
    records: [PersistedRecordIdentity; 3],
) -> Result<(), Denial> {
    for record in [records[1], records[2]] {
        let source_route = source
            .routes
            .binary_search_by_key(&record, |route| route.record())
            .ok()
            .map(|index| source.routes[index]);
        let result_route = result
            .routes
            .binary_search_by_key(&record, |route| route.record())
            .ok()
            .map(|index| result.routes[index]);
        if source_route.is_none() || source_route != result_route {
            return Err(Denial::ControlFrame);
        }
    }
    Ok(())
}
