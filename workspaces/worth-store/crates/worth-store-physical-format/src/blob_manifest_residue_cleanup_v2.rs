//! Versioned manifest-only cleanup with an explicit original-drop proof kind.
//! A NeverReserved claim is representation, not authority: C8 must join it to
//! selected V2 manifest custody and exclude every Reserved transition.

use crate::PersistedRecordIdentity;

pub const BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN: &[u8] =
    b"store.physical.blob-manifest-residue-cleanup.v2";
const BODY_BYTES: usize = 306;
const WIRE_BYTES: usize = 8 + BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN.len() + BODY_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalDropProofV1 {
    ProvenNoEffect {
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
        reserved: Option<ReservedDropRecordV1>,
    },
    NeverReserved,
    /// A recovered, complete original-drop registry proved that the selected
    /// reservation never acquired a durable binding for this exact material.
    RecoveredNoBinding {
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
        reserved: ReservedDropRecordV1,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservedDropRecordV1 {
    record: PersistedRecordIdentity,
    frame_sha256: [u8; 32],
}

impl ReservedDropRecordV1 {
    pub fn new(record: PersistedRecordIdentity, frame_sha256: [u8; 32]) -> Option<Self> {
        (frame_sha256 != [0; 32]).then_some(Self {
            record,
            frame_sha256,
        })
    }
    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }
    pub const fn frame_sha256(self) -> [u8; 32] {
        self.frame_sha256
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobManifestResidueCleanup {
    V1(crate::BlobManifestResidueCleanupV1),
    V2(BlobManifestResidueCleanupV2),
}

impl BlobManifestResidueCleanup {
    pub fn decode(bytes: &[u8]) -> Result<Self, crate::BlobManifestResidueCleanupDenial> {
        if payload_is_blob_manifest_residue_cleanup_v2(bytes) {
            return BlobManifestResidueCleanupV2::decode(bytes).map(Self::V2);
        }
        crate::BlobManifestResidueCleanupV1::decode(bytes).map(Self::V1)
    }
    pub fn encode(self) -> Vec<u8> {
        match self {
            Self::V1(value) => value.encode(),
            Self::V2(value) => value.encode(),
        }
    }
    pub const fn phase(self) -> crate::BlobManifestResidueCleanupPhaseV1 {
        match self {
            Self::V1(value) => value.phase(),
            Self::V2(value) => value.phase(),
        }
    }
    pub const fn store(self) -> [u8; 16] {
        match self {
            Self::V1(value) => value.store(),
            Self::V2(value) => value.store(),
        }
    }
    pub const fn reclaim_attempt(self) -> [u8; 16] {
        match self {
            Self::V1(value) => value.reclaim_attempt(),
            Self::V2(value) => value.reclaim_attempt(),
        }
    }
    pub const fn manifest_record(self) -> PersistedRecordIdentity {
        match self {
            Self::V1(value) => value.manifest_record(),
            Self::V2(value) => value.manifest_record(),
        }
    }
    pub const fn manifest_frame_sha256(self) -> [u8; 32] {
        match self {
            Self::V1(value) => value.manifest_frame_sha256(),
            Self::V2(value) => value.manifest_frame_sha256(),
        }
    }
    pub const fn source_basis_digest(self) -> [u8; 32] {
        match self {
            Self::V1(value) => value.source_basis_digest(),
            Self::V2(value) => value.source_basis_digest(),
        }
    }
    pub const fn proof(self) -> OriginalDropProofV1 {
        match self {
            Self::V1(value) => OriginalDropProofV1::ProvenNoEffect {
                idempotency: value.drop_idempotency_identity(),
                fingerprint: value.drop_request_fingerprint(),
                reserved: None,
            },
            Self::V2(value) => value.proof(),
        }
    }
    pub const fn reserved_record(self) -> Option<ReservedDropRecordV1> {
        match self.proof() {
            OriginalDropProofV1::ProvenNoEffect { reserved, .. } => reserved,
            OriginalDropProofV1::NeverReserved => None,
            OriginalDropProofV1::RecoveredNoBinding { reserved, .. } => Some(reserved),
        }
    }
    pub const fn source_root_generation(self) -> u64 {
        match self {
            Self::V1(value) => value.source_root_generation(),
            Self::V2(value) => value.source_root_generation(),
        }
    }
    pub const fn candidate_root_generation(self) -> u64 {
        match self {
            Self::V1(value) => value.candidate_root_generation(),
            Self::V2(value) => value.candidate_root_generation(),
        }
    }
    pub const fn candidate_root_sha256(self) -> [u8; 32] {
        match self {
            Self::V1(value) => value.candidate_root_sha256(),
            Self::V2(value) => value.candidate_root_sha256(),
        }
    }
    pub const fn retained_metadata_bytes(self) -> u64 {
        match self {
            Self::V1(value) => value.retained_metadata_bytes(),
            Self::V2(value) => value.retained_metadata_bytes(),
        }
    }
    pub const fn publication(self) -> u64 {
        match self {
            Self::V1(value) => value.publication(),
            Self::V2(value) => value.publication(),
        }
    }
    pub const fn completed(self) -> Self {
        match self {
            Self::V1(value) => Self::V1(value.completed()),
            Self::V2(value) => Self::V2(value.completed()),
        }
    }
}

pub fn payload_is_blob_manifest_residue_cleanup_any(bytes: &[u8]) -> bool {
    crate::payload_is_blob_manifest_residue_cleanup(bytes)
        || payload_is_blob_manifest_residue_cleanup_v2(bytes)
}

impl From<crate::BlobManifestResidueCleanupV1> for BlobManifestResidueCleanup {
    fn from(value: crate::BlobManifestResidueCleanupV1) -> Self {
        Self::V1(value)
    }
}

impl From<BlobManifestResidueCleanupV2> for BlobManifestResidueCleanup {
    fn from(value: BlobManifestResidueCleanupV2) -> Self {
        Self::V2(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobManifestResidueCleanupV2 {
    phase: crate::BlobManifestResidueCleanupPhaseV1,
    store: [u8; 16],
    reclaim_attempt: [u8; 16],
    manifest_record: PersistedRecordIdentity,
    manifest_frame_sha256: [u8; 32],
    source_basis_digest: [u8; 32],
    proof: OriginalDropProofV1,
    source_root_generation: u64,
    candidate_root_generation: u64,
    candidate_root_sha256: [u8; 32],
    retained_metadata_bytes: u64,
    publication: u64,
}

impl BlobManifestResidueCleanupV2 {
    #[allow(clippy::too_many_arguments)]
    pub fn intent(
        store: [u8; 16],
        reclaim_attempt: [u8; 16],
        manifest_record: PersistedRecordIdentity,
        manifest_frame_sha256: [u8; 32],
        source_basis_digest: [u8; 32],
        proof: OriginalDropProofV1,
        source_root_generation: u64,
        candidate_root_generation: u64,
        candidate_root_sha256: [u8; 32],
        retained_metadata_bytes: u64,
        publication: u64,
    ) -> Result<Self, crate::BlobManifestResidueCleanupDenial> {
        use crate::BlobManifestResidueCleanupDenial::InvalidBinding;
        if store == [0; 16]
            || reclaim_attempt == [0; 16]
            || manifest_frame_sha256 == [0; 32]
            || source_basis_digest == [0; 32]
            || candidate_root_sha256 == [0; 32]
            || source_root_generation == 0
            || source_root_generation.checked_add(1) != Some(candidate_root_generation)
            || retained_metadata_bytes == 0
            || publication == 0
            || matches!(proof, OriginalDropProofV1::ProvenNoEffect { idempotency, fingerprint, reserved }
                if idempotency == [0; 32] || fingerprint == [0; 32]
                    || reserved.is_none_or(|value| value.record == manifest_record))
            || matches!(proof, OriginalDropProofV1::RecoveredNoBinding { idempotency, fingerprint, reserved }
                if idempotency == [0; 32] || fingerprint == [0; 32]
                    || reserved.record == manifest_record)
        {
            return Err(InvalidBinding);
        }
        Ok(Self {
            phase: crate::BlobManifestResidueCleanupPhaseV1::Intent,
            store,
            reclaim_attempt,
            manifest_record,
            manifest_frame_sha256,
            source_basis_digest,
            proof,
            source_root_generation,
            candidate_root_generation,
            candidate_root_sha256,
            retained_metadata_bytes,
            publication,
        })
    }

    pub const fn completed(mut self) -> Self {
        self.phase = crate::BlobManifestResidueCleanupPhaseV1::Completed;
        self
    }

    pub const fn phase(self) -> crate::BlobManifestResidueCleanupPhaseV1 {
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
    pub const fn proof(self) -> OriginalDropProofV1 {
        self.proof
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

pub fn payload_is_blob_manifest_residue_cleanup_v2(bytes: &[u8]) -> bool {
    bytes.len() >= 8 + BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN.len()
        && bytes[..8] == (BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN.len() as u64).to_le_bytes()
        && &bytes[8..8 + BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN.len()]
            == BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN
}

mod wire;

#[cfg(test)]
#[path = "blob_manifest_residue_cleanup_v2/tests.rs"]
mod tests;
