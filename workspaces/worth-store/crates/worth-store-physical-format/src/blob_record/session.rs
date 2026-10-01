use super::envelope::{admitted_chunk_size, encode, nonzero_16, nonzero_32};
use super::{BlobRecordDenial, BlobRecordKind};

const PAYLOAD_BYTES: usize = 108;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobSessionDeclarationV1 {
    store: [u8; 16],
    session: [u8; 16],
    object: [u8; 16],
    key_scope: [u8; 32],
    chunk_size: u32,
    declared_bytes: u64,
    memory_limit: u64,
    max_checkpoint_sequence: u64,
}

impl BlobSessionDeclarationV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: [u8; 16],
        session: [u8; 16],
        object: [u8; 16],
        key_scope: [u8; 32],
        chunk_size: u32,
        declared_bytes: u64,
        memory_limit: u64,
        max_checkpoint_sequence: u64,
    ) -> Result<Self, BlobRecordDenial> {
        if declared_bytes == 0 || memory_limit == 0 || max_checkpoint_sequence == 0 {
            return Err(BlobRecordDenial::InvalidDeclaration);
        }
        Ok(Self {
            store: nonzero_16(store)?,
            session: nonzero_16(session)?,
            object: nonzero_16(object)?,
            key_scope: nonzero_32(key_scope)?,
            chunk_size: admitted_chunk_size(chunk_size)?,
            declared_bytes,
            memory_limit,
            max_checkpoint_sequence,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(PAYLOAD_BYTES);
        payload.extend_from_slice(&self.store);
        payload.extend_from_slice(&self.session);
        payload.extend_from_slice(&self.object);
        payload.extend_from_slice(&self.key_scope);
        payload.extend_from_slice(&self.chunk_size.to_le_bytes());
        payload.extend_from_slice(&self.declared_bytes.to_le_bytes());
        payload.extend_from_slice(&self.memory_limit.to_le_bytes());
        payload.extend_from_slice(&self.max_checkpoint_sequence.to_le_bytes());
        encode(BlobRecordKind::SessionDeclared, &payload)
            .expect("fixed declaration fits control-frame ceiling")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::SessionDeclared {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(super) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        if payload.len() != PAYLOAD_BYTES {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        Self::new(
            payload[0..16].try_into().expect("fixed field"),
            payload[16..32].try_into().expect("fixed field"),
            payload[32..48].try_into().expect("fixed field"),
            payload[48..80].try_into().expect("fixed field"),
            u32::from_le_bytes(payload[80..84].try_into().expect("fixed field")),
            u64::from_le_bytes(payload[84..92].try_into().expect("fixed field")),
            u64::from_le_bytes(payload[92..100].try_into().expect("fixed field")),
            u64::from_le_bytes(payload[100..108].try_into().expect("fixed field")),
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
    pub const fn key_scope(self) -> [u8; 32] {
        self.key_scope
    }
    pub const fn chunk_size(self) -> u32 {
        self.chunk_size
    }
    pub const fn declared_bytes(self) -> u64 {
        self.declared_bytes
    }
    pub const fn memory_limit(self) -> u64 {
        self.memory_limit
    }
    pub const fn max_checkpoint_sequence(self) -> u64 {
        self.max_checkpoint_sequence
    }
}
