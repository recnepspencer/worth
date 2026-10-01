//! NoRelease admits only selected failed-ingest controls with an observed source.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobAbandonmentReasonV1, BlobReclaimSourceKind, BlobRecordKind,
    BlobRecordV1, CurrentPhysicalRecordPlacement, FailedIngestReclaimBasisV1,
    OriginalDropReservedV1, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    SelectedRecordContentClass,
};

use super::super::control_frames::SelectedArtifactSlice;
use super::super::{SelectedMediaRejoinDenial as Denial, MAX_CONTROL_FRAME_BYTES};
use super::no_release_frame;

struct Manifest {
    store: [u8; 16],
    attempt: [u8; 16],
    basis: FailedIngestReclaimBasisV1,
    digest: [u8; 32],
    count: u16,
    selected_slot: Option<u64>,
}

#[derive(Clone, Copy)]
struct Descriptor {
    record: PersistedRecordIdentity,
    frame_digest: [u8; 32],
    store: [u8; 16],
    attempt: [u8; 16],
    basis_digest: [u8; 32],
    manifest: PersistedRecordIdentity,
    manifest_digest: [u8; 32],
    count: u16,
    source_generation: u64,
    candidate_generation: u64,
}

pub(super) fn verify(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    selected_generation: u64,
    checkpoint_sequence: u64,
    slices: &mut Vec<SelectedArtifactSlice>,
) -> Result<BTreeMap<PersistedRecordIdentity, ([u8; 32], u64)>, Denial> {
    verify_bounded(
        discovery,
        routes,
        format,
        selected_generation,
        checkpoint_sequence,
        slices,
        usize::MAX,
    )
}

pub(super) fn verify_bounded(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    selected_generation: u64,
    checkpoint_sequence: u64,
    slices: &mut Vec<SelectedArtifactSlice>,
    max_slices: usize,
) -> Result<BTreeMap<PersistedRecordIdentity, ([u8; 32], u64)>, Denial> {
    let mut manifests = BTreeMap::new();
    let mut descriptors = Vec::new();
    let mut reservations = Vec::new();
    let mut publications = Vec::new();
    for (&record, &route) in routes {
        let SelectedRecordContentClass::Blob(kind) = route.content_class() else {
            if route.content_class() == SelectedRecordContentClass::UnknownLegacy {
                return Err(Denial::CertificateRoster);
            }
            continue;
        };
        match kind {
            BlobRecordKind::DropSetManifest
            | BlobRecordKind::DropSetManifestV2
            | BlobRecordKind::ReclaimDescriptor
            | BlobRecordKind::ReclaimDescriptorV2
            | BlobRecordKind::OriginalDropReserved
            | BlobRecordKind::GenerationPublished => {}
            BlobRecordKind::DropSetManifestV3 | BlobRecordKind::ReclaimDescriptorV3 => {
                return Err(Denial::CertificateRoster);
            }
            _ => continue,
        }
        let bytes = selected_frame(
            discovery,
            format,
            route,
            MAX_CONTROL_FRAME_BYTES,
            slices,
            max_slices,
        )?;
        let decoded = decode_blob_record(&bytes).map_err(|_| Denial::ControlFrame)?;
        if decoded.kind() != kind {
            return Err(Denial::ControlFrame);
        }
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        match decoded {
            BlobRecordV1::DropSetManifest(value) if value.encode() == bytes => {
                manifests.insert(
                    record,
                    Manifest {
                        store: value.store(),
                        attempt: value.reclaim_attempt(),
                        basis: value.source_basis(),
                        digest,
                        count: value.count(),
                        selected_slot: None,
                    },
                );
            }
            BlobRecordV1::DropSetManifestV2(value) if value.encode() == bytes => {
                manifests.insert(
                    record,
                    Manifest {
                        store: value.store(),
                        attempt: value.reclaim_attempt(),
                        basis: value.source_basis(),
                        digest,
                        count: value.count(),
                        selected_slot: Some(value.never_reserved_slot_generation()),
                    },
                );
            }
            BlobRecordV1::ReclaimDescriptor(value) if value.encode() == bytes => {
                descriptors.push(Descriptor {
                    record,
                    frame_digest: digest,
                    store: value.store(),
                    attempt: value.reclaim_attempt(),
                    basis_digest: value.source_basis_digest(),
                    manifest: value.manifest_record(),
                    manifest_digest: value.manifest_frame_sha256(),
                    count: value.manifest_count(),
                    source_generation: value.source_root_generation(),
                    candidate_generation: value.candidate_root_generation(),
                });
            }
            BlobRecordV1::ReclaimDescriptorV2(value)
                if value.encode() == bytes && failed_ingest_v2(value) =>
            {
                descriptors.push(Descriptor {
                    record,
                    frame_digest: digest,
                    store: value.store(),
                    attempt: value.reclaim_attempt(),
                    basis_digest: value.source_basis_digest(),
                    manifest: value.manifest_record(),
                    manifest_digest: value.manifest_frame_sha256(),
                    count: value.manifest_count(),
                    source_generation: value.source_root_generation(),
                    candidate_generation: value.candidate_root_generation(),
                });
            }
            BlobRecordV1::OriginalDropReserved(value) if value.encode() == bytes => {
                reservations.push(value);
            }
            BlobRecordV1::GenerationPublished(value) if value.encode() == bytes => {
                publications.push(value);
            }
            _ => return Err(Denial::CertificateRoster),
        }
    }
    let store = discovery.store_identity().bytes();
    for manifest in manifests.values() {
        if manifest.store != store
            || manifest
                .selected_slot
                .is_some_and(|slot| slot > selected_generation)
        {
            return Err(Denial::CertificateRoster);
        }
        verify_failed_ingest_source(
            discovery,
            routes,
            format,
            manifest.basis,
            store,
            checkpoint_sequence,
            &publications,
            slices,
            max_slices,
        )?;
    }
    let mut proven_drops = BTreeMap::new();
    for descriptor in descriptors {
        let manifest = manifests
            .get(&descriptor.manifest)
            .ok_or(Denial::CertificateRoster)?;
        if !descriptor_matches_manifest(&descriptor, manifest, store, selected_generation) {
            return Err(Denial::CertificateRoster);
        }
        proven_drops.insert(
            descriptor.record,
            (descriptor.frame_digest, descriptor.candidate_generation),
        );
    }
    for reservation in reservations {
        verify_reservation(&manifests, reservation, store, selected_generation)?;
    }
    Ok(proven_drops)
}

fn failed_ingest_v2(value: worth_store_physical_format::BlobReclaimDescriptorV2) -> bool {
    value.source_kind() == BlobReclaimSourceKind::FailedIngest && value.predecessor().is_none()
}

fn descriptor_matches_manifest(
    descriptor: &Descriptor,
    manifest: &Manifest,
    store: [u8; 16],
    selected_generation: u64,
) -> bool {
    descriptor.store == store
        && descriptor.store == manifest.store
        && descriptor.attempt == manifest.attempt
        && descriptor.basis_digest == manifest.basis.digest(store)
        && descriptor.manifest_digest == manifest.digest
        && descriptor.count == manifest.count
        && manifest
            .selected_slot
            .is_none_or(|slot| slot < descriptor.source_generation)
        && descriptor.source_generation < descriptor.candidate_generation
        && descriptor.candidate_generation <= selected_generation
}

fn verify_reservation(
    manifests: &BTreeMap<PersistedRecordIdentity, Manifest>,
    reservation: OriginalDropReservedV1,
    store: [u8; 16],
    selected_generation: u64,
) -> Result<(), Denial> {
    let manifest = manifests
        .get(&reservation.manifest_record())
        .ok_or(Denial::CertificateRoster)?;
    if reservation.store() != store
        || reservation.store() != manifest.store
        || reservation.reclaim_attempt() != manifest.attempt
        || reservation.source_basis_digest() != manifest.basis.digest(store)
        || reservation.manifest_frame_sha256() != manifest.digest
        || manifest.selected_slot != Some(reservation.manifest_selected_generation())
        || reservation.reserved_selected_generation() > selected_generation
    {
        return Err(Denial::CertificateRoster);
    }
    Ok(())
}

fn verify_failed_ingest_source(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    basis: FailedIngestReclaimBasisV1,
    store: [u8; 16],
    checkpoint_sequence: u64,
    publications: &[worth_store_physical_format::BlobGenerationPublicationV1],
    slices: &mut Vec<SelectedArtifactSlice>,
    max_slices: usize,
) -> Result<(), Denial> {
    let declaration_route = routes
        .get(&basis.declaration_record())
        .ok_or(Denial::CertificateRoster)?;
    let abandoned_route = routes
        .get(&basis.abandoned_record())
        .ok_or(Denial::CertificateRoster)?;
    if declaration_route.content_class()
        != SelectedRecordContentClass::Blob(BlobRecordKind::SessionDeclared)
        || abandoned_route.content_class()
            != SelectedRecordContentClass::Blob(BlobRecordKind::SessionAbandoned)
    {
        return Err(Denial::CertificateRoster);
    }
    let declaration_bytes = selected_frame(
        discovery,
        format,
        *declaration_route,
        4096,
        slices,
        max_slices,
    )?;
    let abandoned_bytes = selected_frame(
        discovery,
        format,
        *abandoned_route,
        4096,
        slices,
        max_slices,
    )?;
    let BlobRecordV1::SessionDeclared(declaration) =
        decode_blob_record(&declaration_bytes).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let BlobRecordV1::SessionAbandoned(abandoned) =
        decode_blob_record(&abandoned_bytes).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    if declaration.encode() != declaration_bytes
        || abandoned.encode() != abandoned_bytes
        || declaration.store() != store
        || abandoned.store() != store
        || declaration.session() != basis.session()
        || abandoned.session() != basis.session()
        || abandoned.declaration_record() != basis.declaration_record()
        || abandoned.declaration_digest() != basis.declaration_frame_sha256()
        || <[u8; 32]>::from(Sha256::digest(&declaration_bytes)) != basis.declaration_frame_sha256()
        || <[u8; 32]>::from(Sha256::digest(&abandoned_bytes)) != basis.abandoned_frame_sha256()
        || publications
            .iter()
            .any(|value| value.session() == basis.session())
    {
        return Err(Denial::CertificateRoster);
    }
    if let BlobAbandonmentReasonV1::CheckpointExpired {
        checkpoint_sequence: expired,
    } = abandoned.reason()
    {
        if declaration.max_checkpoint_sequence() >= expired.get()
            || checkpoint_sequence < expired.get()
        {
            return Err(Denial::CertificateRoster);
        }
    }
    Ok(())
}

fn selected_frame(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    route: CurrentPhysicalRecordPlacement,
    maximum: u64,
    slices: &mut Vec<SelectedArtifactSlice>,
    max_slices: usize,
) -> Result<Vec<u8>, Denial> {
    let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        return Err(Denial::UnsupportedSelectedPlacement);
    };
    no_release_frame::read_with_slice_limit(discovery, format, extent, maximum, slices, max_slices)
}

#[cfg(test)]
#[path = "no_release_controls/tests.rs"]
mod tests;
