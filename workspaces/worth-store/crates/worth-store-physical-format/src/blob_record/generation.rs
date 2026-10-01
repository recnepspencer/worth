use super::envelope::{admitted_chunk_size, nonzero_16, nonzero_32, visit_canonical_frame};
use super::{BlobRecordDenial, BlobRecordKind};
use crate::PersistedRecordIdentity;

const PAYLOAD_BYTES: usize = 188;
pub(super) const GENERATION_PUBLICATION_FRAME_BYTES: usize =
    super::BLOB_RECORD_HEADER_BYTES + PAYLOAD_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobGenerationPublicationV1 {
    store: [u8; 16],
    session: [u8; 16],
    object: [u8; 16],
    generation: u64,
    root_record: PersistedRecordIdentity,
    root_digest: [u8; 32],
    total_bytes: u64,
    logical_digest: [u8; 32],
    chunk_size: u32,
    key_scope: [u8; 32],
}

impl BlobGenerationPublicationV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: [u8; 16],
        session: [u8; 16],
        object: [u8; 16],
        generation: u64,
        root_record: PersistedRecordIdentity,
        root_digest: [u8; 32],
        total_bytes: u64,
        logical_digest: [u8; 32],
        chunk_size: u32,
        key_scope: [u8; 32],
    ) -> Result<Self, BlobRecordDenial> {
        if generation == 0 || total_bytes == 0 {
            return Err(BlobRecordDenial::InvalidGeneration);
        }
        Ok(Self {
            store: nonzero_16(store)?,
            session: nonzero_16(session)?,
            object: nonzero_16(object)?,
            generation,
            root_record,
            root_digest: nonzero_32(root_digest)?,
            total_bytes,
            logical_digest: nonzero_32(logical_digest)?,
            chunk_size: admitted_chunk_size(chunk_size)?,
            key_scope: nonzero_32(key_scope)?,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut frame = Vec::with_capacity(GENERATION_PUBLICATION_FRAME_BYTES);
        self.visit_canonical_frame(&mut |part| frame.extend_from_slice(part));
        frame
    }

    pub(super) fn visit_canonical_frame(self, emit: &mut dyn FnMut(&[u8])) {
        visit_canonical_frame(
            BlobRecordKind::GenerationPublished,
            PAYLOAD_BYTES,
            |part| self.visit_payload(part),
            emit,
        )
        .expect("fixed publication fits control-frame ceiling");
    }

    fn visit_payload(self, emit: &mut dyn FnMut(&[u8])) {
        emit(&self.store);
        emit(&self.session);
        emit(&self.object);
        emit(&self.generation.to_le_bytes());
        emit(&self.root_record.allocation_epoch());
        emit(&self.root_record.ordinal().to_le_bytes());
        emit(&self.root_digest);
        emit(&self.total_bytes.to_le_bytes());
        emit(&self.logical_digest);
        emit(&self.chunk_size.to_le_bytes());
        emit(&self.key_scope);
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::GenerationPublished {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(super) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        if payload.len() != PAYLOAD_BYTES {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        let root_record = PersistedRecordIdentity::new(
            payload[56..72].try_into().expect("fixed field"),
            u64::from_le_bytes(payload[72..80].try_into().expect("fixed field")),
        )
        .ok_or(BlobRecordDenial::InvalidIdentity)?;
        Self::new(
            payload[0..16].try_into().expect("fixed field"),
            payload[16..32].try_into().expect("fixed field"),
            payload[32..48].try_into().expect("fixed field"),
            u64::from_le_bytes(payload[48..56].try_into().expect("fixed field")),
            root_record,
            payload[80..112].try_into().expect("fixed field"),
            u64::from_le_bytes(payload[112..120].try_into().expect("fixed field")),
            payload[120..152].try_into().expect("fixed field"),
            u32::from_le_bytes(payload[152..156].try_into().expect("fixed field")),
            payload[156..188].try_into().expect("fixed field"),
        )
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn session(self) -> [u8; 16] {
        self.session
    }
    pub const fn object(self) -> [u8; 16] {
        self.object
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
    pub const fn root_record(self) -> PersistedRecordIdentity {
        self.root_record
    }
    pub const fn root_digest(self) -> [u8; 32] {
        self.root_digest
    }
    pub const fn total_bytes(self) -> u64 {
        self.total_bytes
    }
    pub const fn logical_digest(self) -> [u8; 32] {
        self.logical_digest
    }
    pub const fn chunk_size(self) -> u32 {
        self.chunk_size
    }
    pub const fn key_scope(self) -> [u8; 32] {
        self.key_scope
    }
}
