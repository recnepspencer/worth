//! A maintenance WAL intent for removing one selected reclaim custody record.
//! This is separate from C5 data-frame redo and from a v7 records-dropped
//! descriptor: it never authorizes removal of the manifest's payload IDs.

use crate::PersistedRecordIdentity;

pub const BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN: &[u8] =
    b"store.physical.blob-manifest-residue-cleanup.v1";
const BODY_BYTES: usize = 248;
const WIRE_BYTES: usize = 8 + BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len() + 1 + BODY_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobManifestResidueCleanupPhaseV1 {
    Intent,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobManifestResidueCleanupDenial {
    Malformed,
    InvalidBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobManifestResidueCleanupV1 {
    phase: BlobManifestResidueCleanupPhaseV1,
    store: [u8; 16],
    reclaim_attempt: [u8; 16],
    manifest_record: PersistedRecordIdentity,
    manifest_frame_sha256: [u8; 32],
    source_basis_digest: [u8; 32],
    drop_idempotency_identity: [u8; 32],
    drop_request_fingerprint: [u8; 32],
    source_root_generation: u64,
    candidate_root_generation: u64,
    candidate_root_sha256: [u8; 32],
    retained_metadata_bytes: u64,
    publication: u64,
}

impl BlobManifestResidueCleanupV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn intent(
        store: [u8; 16],
        reclaim_attempt: [u8; 16],
        manifest_record: PersistedRecordIdentity,
        manifest_frame_sha256: [u8; 32],
        source_basis_digest: [u8; 32],
        drop_idempotency_identity: [u8; 32],
        drop_request_fingerprint: [u8; 32],
        source_root_generation: u64,
        candidate_root_generation: u64,
        candidate_root_sha256: [u8; 32],
        retained_metadata_bytes: u64,
        publication: u64,
    ) -> Result<Self, BlobManifestResidueCleanupDenial> {
        if store == [0; 16]
            || reclaim_attempt == [0; 16]
            || manifest_frame_sha256 == [0; 32]
            || source_basis_digest == [0; 32]
            || drop_idempotency_identity == [0; 32]
            || drop_request_fingerprint == [0; 32]
            || candidate_root_sha256 == [0; 32]
            || source_root_generation == 0
            || source_root_generation.checked_add(1) != Some(candidate_root_generation)
            || retained_metadata_bytes == 0
            || publication == 0
        {
            return Err(BlobManifestResidueCleanupDenial::InvalidBinding);
        }
        Ok(Self {
            phase: BlobManifestResidueCleanupPhaseV1::Intent,
            store,
            reclaim_attempt,
            manifest_record,
            manifest_frame_sha256,
            source_basis_digest,
            drop_idempotency_identity,
            drop_request_fingerprint,
            source_root_generation,
            candidate_root_generation,
            candidate_root_sha256,
            retained_metadata_bytes,
            publication,
        })
    }

    pub const fn completed(mut self) -> Self {
        self.phase = BlobManifestResidueCleanupPhaseV1::Completed;
        self
    }

    pub fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(WIRE_BYTES);
        bytes.extend_from_slice(&(BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len() as u64).to_le_bytes());
        bytes.extend_from_slice(BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN);
        bytes.push(match self.phase {
            BlobManifestResidueCleanupPhaseV1::Intent => 1,
            BlobManifestResidueCleanupPhaseV1::Completed => 2,
        });
        bytes.extend_from_slice(&self.store);
        bytes.extend_from_slice(&self.reclaim_attempt);
        bytes.extend_from_slice(&self.manifest_record.allocation_epoch());
        bytes.extend_from_slice(&self.manifest_record.ordinal().to_le_bytes());
        bytes.extend_from_slice(&self.manifest_frame_sha256);
        bytes.extend_from_slice(&self.source_basis_digest);
        bytes.extend_from_slice(&self.drop_idempotency_identity);
        bytes.extend_from_slice(&self.drop_request_fingerprint);
        bytes.extend_from_slice(&self.source_root_generation.to_le_bytes());
        bytes.extend_from_slice(&self.candidate_root_generation.to_le_bytes());
        bytes.extend_from_slice(&self.candidate_root_sha256);
        bytes.extend_from_slice(&self.retained_metadata_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.publication.to_le_bytes());
        debug_assert_eq!(bytes.len(), WIRE_BYTES);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobManifestResidueCleanupDenial> {
        if bytes.len() != WIRE_BYTES
            || bytes[..8] != (BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len() as u64).to_le_bytes()
            || &bytes[8..8 + BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len()]
                != BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN
        {
            return Err(BlobManifestResidueCleanupDenial::Malformed);
        }
        let mut cursor = 8 + BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len();
        let phase = match take::<1>(bytes, &mut cursor)[0] {
            1 => BlobManifestResidueCleanupPhaseV1::Intent,
            2 => BlobManifestResidueCleanupPhaseV1::Completed,
            _ => return Err(BlobManifestResidueCleanupDenial::Malformed),
        };
        let store = take::<16>(bytes, &mut cursor);
        let attempt = take::<16>(bytes, &mut cursor);
        let epoch = take::<16>(bytes, &mut cursor);
        let ordinal = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let manifest_record = PersistedRecordIdentity::new(epoch, ordinal)
            .ok_or(BlobManifestResidueCleanupDenial::InvalidBinding)?;
        let manifest_sha = take::<32>(bytes, &mut cursor);
        let source_digest = take::<32>(bytes, &mut cursor);
        let drop_idempotency = take::<32>(bytes, &mut cursor);
        let drop_fingerprint = take::<32>(bytes, &mut cursor);
        let source = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let candidate = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let candidate_sha = take::<32>(bytes, &mut cursor);
        let metadata_bytes = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let publication = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let mut value = Self::intent(
            store,
            attempt,
            manifest_record,
            manifest_sha,
            source_digest,
            drop_idempotency,
            drop_fingerprint,
            source,
            candidate,
            candidate_sha,
            metadata_bytes,
            publication,
        )?;
        value.phase = phase;
        Ok(value)
    }

    pub const fn phase(self) -> BlobManifestResidueCleanupPhaseV1 {
        self.phase
    }
    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn reclaim_attempt(self) -> [u8; 16] {
        self.reclaim_attempt
    }
    pub const fn manifest_record(self) -> PersistedRecordIdentity {
        self.manifest_record
    }
    pub const fn manifest_frame_sha256(self) -> [u8; 32] {
        self.manifest_frame_sha256
    }
    pub const fn source_basis_digest(self) -> [u8; 32] {
        self.source_basis_digest
    }
    pub const fn drop_idempotency_identity(self) -> [u8; 32] {
        self.drop_idempotency_identity
    }
    pub const fn drop_request_fingerprint(self) -> [u8; 32] {
        self.drop_request_fingerprint
    }
    pub const fn source_root_generation(self) -> u64 {
        self.source_root_generation
    }
    pub const fn candidate_root_generation(self) -> u64 {
        self.candidate_root_generation
    }
    pub const fn candidate_root_sha256(self) -> [u8; 32] {
        self.candidate_root_sha256
    }
    pub const fn retained_metadata_bytes(self) -> u64 {
        self.retained_metadata_bytes
    }
    pub const fn publication(self) -> u64 {
        self.publication
    }
}

pub fn payload_is_blob_manifest_residue_cleanup(bytes: &[u8]) -> bool {
    bytes.len() >= 8 + BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len()
        && bytes[..8] == (BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len() as u64).to_le_bytes()
        && &bytes[8..8 + BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len()]
            == BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN
}

fn take<const N: usize>(bytes: &[u8], cursor: &mut usize) -> [u8; N] {
    let end = *cursor + N;
    let value = bytes[*cursor..end]
        .try_into()
        .expect("fixed wire length admitted");
    *cursor = end;
    value
}

#[cfg(test)]
#[path = "blob_manifest_residue_cleanup/tests.rs"]
mod tests;
