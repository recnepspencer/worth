//! Selected-root custody for removing one orphan DropSetManifest. The
//! original negative fate is discovered under Store idempotency authority;
//! caller bytes and mere absence of a descriptor are never fate evidence.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV1, BlobRecordV1, DropSetManifestV1,
    DropSetManifestV2, OriginalDropReservedV1, PersistedRecordIdentity, ReservedDropRecordV1,
};

use crate::physical_runtime::{
    durability::{
        AdmittedManifestResidueRetirement, PhysicalBlobSessionClaim, SelectedOriginalDropProof,
    },
    AdmittedRecordPlacementPolicy, BlobPhysicalAllocation, ServingPhysicalRuntime,
};

use super::{scan, selection::SelectedFailedBlobResidue, BlobReclaimFailure, BlobReclaimLimits};

mod proof;
use proof::reservation_matches_manifest;
use proof::selected_proof;
mod references;
use references::{
    conflicts_with_manifest, conflicts_with_metadata_reference, selected_payload_present,
};

impl ServingPhysicalRuntime {
    /// Continue the already claimed, empty-payload reclaim through exactly one
    /// manifest-only maintenance admission. `None` means every historical
    /// manifest is descriptor-linked and there is no orphan to clean.
    pub(super) fn admit_selected_blob_manifest_residue(
        &self,
        selected: SelectedFailedBlobResidue,
        claim: &mut PhysicalBlobSessionClaim,
        allocation: &BlobPhysicalAllocation<'_>,
        limits: BlobReclaimLimits,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Result<
        (
            Option<AdmittedManifestResidueRetirement>,
            scan::ReclaimInspectionWork,
        ),
        BlobReclaimFailure,
    > {
        if !selected.is_empty() || selected.remaining() != 0 {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
        let mut work = selected.inspection();
        let (reader, basis, dropped, _, occupied_attempts, _) = selected.into_reclaim_parts();
        drop(dropped);
        drop(occupied_attempts);
        let Some((reader, manifest, sha256, count, proof)) = inspect_selected(
            self, reader, basis, limits, placement, allocation, &mut work,
        )?
        else {
            return Ok((None, work));
        };
        self.admit_blob_manifest_residue(
            claim, reader, allocation, basis, manifest, sha256, count, proof, placement,
        )
        .map(|admitted| (Some(admitted), work))
        .map_err(|_| BlobReclaimFailure::ConflictingSelectedFate)
    }
}

enum Manifest {
    V1(DropSetManifestV1),
    V2(DropSetManifestV2),
}

impl Manifest {
    fn drop_set(&self) -> &DropSetManifestV1 {
        match self {
            Self::V1(value) => value,
            Self::V2(value) => value.drop_set(),
        }
    }

    fn slot(&self) -> Option<u64> {
        match self {
            Self::V1(_) => None,
            Self::V2(value) => Some(value.never_reserved_slot_generation()),
        }
    }
}

struct SelectedManifest {
    record: PersistedRecordIdentity,
    manifest: Manifest,
    sha256: [u8; 32],
    proof: SelectedOriginalDropProof,
}

pub(super) struct ReservedLink {
    record: PersistedRecordIdentity,
    sha256: [u8; 32],
    reserved: OriginalDropReservedV1,
}

/// Pre-admitted descriptor roster used only to ignore genuinely completed
/// historical manifests. Every field is checked against the manifest frame.
pub(super) struct DescriptorLink {
    record: PersistedRecordIdentity,
    descriptor: BlobReclaimDescriptorV1,
}

fn descriptor_matches_manifest(
    link: &DescriptorLink,
    manifest: &DropSetManifestV1,
    sha256: [u8; 32],
) -> bool {
    link.descriptor.store() == manifest.store()
        && link.descriptor.reclaim_attempt() == manifest.reclaim_attempt()
        && link.descriptor.source_basis_digest() == manifest.source_basis_digest()
        && link.descriptor.manifest_frame_sha256() == sha256
        && link.descriptor.manifest_count() == manifest.count()
}

fn inspect_selected(
    runtime: &ServingPhysicalRuntime,
    reader: crate::physical_runtime::PhysicalRecordReader,
    basis: worth_store_physical_format::FailedIngestReclaimBasisV1,
    limits: BlobReclaimLimits,
    placement: AdmittedRecordPlacementPolicy,
    allocation: &BlobPhysicalAllocation<'_>,
    work: &mut scan::ReclaimInspectionWork,
) -> Result<
    Option<(
        crate::physical_runtime::PhysicalRecordReader,
        PersistedRecordIdentity,
        [u8; 32],
        u16,
        SelectedOriginalDropProof,
    )>,
    BlobReclaimFailure,
> {
    let required = limits
        .memory_bytes()
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if allocation.bytes() < required.get() {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let reader = reader.into_rebuild();
    let capacity = usize::try_from(limits.maximum_selected_records())
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    let mut descriptor_links = Vec::new();
    descriptor_links
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if descriptor_links.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let mut reserved_links = Vec::new();
    reserved_links
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if reserved_links.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let mut manifest_attempts = Vec::new();
    manifest_attempts
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if manifest_attempts.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let mut scratch = scan::frame_window()?;
    let store = runtime.store_identity().bytes();
    let basis_digest = basis.digest(store);
    let reader = scan::walk(reader, limits, &mut scratch, work, |record, bytes| {
        if !bytes.starts_with(b"WRC11BLB") {
            return Ok(());
        }
        match decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)? {
            BlobRecordV1::ReclaimDescriptor(value)
                if value.store() == store && value.source_basis_digest() == basis_digest =>
            {
                descriptor_links.push(DescriptorLink {
                    record: value.manifest_record(),
                    descriptor: value,
                });
            }
            BlobRecordV1::OriginalDropReserved(value)
                if value.store() == store && value.source_basis_digest() == basis_digest =>
            {
                reserved_links.push(ReservedLink {
                    record,
                    sha256: Sha256::digest(bytes).into(),
                    reserved: value,
                });
            }
            _ => {}
        }
        Ok(())
    })?;
    descriptor_links.sort_unstable_by_key(|link| link.descriptor.reclaim_attempt());
    if descriptor_links
        .windows(2)
        .any(|pair| pair[0].descriptor.reclaim_attempt() == pair[1].descriptor.reclaim_attempt())
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    reserved_links.sort_unstable_by_key(|link| link.reserved.reclaim_attempt());
    if reserved_links
        .windows(2)
        .any(|pair| pair[0].reserved.reclaim_attempt() == pair[1].reserved.reclaim_attempt())
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let mut chosen: Option<SelectedManifest> = None;
    let mut unproven_orphan = false;
    let selected_root = reader.protected_root().root();
    let reader = scan::walk(reader, limits, &mut scratch, work, |record, bytes| {
        if !bytes.starts_with(b"WRC11BLB") {
            return Ok(());
        }
        let manifest = match decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)? {
            BlobRecordV1::DropSetManifest(value) => Manifest::V1(value),
            BlobRecordV1::DropSetManifestV2(value) => Manifest::V2(value),
            _ => return Ok(()),
        };
        let value = manifest.drop_set();
        if value.source_basis().session() != basis.session() {
            return Ok(());
        }
        if value.store() != store || value.source_basis() != basis {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
        manifest_attempts.push(value.reclaim_attempt());
        let sha256: [u8; 32] = Sha256::digest(bytes).into();
        let descriptor = descriptor_links.binary_search_by_key(&value.reclaim_attempt(), |link| {
            link.descriptor.reclaim_attempt()
        });
        let reservation = reserved_links.binary_search_by_key(&value.reclaim_attempt(), |link| {
            link.reserved.reclaim_attempt()
        });
        if let Ok(index) = descriptor {
            if !descriptor_matches_manifest(&descriptor_links[index], &value, sha256)
                || descriptor_links[index].record != record
                || runtime
                    .discover_blob_manifest_residue_no_effect(store, value.reclaim_attempt())
                    .is_some()
            {
                return Err(BlobReclaimFailure::ConflictingSelectedFate);
            }
            return Ok(());
        }
        let reserved = reservation.ok().map(|index| &reserved_links[index]);
        if reserved.is_some_and(|link| {
            !matches!(&manifest, Manifest::V2(value) if reservation_matches_manifest(
                link.reserved, value, record, sha256
            ))
        }) {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
        let reserved = reserved
            .map(|link| {
                ReservedDropRecordV1::new(link.record, link.sha256)
                    .map(|binding| (binding, link.reserved))
                    .ok_or(BlobReclaimFailure::ConflictingSelectedFate)
            })
            .transpose()?;
        let Some(proof) = selected_proof(
            runtime,
            selected_root,
            &manifest,
            record,
            sha256,
            reserved,
            placement,
        )?
        else {
            unproven_orphan = true;
            return Ok(());
        };
        if chosen
            .as_ref()
            .is_none_or(|previous| record < previous.record)
        {
            chosen = Some(SelectedManifest {
                record,
                manifest,
                sha256,
                proof,
            });
        }
        Ok(())
    })?;
    manifest_attempts.sort_unstable();
    // A distinct unresolved attempt stays routed while one independently
    // proved orphan is cleaned. Duplicate custody for the *same* attempt is
    // conflicting, never an alternate candidate.
    if manifest_attempts.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let Some(SelectedManifest {
        record: manifest_record,
        manifest,
        sha256,
        proof,
    }) = chosen
    else {
        return if unproven_orphan {
            Err(BlobReclaimFailure::ConflictingSelectedFate)
        } else {
            Ok(None)
        };
    };
    let drop_set = manifest.drop_set();
    if drop_set.dropped().binary_search(&manifest_record).is_ok() {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let mut reserved: Option<(ReservedDropRecordV1, OriginalDropReservedV1)> = None;
    let reader = scan::walk(reader, limits, &mut scratch, work, |record, bytes| {
        if selected_payload_present(record, drop_set) {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
        if !bytes.starts_with(b"WRC11BLB") {
            return Ok(());
        }
        let fact = decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)?;
        if let BlobRecordV1::OriginalDropReserved(value) = &fact {
            if value.manifest_record() == manifest_record {
                let binding = ReservedDropRecordV1::new(record, Sha256::digest(bytes).into())
                    .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
                if reserved.replace((binding, *value)).is_some() {
                    return Err(BlobReclaimFailure::ConflictingSelectedFate);
                }
            } else if value.reclaim_attempt() == drop_set.reclaim_attempt() {
                return Err(BlobReclaimFailure::ConflictingSelectedFate);
            }
            return Ok(());
        }
        if conflicts_with_manifest(&fact, record, drop_set, manifest_record) {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
        Ok(())
    })?;
    let expected_reserved = match &proof {
        SelectedOriginalDropProof::V2ProvenNoEffect { reserved, .. }
        | SelectedOriginalDropProof::V2RecoveredNoBinding { reserved, .. } => Some(*reserved),
        _ => None,
    };
    if reserved.map(|(binding, _)| binding) != expected_reserved {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let protected_reserved = match &proof {
        SelectedOriginalDropProof::V2ProvenNoEffect { reserved, .. }
        | SelectedOriginalDropProof::V2RecoveredNoBinding { reserved, .. } => Some(*reserved),
        _ => None,
    };
    let reader = if let Some(reserved) = protected_reserved {
        let target = reserved.record();
        scan::walk(reader, limits, &mut scratch, work, |_, bytes| {
            if !bytes.starts_with(b"WRC11BLB") {
                return Ok(());
            }
            let fact = decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)?;
            if conflicts_with_metadata_reference(&fact, target) {
                return Err(BlobReclaimFailure::ConflictingSelectedFate);
            }
            Ok(())
        })?
    } else {
        reader
    };
    Ok(Some((
        reader,
        manifest_record,
        sha256,
        drop_set.count(),
        proof,
    )))
}

#[cfg(test)]
#[path = "manifest_residue/tests.rs"]
mod tests;
