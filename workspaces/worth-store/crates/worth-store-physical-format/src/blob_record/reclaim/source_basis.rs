use sha2::{Digest, Sha256};

use crate::PersistedRecordIdentity;

use super::super::envelope::nonzero_32;
use super::super::generation::GENERATION_PUBLICATION_FRAME_BYTES;
use super::super::{BlobGenerationPublicationV1, BlobRecordDenial};
use super::basis::read_record;
use super::FailedIngestReclaimBasisV1;

const RELEASED_SOURCE_DOMAIN: &[u8] = b"store.physical.released-generation-reclaim-basis.v1";
const RELEASED_SOURCE_BYTES: usize = 24 + 32 + 32 + GENERATION_PUBLICATION_FRAME_BYTES;
const FAILED_INGEST_SOURCE_BYTES: usize = 128;
/// The longest encoded source basis is the released-generation form.
pub(super) const MAXIMUM_SOURCE_BASIS_BYTES: usize = RELEASED_SOURCE_BYTES;
const _: () = assert!(FAILED_INGEST_SOURCE_BYTES <= MAXIMUM_SOURCE_BASIS_BYTES);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlobReclaimSourceKind {
    FailedIngest = 1,
    ReleasedGeneration = 2,
}

impl BlobReclaimSourceKind {
    pub(super) fn decode(tag: u8) -> Result<Self, BlobRecordDenial> {
        match tag {
            1 => Ok(Self::FailedIngest),
            2 => Ok(Self::ReleasedGeneration),
            _ => Err(BlobRecordDenial::InvalidReclaimSource),
        }
    }
}

/// Selected publication custody plus the issuer's evidence identity. This
/// representation is never itself a release grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleasedGenerationReclaimBasisV1 {
    publication: BlobGenerationPublicationV1,
    publication_record: PersistedRecordIdentity,
    publication_frame_sha256: [u8; 32],
    issuer_evidence_sha256: [u8; 32],
}

impl ReleasedGenerationReclaimBasisV1 {
    pub fn new(
        publication: BlobGenerationPublicationV1,
        publication_record: PersistedRecordIdentity,
        publication_frame_sha256: [u8; 32],
        issuer_evidence_sha256: [u8; 32],
    ) -> Result<Self, BlobRecordDenial> {
        let mut hash = Sha256::new();
        publication.visit_canonical_frame(&mut |part| hash.update(part));
        let observed: [u8; 32] = hash.finalize().into();
        if observed != publication_frame_sha256 {
            return Err(BlobRecordDenial::InvalidReclaimSource);
        }
        Ok(Self {
            publication,
            publication_record,
            publication_frame_sha256: nonzero_32(publication_frame_sha256)?,
            issuer_evidence_sha256: nonzero_32(issuer_evidence_sha256)?,
        })
    }

    pub const fn object(self) -> [u8; 16] {
        self.publication.object()
    }
    pub const fn generation(self) -> u64 {
        self.publication.generation()
    }
    pub const fn session(self) -> [u8; 16] {
        self.publication.session()
    }
    pub const fn publication(self) -> BlobGenerationPublicationV1 {
        self.publication
    }
    pub const fn publication_record(self) -> PersistedRecordIdentity {
        self.publication_record
    }
    pub const fn publication_frame_sha256(self) -> [u8; 32] {
        self.publication_frame_sha256
    }
    pub const fn issuer_evidence_sha256(self) -> [u8; 32] {
        self.issuer_evidence_sha256
    }

    fn encode_into(self, target: &mut Vec<u8>) {
        self.visit_bytes(&mut |part| target.extend_from_slice(part));
    }

    fn visit_bytes(self, emit: &mut dyn FnMut(&[u8])) {
        emit(&self.publication_record.allocation_epoch());
        emit(&self.publication_record.ordinal().to_le_bytes());
        emit(&self.publication_frame_sha256);
        emit(&self.issuer_evidence_sha256);
        self.publication.visit_canonical_frame(emit);
    }

    /// Canonical source-basis bytes for a WAL-bound release-head transition.
    /// These bytes are a digest witness, not authority over selected media.
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(RELEASED_SOURCE_BYTES);
        self.encode_into(&mut bytes);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        if bytes.len() != RELEASED_SOURCE_BYTES {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        Self::new(
            BlobGenerationPublicationV1::decode(&bytes[88..])?,
            read_record(&bytes[0..24])?,
            bytes[24..56].try_into().expect("fixed frame digest"),
            bytes[56..88].try_into().expect("fixed evidence digest"),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReclaimSourceBasisV1 {
    FailedIngest(FailedIngestReclaimBasisV1),
    ReleasedGeneration(ReleasedGenerationReclaimBasisV1),
}

impl BlobReclaimSourceBasisV1 {
    pub const fn kind(self) -> BlobReclaimSourceKind {
        match self {
            Self::FailedIngest(_) => BlobReclaimSourceKind::FailedIngest,
            Self::ReleasedGeneration(_) => BlobReclaimSourceKind::ReleasedGeneration,
        }
    }

    pub fn digest(self, store: [u8; 16]) -> [u8; 32] {
        match self {
            Self::FailedIngest(basis) => basis.digest(store),
            Self::ReleasedGeneration(basis) => {
                let mut hash = Sha256::new();
                hash.update(RELEASED_SOURCE_DOMAIN);
                hash.update(store);
                hash.update([BlobReclaimSourceKind::ReleasedGeneration as u8]);
                basis.visit_bytes(&mut |part| hash.update(part));
                hash.finalize().into()
            }
        }
    }

    pub(super) fn encoded_len(self) -> usize {
        match self {
            Self::FailedIngest(_) => FAILED_INGEST_SOURCE_BYTES,
            Self::ReleasedGeneration(_) => RELEASED_SOURCE_BYTES,
        }
    }

    pub(super) fn visit_bytes(self, emit: &mut dyn FnMut(&[u8])) {
        match self {
            Self::FailedIngest(basis) => basis.visit_bytes(emit),
            Self::ReleasedGeneration(basis) => basis.visit_bytes(emit),
        }
    }

    pub(super) fn decode(
        kind: BlobReclaimSourceKind,
        bytes: &[u8],
    ) -> Result<Self, BlobRecordDenial> {
        match kind {
            BlobReclaimSourceKind::FailedIngest => {
                FailedIngestReclaimBasisV1::decode(bytes).map(Self::FailedIngest)
            }
            BlobReclaimSourceKind::ReleasedGeneration => {
                ReleasedGenerationReclaimBasisV1::decode(bytes).map(Self::ReleasedGeneration)
            }
        }
    }
}
