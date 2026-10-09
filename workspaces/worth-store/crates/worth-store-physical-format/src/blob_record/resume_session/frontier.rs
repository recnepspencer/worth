use crate::PersistedRecordIdentity;

use super::super::envelope::{encode, nonzero_16, nonzero_32};
use super::super::{BlobRecordDenial, BlobRecordKind};

const PAYLOAD_BYTES: usize = 160;

/// A durable selected-prefix claim, not a resume authority by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobSessionFrontierV1 {
    store: [u8; 16],
    session: [u8; 16],
    declaration_record: PersistedRecordIdentity,
    declaration_digest: [u8; 32],
    next_chunk_ordinal: u64,
    durable_bytes: u64,
    last_chunk_record: PersistedRecordIdentity,
    last_chunk_digest: [u8; 32],
}

impl BlobSessionFrontierV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: [u8; 16],
        session: [u8; 16],
        declaration_record: PersistedRecordIdentity,
        declaration_digest: [u8; 32],
        next_chunk_ordinal: u64,
        durable_bytes: u64,
        last_chunk_record: PersistedRecordIdentity,
        last_chunk_digest: [u8; 32],
    ) -> Result<Self, BlobRecordDenial> {
        if next_chunk_ordinal == 0 || durable_bytes == 0 || declaration_record == last_chunk_record
        {
            return Err(BlobRecordDenial::InvalidFrontier);
        }
        Ok(Self {
            store: nonzero_16(store)?,
            session: nonzero_16(session)?,
            declaration_record,
            declaration_digest: nonzero_32(declaration_digest)?,
            next_chunk_ordinal,
            durable_bytes,
            last_chunk_record,
            last_chunk_digest: nonzero_32(last_chunk_digest)?,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(PAYLOAD_BYTES);
        payload.extend_from_slice(&self.store);
        payload.extend_from_slice(&self.session);
        write_record(&mut payload, self.declaration_record);
        payload.extend_from_slice(&self.declaration_digest);
        payload.extend_from_slice(&self.next_chunk_ordinal.to_le_bytes());
        payload.extend_from_slice(&self.durable_bytes.to_le_bytes());
        write_record(&mut payload, self.last_chunk_record);
        payload.extend_from_slice(&self.last_chunk_digest);
        encode(BlobRecordKind::SessionFrontier, &payload)
            .expect("fixed frontier fits control-frame ceiling")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::SessionFrontier {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(in crate::blob_record) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        if payload.len() != PAYLOAD_BYTES {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        Self::new(
            payload[0..16].try_into().expect("fixed field"),
            payload[16..32].try_into().expect("fixed field"),
            read_record(&payload[32..56])?,
            payload[56..88].try_into().expect("fixed field"),
            u64::from_le_bytes(payload[88..96].try_into().expect("fixed field")),
            u64::from_le_bytes(payload[96..104].try_into().expect("fixed field")),
            read_record(&payload[104..128])?,
            payload[128..160].try_into().expect("fixed field"),
        )
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn session(self) -> [u8; 16] {
        self.session
    }
    pub const fn declaration_record(self) -> PersistedRecordIdentity {
        self.declaration_record
    }
    pub const fn declaration_digest(self) -> [u8; 32] {
        self.declaration_digest
    }
    pub const fn next_chunk_ordinal(self) -> u64 {
        self.next_chunk_ordinal
    }
    pub const fn durable_bytes(self) -> u64 {
        self.durable_bytes
    }
    pub const fn last_chunk_record(self) -> PersistedRecordIdentity {
        self.last_chunk_record
    }
    pub const fn last_chunk_digest(self) -> [u8; 32] {
        self.last_chunk_digest
    }
}

fn write_record(bytes: &mut Vec<u8>, record: PersistedRecordIdentity) {
    bytes.extend_from_slice(&record.allocation_epoch());
    bytes.extend_from_slice(&record.ordinal().to_le_bytes());
}

fn read_record(bytes: &[u8]) -> Result<PersistedRecordIdentity, BlobRecordDenial> {
    PersistedRecordIdentity::new(
        bytes[0..16].try_into().expect("fixed record"),
        u64::from_le_bytes(bytes[16..24].try_into().expect("fixed record")),
    )
    .ok_or(BlobRecordDenial::InvalidFrontier)
}
