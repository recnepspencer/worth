//! Every selected stage-2 reservation is joined to its exact manifest and a
//! Store-sealed terminal fate or recovered no-binding certificate before retry.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV1, BlobRecordV1, DropSetManifestV2,
    FailedIngestReclaimBasisV1, OriginalDropReservedV1, PersistedRecordIdentity,
};

use super::{scan, BlobReclaimFailure, BlobReclaimLimits};
use crate::physical_runtime::{
    durability::PhysicalRecoveredOriginalDropNoDurableEffect, AdmittedRecordPlacementPolicy,
};

pub(super) struct SelectionReservationLink {
    pub(super) record: PersistedRecordIdentity,
    pub(super) frame_sha256: [u8; 32],
    pub(super) reserved: OriginalDropReservedV1,
    pub(super) matched_manifest_count: Option<u16>,
}

pub(super) struct SelectionDescriptorLink {
    pub(super) record: PersistedRecordIdentity,
    pub(super) descriptor: BlobReclaimDescriptorV1,
}
use crate::physical_runtime::{PhysicalRecordReader, ServingPhysicalRuntime};

pub(super) fn reservation_blocks_fresh_attempt(
    reserved: OriginalDropReservedV1,
    store: [u8; 16],
    basis_digest: [u8; 32],
) -> bool {
    reserved.store() == store && reserved.source_basis_digest() == basis_digest
}

#[allow(clippy::too_many_arguments)]
pub(super) fn verify_terminal_history(
    runtime: &ServingPhysicalRuntime,
    reader: PhysicalRecordReader,
    store: [u8; 16],
    basis: FailedIngestReclaimBasisV1,
    reservations: &mut [SelectionReservationLink],
    descriptors: &mut [SelectionDescriptorLink],
    limits: BlobReclaimLimits,
    placement: AdmittedRecordPlacementPolicy,
    scratch: &mut [u8],
    work: &mut scan::ReclaimInspectionWork,
) -> Result<
    (
        PhysicalRecordReader,
        Vec<PhysicalRecoveredOriginalDropNoDurableEffect>,
    ),
    BlobReclaimFailure,
> {
    let deny = || BlobReclaimFailure::ConflictingSelectedFate;
    reservations.sort_unstable_by_key(|link| link.reserved.reclaim_attempt());
    if !reservation_attempts_unique(reservations) {
        return Err(deny());
    }
    descriptors.sort_unstable_by_key(|link| link.descriptor.manifest_record());
    if descriptors
        .windows(2)
        .any(|pair| pair[0].descriptor.manifest_record() == pair[1].descriptor.manifest_record())
    {
        return Err(deny());
    }
    let selected_generation = reader.protected_root().root().generation().get();
    let selected_root = reader.protected_root().root();
    let reader = scan::walk(reader, limits, scratch, work, |record, bytes| {
        if !bytes.starts_with(b"WRC11BLB") {
            return Ok(());
        }
        let BlobRecordV1::DropSetManifestV2(manifest) =
            decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)?
        else {
            return Ok(());
        };
        if manifest.source_basis() != basis || manifest.store() != store {
            return Ok(());
        }
        let Ok(index) = reservations.binary_search_by_key(&manifest.reclaim_attempt(), |link| {
            link.reserved.reclaim_attempt()
        }) else {
            return Ok(());
        };
        let link = &mut reservations[index];
        if link.matched_manifest_count.is_some()
            || !reservation_matches_selected_manifest(
                link.reserved,
                &manifest,
                record,
                Sha256::digest(bytes).into(),
                basis,
            )
            || link.reserved.reserved_selected_generation() > selected_generation
        {
            return Err(deny());
        }
        if let Ok(descriptor_index) =
            descriptors.binary_search_by_key(&record, |link| link.descriptor.manifest_record())
        {
            let descriptor = descriptors[descriptor_index].descriptor;
            if descriptor.reclaim_attempt() != manifest.reclaim_attempt()
                || descriptor.store() != store
                || descriptor.source_basis_digest() != manifest.source_basis_digest()
                || descriptor.manifest_frame_sha256() != link.reserved.manifest_frame_sha256()
                || descriptor.manifest_count() != manifest.count()
            {
                return Err(deny());
            }
        }
        link.matched_manifest_count = Some(manifest.count());
        Ok(())
    })?;
    descriptors.sort_unstable_by_key(|link| link.descriptor.reclaim_attempt());
    if descriptors
        .windows(2)
        .any(|pair| pair[0].descriptor.reclaim_attempt() == pair[1].descriptor.reclaim_attempt())
    {
        return Err(deny());
    }
    let capacity = usize::try_from(limits.maximum_selected_records())
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    let mut recovered = Vec::new();
    recovered
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if recovered.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    for link in reservations {
        let reserved = link.reserved;
        if link.record == reserved.manifest_record() {
            return Err(deny());
        }
        let count = link.matched_manifest_count.ok_or_else(deny)?;
        let key = reserved.request().idempotency();
        let fingerprint = reserved.request().fingerprint();
        let candidate_generation = reserved
            .reserved_selected_generation()
            .checked_add(1)
            .ok_or_else(deny)?;
        let descriptor = BlobReclaimDescriptorV1::new(
            store,
            reserved.reclaim_attempt(),
            basis.digest(store),
            reserved.manifest_record(),
            reserved.manifest_frame_sha256(),
            count,
            reserved.reserved_selected_generation(),
            candidate_generation,
        )
        .map_err(|_| deny())?;
        if runtime
            .record_submission()
            .expected_original_drop_fingerprint(descriptor, placement)
            != Some(fingerprint)
        {
            return Err(deny());
        }
        if let Ok(index) = descriptors.binary_search_by_key(&reserved.reclaim_attempt(), |entry| {
            entry.descriptor.reclaim_attempt()
        }) {
            let selected = &descriptors[index];
            let completed = runtime
                .discover_blob_manifest_residue_completed(
                    store,
                    reserved.reclaim_attempt(),
                    key,
                    fingerprint,
                )
                .ok_or_else(deny)?;
            if completed.idempotency_identity() != key
                || completed.request_fingerprint() != fingerprint
                || !descriptor_matches_completed(
                    link,
                    selected,
                    count,
                    completed.descriptor_record(),
                    completed.current_root_generation(),
                    selected_generation,
                    basis.digest(store),
                )
            {
                return Err(deny());
            }
        } else {
            if let Some(negative) =
                runtime.discover_blob_manifest_residue_no_effect(store, reserved.reclaim_attempt())
            {
                if negative.idempotency_identity() != key
                    || negative.request_fingerprint() != fingerprint
                {
                    return Err(deny());
                }
            } else {
                let sealed = runtime
                    .discover_recovered_original_drop_no_durable_effect(
                        link.record,
                        link.frame_sha256,
                        reserved,
                        selected_root,
                    )
                    .ok_or_else(deny)?;
                if sealed.reservation_record() != link.record
                    || sealed.reservation_sha256() != link.frame_sha256
                    || sealed.reservation() != reserved
                    || sealed.selected_root() != selected_root
                    || sealed.registry_runtime() != runtime.runtime_identity()
                {
                    return Err(deny());
                }
                recovered.push(sealed);
            }
        }
    }
    Ok((reader, recovered))
}

fn reservation_attempts_unique(reservations: &[SelectionReservationLink]) -> bool {
    !reservations
        .windows(2)
        .any(|pair| pair[0].reserved.reclaim_attempt() == pair[1].reserved.reclaim_attempt())
}

#[allow(clippy::too_many_arguments)]
fn descriptor_matches_completed(
    reservation: &SelectionReservationLink,
    selected: &SelectionDescriptorLink,
    manifest_count: u16,
    completed_descriptor: PersistedRecordIdentity,
    completed_root_generation: u64,
    selected_generation: u64,
    basis_digest: [u8; 32],
) -> bool {
    let reserved = reservation.reserved;
    let descriptor = selected.descriptor;
    selected.record == completed_descriptor
        && selected.record != reservation.record
        && descriptor.manifest_record() == reserved.manifest_record()
        && descriptor.manifest_frame_sha256() == reserved.manifest_frame_sha256()
        && descriptor.manifest_count() == manifest_count
        && descriptor.source_basis_digest() == basis_digest
        && descriptor.source_root_generation() == reserved.reserved_selected_generation()
        && descriptor.candidate_root_generation() == completed_root_generation
        && completed_root_generation <= selected_generation
}

pub(super) fn reservation_matches_selected_manifest(
    reserved: OriginalDropReservedV1,
    manifest: &DropSetManifestV2,
    record: PersistedRecordIdentity,
    sha256: [u8; 32],
    basis: FailedIngestReclaimBasisV1,
) -> bool {
    reserved.manifest_record() == record
        && reserved.manifest_frame_sha256() == sha256
        && reserved.store() == manifest.store()
        && reserved.reclaim_attempt() == manifest.reclaim_attempt()
        && reserved.source_basis_digest() == manifest.source_basis_digest()
        && manifest.source_basis() == basis
        && reserved.manifest_selected_generation() == manifest.never_reserved_slot_generation()
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::OriginalDropReservationRequestV1;

    fn record(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
    }

    fn reservation(record_id: PersistedRecordIdentity) -> SelectionReservationLink {
        let request = OriginalDropReservationRequestV1::new([3; 32], [4; 32], 1, 9).unwrap();
        SelectionReservationLink {
            record: record(7),
            frame_sha256: [8; 32],
            reserved: OriginalDropReservedV1::new(
                [1; 16], [2; 16], record_id, [5; 32], [6; 32], 3, 4, request,
            )
            .unwrap(),
            matched_manifest_count: Some(1),
        }
    }

    #[test]
    fn duplicate_selected_reservation_attempt_is_ambiguous() {
        let first = reservation(record(6));
        let second = reservation(record(8));
        assert!(!reservation_attempts_unique(&[first, second]));
    }

    #[test]
    fn completed_descriptor_requires_exact_selected_output_and_root() {
        let reserved = reservation(record(6));
        let descriptor =
            BlobReclaimDescriptorV1::new([1; 16], [2; 16], [6; 32], record(6), [5; 32], 1, 4, 5)
                .unwrap();
        let selected = SelectionDescriptorLink {
            record: record(8),
            descriptor,
        };
        assert!(descriptor_matches_completed(
            &reserved,
            &selected,
            1,
            record(8),
            5,
            6,
            [6; 32],
        ));
        assert!(!descriptor_matches_completed(
            &reserved,
            &selected,
            1,
            record(9),
            5,
            6,
            [6; 32],
        ));
        assert!(!descriptor_matches_completed(
            &reserved,
            &selected,
            1,
            record(8),
            6,
            6,
            [6; 32],
        ));
        assert!(!descriptor_matches_completed(
            &reserved,
            &selected,
            1,
            record(8),
            5,
            4,
            [6; 32],
        ));
        let forged = SelectionDescriptorLink {
            record: record(8),
            descriptor: BlobReclaimDescriptorV1::new(
                [1; 16],
                [2; 16],
                [6; 32],
                record(6),
                [9; 32],
                1,
                4,
                5,
            )
            .unwrap(),
        };
        assert!(!descriptor_matches_completed(
            &reserved,
            &forged,
            1,
            record(8),
            5,
            6,
            [6; 32],
        ));
    }
}
