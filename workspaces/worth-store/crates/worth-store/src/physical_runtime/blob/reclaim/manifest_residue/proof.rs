use super::*;

pub(super) fn selected_proof(
    runtime: &ServingPhysicalRuntime,
    selected_root: worth_store_physical_format::RootPublicationCell,
    manifest: &Manifest,
    record: PersistedRecordIdentity,
    sha256: [u8; 32],
    reserved: Option<(ReservedDropRecordV1, OriginalDropReservedV1)>,
    placement: AdmittedRecordPlacementPolicy,
) -> Result<Option<SelectedOriginalDropProof>, BlobReclaimFailure> {
    let drop_set = manifest.drop_set();
    let store = drop_set.store();
    let attempt = drop_set.reclaim_attempt();
    let original = runtime.discover_blob_manifest_residue_no_effect(store, attempt);
    match (manifest, reserved, original) {
        (Manifest::V1(_), None, Some(original)) => {
            Ok(Some(SelectedOriginalDropProof::V1(original)))
        }
        (Manifest::V1(_), None, None) => Ok(None),
        (Manifest::V2(value), None, None)
            if manifest
                .slot()
                .is_some_and(|slot| slot <= selected_root.generation().get())
                && runtime.original_drop_binding_absent(store, attempt) == Some(true) =>
        {
            Ok(Some(SelectedOriginalDropProof::V2NeverReserved(
                value.clone(),
            )))
        }
        (Manifest::V2(_), None, None) => Ok(None),
        (Manifest::V2(value), Some((binding, reservation)), Some(original))
            if reservation_matches_manifest(reservation, value, record, sha256)
                && reservation_fingerprint_matches(runtime, value, reservation, placement)
                && reservation.request().idempotency() == original.idempotency_identity()
                && reservation.request().fingerprint() == original.request_fingerprint()
                && reservation.reserved_selected_generation()
                    <= selected_root.generation().get() =>
        {
            Ok(Some(SelectedOriginalDropProof::V2ProvenNoEffect {
                original,
                reserved: binding,
                reserved_selected_generation: reservation.reserved_selected_generation(),
            }))
        }
        (Manifest::V2(value), Some((binding, reservation)), None)
            if reservation_matches_manifest(reservation, value, record, sha256)
                && reservation_fingerprint_matches(runtime, value, reservation, placement)
                && reservation.reserved_selected_generation()
                    <= selected_root.generation().get() =>
        {
            let Some(recovered) = runtime.discover_recovered_original_drop_no_durable_effect(
                binding.record(),
                binding.frame_sha256(),
                reservation,
                selected_root,
            ) else {
                return Ok(None);
            };
            if recovered.reservation() != reservation
                || recovered.reservation_record() != binding.record()
                || recovered.reservation_sha256() != binding.frame_sha256()
                || recovered.selected_root() != selected_root
                || recovered.registry_runtime() != runtime.runtime_identity()
            {
                return Err(BlobReclaimFailure::ConflictingSelectedFate);
            }
            Ok(Some(SelectedOriginalDropProof::V2RecoveredNoBinding {
                recovered,
                reserved: binding,
            }))
        }
        _ => Err(BlobReclaimFailure::ConflictingSelectedFate),
    }
}

fn reservation_fingerprint_matches(
    runtime: &ServingPhysicalRuntime,
    manifest: &DropSetManifestV2,
    reserved: OriginalDropReservedV1,
    placement: AdmittedRecordPlacementPolicy,
) -> bool {
    let Some(candidate) = reserved.reserved_selected_generation().checked_add(1) else {
        return false;
    };
    let Ok(descriptor) = BlobReclaimDescriptorV1::new(
        manifest.store(),
        manifest.reclaim_attempt(),
        manifest.source_basis_digest(),
        reserved.manifest_record(),
        reserved.manifest_frame_sha256(),
        manifest.count(),
        reserved.reserved_selected_generation(),
        candidate,
    ) else {
        return false;
    };
    runtime
        .record_submission()
        .expected_original_drop_fingerprint(descriptor, placement)
        == Some(reserved.request().fingerprint())
}

pub(super) fn reservation_matches_manifest(
    reserved: OriginalDropReservedV1,
    manifest: &DropSetManifestV2,
    record: PersistedRecordIdentity,
    sha256: [u8; 32],
) -> bool {
    reserved.store() == manifest.store()
        && reserved.reclaim_attempt() == manifest.reclaim_attempt()
        && reserved.manifest_record() == record
        && reserved.manifest_frame_sha256() == sha256
        && reserved.source_basis_digest() == manifest.source_basis_digest()
        && reserved.manifest_selected_generation() == manifest.never_reserved_slot_generation()
}
