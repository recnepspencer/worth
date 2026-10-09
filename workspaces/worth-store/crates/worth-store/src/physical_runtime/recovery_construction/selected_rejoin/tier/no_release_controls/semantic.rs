//! One failed-ingest source, manifest, descriptor and reservation join for
//! both ordinary NoRelease and completed historical rejoin.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobAbandonmentReasonV1, BlobGenerationPublicationV1, BlobSessionAbandonedV1,
    BlobSessionDeclarationV1, FailedIngestReclaimBasisV1, OriginalDropReservedV1,
    PersistedRecordIdentity,
};

use super::super::super::SelectedMediaRejoinDenial as Denial;

#[derive(Clone, Copy)]
pub(super) struct Manifest {
    pub(super) store: [u8; 16],
    pub(super) attempt: [u8; 16],
    pub(super) basis: FailedIngestReclaimBasisV1,
    pub(super) digest: [u8; 32],
    pub(super) count: u16,
    pub(super) selected_slot: Option<u64>,
}

#[derive(Clone, Copy)]
pub(super) struct Descriptor {
    pub(super) record: PersistedRecordIdentity,
    pub(super) frame_digest: [u8; 32],
    pub(super) store: [u8; 16],
    pub(super) attempt: [u8; 16],
    pub(super) basis_digest: [u8; 32],
    pub(super) manifest: PersistedRecordIdentity,
    pub(super) manifest_digest: [u8; 32],
    pub(super) count: u16,
    pub(super) source_generation: u64,
    pub(super) candidate_generation: u64,
}

pub(super) fn descriptor_matches_manifest(
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

pub(super) fn verify_reservation(
    manifests: &[(PersistedRecordIdentity, Manifest)],
    reservation: OriginalDropReservedV1,
    store: [u8; 16],
    selected_generation: u64,
) -> Result<(), Denial> {
    let manifest = manifests
        .binary_search_by_key(&reservation.manifest_record(), |(record, _)| *record)
        .ok()
        .map(|index| &manifests[index].1)
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

pub(super) fn source_matches(
    basis: FailedIngestReclaimBasisV1,
    declaration: BlobSessionDeclarationV1,
    declaration_bytes: &[u8],
    abandoned: BlobSessionAbandonedV1,
    abandoned_bytes: &[u8],
    store: [u8; 16],
    checkpoint_sequence: u64,
    publications: &[BlobGenerationPublicationV1],
) -> bool {
    if declaration.store() != store
        || abandoned.store() != store
        || declaration.session() != basis.session()
        || abandoned.session() != basis.session()
        || abandoned.declaration_record() != basis.declaration_record()
        || abandoned.declaration_digest() != basis.declaration_frame_sha256()
        || <[u8; 32]>::from(Sha256::digest(declaration_bytes)) != basis.declaration_frame_sha256()
        || <[u8; 32]>::from(Sha256::digest(abandoned_bytes)) != basis.abandoned_frame_sha256()
        || publications
            .iter()
            .any(|value| value.session() == basis.session())
    {
        return false;
    }
    if let BlobAbandonmentReasonV1::CheckpointExpired {
        checkpoint_sequence: expired,
    } = abandoned.reason()
    {
        if declaration.max_checkpoint_sequence() >= expired.get()
            || checkpoint_sequence < expired.get()
        {
            return false;
        }
    }
    true
}
