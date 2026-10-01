use sha2::{Digest, Sha256};

use crate::PersistedRecordIdentity;

use super::super::envelope::{nonzero_16, nonzero_32};
use super::super::BlobRecordDenial;

const SOURCE_BASIS_DOMAIN: &[u8] = b"store.physical.failed-ingest-reclaim-basis.v1";

/// Concrete selected facts to be re-admitted by Store and recovery before a drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailedIngestReclaimBasisV1 {
    session: [u8; 16],
    declaration_record: PersistedRecordIdentity,
    declaration_frame_sha256: [u8; 32],
    abandoned_record: PersistedRecordIdentity,
    abandoned_frame_sha256: [u8; 32],
}

impl FailedIngestReclaimBasisV1 {
    pub fn new(
        session: [u8; 16],
        declaration_record: PersistedRecordIdentity,
        declaration_frame_sha256: [u8; 32],
        abandoned_record: PersistedRecordIdentity,
        abandoned_frame_sha256: [u8; 32],
    ) -> Result<Self, BlobRecordDenial> {
        if declaration_record == abandoned_record {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        Ok(Self {
            session: nonzero_16(session)?,
            declaration_record,
            declaration_frame_sha256: nonzero_32(declaration_frame_sha256)?,
            abandoned_record,
            abandoned_frame_sha256: nonzero_32(abandoned_frame_sha256)?,
        })
    }

    pub fn digest(self, store: [u8; 16]) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(SOURCE_BASIS_DOMAIN);
        hash.update(store);
        hash.update(self.session);
        hash.update(self.declaration_record.allocation_epoch());
        hash.update(self.declaration_record.ordinal().to_le_bytes());
        hash.update(self.declaration_frame_sha256);
        hash.update(self.abandoned_record.allocation_epoch());
        hash.update(self.abandoned_record.ordinal().to_le_bytes());
        hash.update(self.abandoned_frame_sha256);
        hash.finalize().into()
    }

    pub const fn session(self) -> [u8; 16] {
        self.session
    }
    pub const fn declaration_record(self) -> PersistedRecordIdentity {
        self.declaration_record
    }
    pub const fn declaration_frame_sha256(self) -> [u8; 32] {
        self.declaration_frame_sha256
    }
    pub const fn abandoned_record(self) -> PersistedRecordIdentity {
        self.abandoned_record
    }
    pub const fn abandoned_frame_sha256(self) -> [u8; 32] {
        self.abandoned_frame_sha256
    }

    pub(super) fn encode_into(self, target: &mut Vec<u8>) {
        self.visit_bytes(&mut |part| target.extend_from_slice(part));
    }

    pub(super) fn visit_bytes(self, emit: &mut dyn FnMut(&[u8])) {
        emit(&self.session);
        emit(&self.declaration_record.allocation_epoch());
        emit(&self.declaration_record.ordinal().to_le_bytes());
        emit(&self.declaration_frame_sha256);
        emit(&self.abandoned_record.allocation_epoch());
        emit(&self.abandoned_record.ordinal().to_le_bytes());
        emit(&self.abandoned_frame_sha256);
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        if bytes.len() != 128 {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        Self::new(
            bytes[..16].try_into().expect("fixed session"),
            read_record(&bytes[16..40])?,
            bytes[40..72].try_into().expect("fixed digest"),
            read_record(&bytes[72..96])?,
            bytes[96..128].try_into().expect("fixed digest"),
        )
    }
}

pub(super) fn write_record(target: &mut Vec<u8>, record: PersistedRecordIdentity) {
    target.extend_from_slice(&record.allocation_epoch());
    target.extend_from_slice(&record.ordinal().to_le_bytes());
}

pub(super) fn read_record(bytes: &[u8]) -> Result<PersistedRecordIdentity, BlobRecordDenial> {
    if bytes.len() != 24 {
        return Err(BlobRecordDenial::LengthMismatch);
    }
    PersistedRecordIdentity::new(
        bytes[..16].try_into().expect("fixed epoch"),
        u64::from_le_bytes(bytes[16..24].try_into().expect("fixed ordinal")),
    )
    .ok_or(BlobRecordDenial::InvalidIdentity)
}
