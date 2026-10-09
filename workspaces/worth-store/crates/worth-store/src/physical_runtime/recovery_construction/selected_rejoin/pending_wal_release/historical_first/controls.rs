//! Complete first V3 control triple re-read from Store's addressed result.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordV1, CurrentPhysicalRecordPlacement,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::VerifiedHistoricalPendingWalBatch;

use super::super::super::{
    control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint},
    tier, SelectedMediaRejoinDenial as Denial, MAX_CONTROL_FRAME_BYTES,
};
use super::super::delta::Snapshot;

pub(super) fn verify(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    source: &Snapshot,
    result: &Snapshot,
    batch: &VerifiedHistoricalPendingWalBatch,
    format: PhysicalRecordFormatDeclaration,
) -> Result<SelectedControlMediaFingerprint, Denial> {
    let mut slices = Vec::new();
    let descriptor = read(
        discovery,
        result,
        batch.descriptor_record(),
        format,
        &mut slices,
    )?;
    let reservation = read(
        discovery,
        result,
        batch.reservation_record(),
        format,
        &mut slices,
    )?;
    let manifest = read(
        discovery,
        result,
        batch.manifest_record(),
        format,
        &mut slices,
    )?;
    let BlobRecordV1::ReclaimDescriptorV3(drop) =
        decode_blob_record(&descriptor).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let BlobRecordV1::OriginalDropReserved(reserved) =
        decode_blob_record(&reservation).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let BlobRecordV1::DropSetManifestV3(dropped) =
        decode_blob_record(&manifest).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let base = drop.base();
    let publication = match dropped.source_basis() {
        BlobReclaimSourceBasisV1::ReleasedGeneration(source) => source.publication_record(),
        _ => return Err(Denial::ControlFrame),
    };
    if drop != batch.descriptor()
        || reserved != batch.reservation()
        || dropped != *batch.manifest()
        || drop.encode() != descriptor
        || reserved.encode() != reservation
        || dropped.encode() != manifest
        || digest(&descriptor) != batch.descriptor_frame_sha256()
        || digest(&reservation) != batch.reservation_frame_sha256()
        || digest(&manifest) != batch.manifest_frame_sha256()
        || base.predecessor().is_some()
        || base.store() != discovery.store_identity().bytes()
        || base.source_root_generation() != batch.chain().source_root_generation()
        || base.candidate_root_generation() != batch.chain().first_result_generation()
        || drop.custody().source_root_frame_sha256() != batch.chain().source_root_frame_sha256()
        || base.manifest_record() != batch.manifest_record()
        || base.manifest_frame_sha256() != batch.manifest_frame_sha256()
        || reserved.store() != base.store()
        || reserved.manifest_record() != batch.manifest_record()
        || reserved.manifest_frame_sha256() != batch.manifest_frame_sha256()
        || reserved.reclaim_attempt() != base.reclaim_attempt()
        || reserved.source_basis_digest() != base.source_basis_digest()
        || reserved.reserved_selected_generation() != base.source_root_generation()
        || reserved.request() != drop.custody().request()
        || dropped.store() != base.store()
        || dropped.reclaim_attempt() != base.reclaim_attempt()
        || dropped.source_basis_digest() != base.source_basis_digest()
        || dropped.source_basis().digest(base.store()) != base.source_basis_digest()
        || dropped.count() != base.manifest_count()
        || dropped.dropped().binary_search(&publication).is_err()
        || result
            .routes
            .binary_search_by_key(&publication, |route| route.record())
            .is_ok()
        || source
            .routes
            .binary_search_by_key(&batch.descriptor_record(), |route| route.record())
            .is_ok()
    {
        return Err(Denial::ControlFrame);
    }
    // Reservation and manifest are WAL-bound before the descriptor's root
    // publication; both must be the same routed frames across this V3 edge.
    for record in [batch.reservation_record(), batch.manifest_record()] {
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
    Ok(SelectedControlMediaFingerprint::observed(slices))
}

fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    result: &Snapshot,
    record: PersistedRecordIdentity,
    format: PhysicalRecordFormatDeclaration,
    slices: &mut Vec<SelectedArtifactSlice>,
) -> Result<Vec<u8>, Denial> {
    let index = result
        .routes
        .binary_search_by_key(&record, |route| route.record())
        .map_err(|_| Denial::CertificateRoster)?;
    let CurrentPhysicalRecordPlacement::Extent(extent) = result.routes[index] else {
        return Err(Denial::UnsupportedSelectedPlacement);
    };
    tier::no_release_frame::read(discovery, format, extent, MAX_CONTROL_FRAME_BYTES, slices)
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
