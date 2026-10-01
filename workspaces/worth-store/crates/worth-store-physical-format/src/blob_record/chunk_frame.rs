use super::envelope::{admitted_chunk_size, digest, encode, nonzero_16, nonzero_32};
use super::{BlobRecordDenial, BlobRecordKind};

const OCCURRENCE_BYTES: usize = 80;
const CONTENT_PREFIX_BYTES: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobChunkOccurrenceV1 {
    store: [u8; 16],
    session: [u8; 16],
    ordinal: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobChunkFrameV1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedBlobChunkFrameV1<'bytes> {
    occurrence: BlobChunkOccurrenceV1,
    chunk_size: u32,
    stored_digest: [u8; 32],
    bytes: &'bytes [u8],
}

impl BlobChunkOccurrenceV1 {
    pub fn new(store: [u8; 16], session: [u8; 16], ordinal: u64) -> Result<Self, BlobRecordDenial> {
        Ok(Self {
            store: nonzero_16(store)?,
            session: nonzero_16(session)?,
            ordinal,
        })
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn session(self) -> [u8; 16] {
        self.session
    }
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}

impl BlobChunkFrameV1 {
    pub fn encode(
        occurrence: BlobChunkOccurrenceV1,
        chunk_size: u32,
        bytes: &[u8],
    ) -> Result<Vec<u8>, BlobRecordDenial> {
        let chunk_size = admitted_chunk_size(chunk_size)?;
        if bytes.is_empty() || bytes.len() > chunk_size as usize {
            return Err(BlobRecordDenial::InvalidChunkLength);
        }
        let length = u32::try_from(bytes.len()).map_err(|_| BlobRecordDenial::FrameTooLarge)?;
        let mut content = Vec::with_capacity(CONTENT_PREFIX_BYTES + bytes.len());
        content.push(1); // canonical fixed-size rule version
        content.extend_from_slice(&[0; 3]);
        content.extend_from_slice(&chunk_size.to_le_bytes());
        content.extend_from_slice(&length.to_le_bytes());
        content.extend_from_slice(bytes);
        let stored_digest = digest(&[&content]);
        let mut payload = Vec::with_capacity(OCCURRENCE_BYTES + content.len());
        payload.extend_from_slice(&occurrence.store);
        payload.extend_from_slice(&occurrence.session);
        payload.extend_from_slice(&occurrence.ordinal.to_le_bytes());
        payload.extend_from_slice(&chunk_size.to_le_bytes());
        payload.extend_from_slice(&length.to_le_bytes());
        payload.extend_from_slice(&stored_digest);
        payload.extend_from_slice(&content);
        encode(BlobRecordKind::Chunk, &payload)
    }
}

impl<'bytes> DecodedBlobChunkFrameV1<'bytes> {
    pub fn decode(bytes: &'bytes [u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::Chunk {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload, frame.flags)
    }

    pub(super) fn decode_payload(
        payload: &'bytes [u8],
        flags: u16,
    ) -> Result<Self, BlobRecordDenial> {
        if flags != 1 || payload.len() < OCCURRENCE_BYTES + CONTENT_PREFIX_BYTES {
            return Err(BlobRecordDenial::MissingOccurrenceClaim);
        }
        let occurrence = BlobChunkOccurrenceV1::new(
            payload[0..16].try_into().expect("fixed field"),
            payload[16..32].try_into().expect("fixed field"),
            u64::from_le_bytes(payload[32..40].try_into().expect("fixed field")),
        )?;
        let chunk_size = admitted_chunk_size(u32::from_le_bytes(
            payload[40..44].try_into().expect("fixed field"),
        ))?;
        let length = u32::from_le_bytes(payload[44..48].try_into().expect("fixed field"));
        let stored_digest = nonzero_32(payload[48..80].try_into().expect("fixed field"))?;
        let content = &payload[OCCURRENCE_BYTES..];
        if content[0..4] != [1, 0, 0, 0]
            || u32::from_le_bytes(content[4..8].try_into().expect("fixed field")) != chunk_size
            || u32::from_le_bytes(content[8..12].try_into().expect("fixed field")) != length
            || length == 0
            || length > chunk_size
            || content.len() != CONTENT_PREFIX_BYTES + length as usize
        {
            return Err(BlobRecordDenial::InvalidChunkLength);
        }
        if digest(&[content]) != stored_digest {
            return Err(BlobRecordDenial::IntegrityMismatch);
        }
        Ok(Self {
            occurrence,
            chunk_size,
            stored_digest,
            bytes: &content[12..],
        })
    }

    pub const fn occurrence(&self) -> BlobChunkOccurrenceV1 {
        self.occurrence
    }
    pub const fn chunk_size(&self) -> u32 {
        self.chunk_size
    }
    pub const fn stored_digest(&self) -> [u8; 32] {
        self.stored_digest
    }
    pub const fn bytes(&self) -> &'bytes [u8] {
        self.bytes
    }
}
