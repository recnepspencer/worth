//! Each non-tip tag-7 Batch is joined to its checkpoint-source control triple.

use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, BlobReclaimSourceKind,
    BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement, ReleaseCheckpointBatchV1,
    ReleasedDropWalFateWitnessV1, ReleasedGenerationReclaimBasisV1,
};

use super::{
    checks::{require_addressed_fate, require_frame},
    AddressedCheckpointBatchControl,
};
use crate::source_precedence::release_custody::SelectedCustodyDenial;
use crate::{PhysicalSourceSelection, ReconciledOperationFates};

#[allow(clippy::too_many_arguments)]
pub(super) fn verify_prior_control(
    routes: &[CurrentPhysicalRecordPlacement],
    controls: &AddressedCheckpointBatchControl,
    batch: ReleaseCheckpointBatchV1,
    selected: &PhysicalSourceSelection,
    fates: &ReconciledOperationFates,
    frames: &[ReleasedDropWalFateWitnessV1],
    members: &[([u8; 32], u64, u64)],
    policy: [u8; 32],
    cutoff: u64,
) -> Result<
    (
        BlobReclaimDescriptorV3,
        u16,
        ReleasedGenerationReclaimBasisV1,
    ),
    SelectedCustodyDenial,
> {
    let denial = SelectedCustodyDenial::ReleaseBinding;
    require_frame(
        routes,
        controls.descriptor(),
        BlobRecordKind::ReclaimDescriptorV3,
    )?;
    require_frame(
        routes,
        controls.reservation(),
        BlobRecordKind::OriginalDropReserved,
    )?;
    require_frame(
        routes,
        controls.manifest(),
        BlobRecordKind::DropSetManifestV3,
    )?;
    let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
        decode_blob_record(controls.descriptor().bytes()).map_err(|_| denial)?
    else {
        return Err(denial);
    };
    let BlobRecordV1::OriginalDropReserved(reservation) =
        decode_blob_record(controls.reservation().bytes()).map_err(|_| denial)?
    else {
        return Err(denial);
    };
    let BlobRecordV1::DropSetManifestV3(manifest) =
        decode_blob_record(controls.manifest().bytes()).map_err(|_| denial)?
    else {
        return Err(denial);
    };
    let base = descriptor.base();
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        return Err(denial);
    };
    if controls.descriptor().selected_placement().record() != batch.descriptor_record()
        || controls.descriptor().selected_payload_sha256() != batch.descriptor_frame_sha256()
        || controls.reservation().selected_placement().record() != batch.reservation_record()
        || controls.reservation().selected_payload_sha256() != batch.reservation_frame_sha256()
        || controls.manifest().selected_placement().record() != base.manifest_record()
        || controls.manifest().selected_payload_sha256() != base.manifest_frame_sha256()
        || descriptor.custody_digest() != batch.custody_digest()
        || descriptor.custody().request() != batch.request()
        || base.store()
            != selected
                .root()
                .selected()
                .selector()
                .store_identity()
                .bytes()
        || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
        || base.candidate_root_generation() != batch.candidate_root_generation()
        || base.predecessor() != batch.predecessor()
        || base.terminal() != batch.terminal()
        || reservation.store() != base.store()
        || reservation.reclaim_attempt() != base.reclaim_attempt()
        || reservation.request() != batch.request()
        || reservation.manifest_record() != base.manifest_record()
        || reservation.manifest_frame_sha256() != base.manifest_frame_sha256()
        || reservation.source_basis_digest() != base.source_basis_digest()
        || reservation.reserved_selected_generation() != base.source_root_generation()
        || manifest.store() != base.store()
        || manifest.reclaim_attempt() != base.reclaim_attempt()
        || manifest.source_basis_digest() != base.source_basis_digest()
        || manifest.count() != base.manifest_count()
        || manifest.never_reserved_slot_generation() != reservation.manifest_selected_generation()
        || (base.predecessor().is_none()
            && (base.cumulative_dropped() != u64::from(manifest.count())
                || manifest
                    .dropped()
                    .binary_search(&source.publication_record())
                    .is_err()))
    {
        return Err(denial);
    }
    require_addressed_fate(
        fates,
        frames,
        members,
        policy,
        batch.request(),
        base.store(),
        base.reclaim_attempt(),
        batch.fate(),
        cutoff,
    )?;
    Ok((descriptor, manifest.count(), source))
}
