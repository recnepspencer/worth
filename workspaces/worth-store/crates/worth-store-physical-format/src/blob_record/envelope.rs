use sha2::{Digest, Sha256};

pub const BLOB_RECORD_VERSION: u8 = 1;
pub const BLOB_RECORD_HEADER_BYTES: usize = 48;
pub const BLOB_CHUNK_FRAME_MAX_BYTES: usize = 1 << 20;
pub const BLOB_TREE_NODE_FRAME_MAX_BYTES: usize = 512 << 10;
pub const BLOB_CONTROL_FRAME_MAX_BYTES: usize = 64 << 10;

const MAGIC: [u8; 8] = *b"WRC11BLB";
const CLAIM_PRESENT: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlobRecordKind {
    SessionDeclared = 1,
    Chunk = 2,
    TreeNode = 3,
    GenerationPublished = 4,
    SessionFrontier = 5,
    SessionAbandoned = 6,
    DropSetManifest = 7,
    ReclaimDescriptor = 8,
    DropSetManifestV2 = 9,
    OriginalDropReserved = 10,
    ChunkReuseClaim = 11,
    DedupeQuarantine = 12,
    DropSetManifestV3 = 13,
    ReclaimDescriptorV2 = 14,
    ChunkReuseClaimV2 = 15,
    ReclaimDescriptorV3 = 16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobRecordDenial {
    Truncated,
    WrongMagic,
    UnsupportedVersion,
    UnknownKind,
    InvalidFlags,
    LengthMismatch,
    FrameTooLarge,
    IntegrityMismatch,
    MissingOccurrenceClaim,
    InvalidIdentity,
    InvalidChunkRule,
    InvalidChunkLength,
    InvalidTreeShape,
    InvalidGeneration,
    InvalidDeclaration,
    InvalidFrontier,
    InvalidAbandonment,
    InvalidDropSet,
    InvalidReclaimDescriptor,
    InvalidReuseClaim,
    InvalidDedupeQuarantine,
    InvalidReclaimSource,
}

pub(super) struct DecodedEnvelope<'a> {
    pub(super) kind: BlobRecordKind,
    pub(super) flags: u16,
    pub(super) payload: &'a [u8],
}

impl BlobRecordKind {
    pub(crate) const fn from_route_code(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::SessionDeclared),
            2 => Some(Self::Chunk),
            3 => Some(Self::TreeNode),
            4 => Some(Self::GenerationPublished),
            5 => Some(Self::SessionFrontier),
            6 => Some(Self::SessionAbandoned),
            7 => Some(Self::DropSetManifest),
            8 => Some(Self::ReclaimDescriptor),
            9 => Some(Self::DropSetManifestV2),
            10 => Some(Self::OriginalDropReserved),
            11 => Some(Self::ChunkReuseClaim),
            12 => Some(Self::DedupeQuarantine),
            13 => Some(Self::DropSetManifestV3),
            14 => Some(Self::ReclaimDescriptorV2),
            15 => Some(Self::ChunkReuseClaimV2),
            16 => Some(Self::ReclaimDescriptorV3),
            _ => None,
        }
    }

    fn from_byte(byte: u8) -> Result<Self, BlobRecordDenial> {
        match byte {
            1 => Ok(Self::SessionDeclared),
            2 => Ok(Self::Chunk),
            3 => Ok(Self::TreeNode),
            4 => Ok(Self::GenerationPublished),
            5 => Ok(Self::SessionFrontier),
            6 => Ok(Self::SessionAbandoned),
            7 => Ok(Self::DropSetManifest),
            8 => Ok(Self::ReclaimDescriptor),
            9 => Ok(Self::DropSetManifestV2),
            10 => Ok(Self::OriginalDropReserved),
            11 => Ok(Self::ChunkReuseClaim),
            12 => Ok(Self::DedupeQuarantine),
            13 => Ok(Self::DropSetManifestV3),
            14 => Ok(Self::ReclaimDescriptorV2),
            15 => Ok(Self::ChunkReuseClaimV2),
            16 => Ok(Self::ReclaimDescriptorV3),
            _ => Err(BlobRecordDenial::UnknownKind),
        }
    }

    pub(super) const fn maximum_bytes(self) -> usize {
        match self {
            Self::Chunk => BLOB_CHUNK_FRAME_MAX_BYTES,
            Self::TreeNode => BLOB_TREE_NODE_FRAME_MAX_BYTES,
            Self::SessionDeclared
            | Self::GenerationPublished
            | Self::SessionFrontier
            | Self::SessionAbandoned => BLOB_CONTROL_FRAME_MAX_BYTES,
            Self::DropSetManifest
            | Self::ReclaimDescriptor
            | Self::DropSetManifestV2
            | Self::OriginalDropReserved
            | Self::ChunkReuseClaim
            | Self::DedupeQuarantine => BLOB_CONTROL_FRAME_MAX_BYTES,
            Self::DropSetManifestV3
            | Self::ReclaimDescriptorV2
            | Self::ReclaimDescriptorV3
            | Self::ChunkReuseClaimV2 => BLOB_CONTROL_FRAME_MAX_BYTES,
        }
    }

    pub(super) const fn requires_claim(self) -> bool {
        matches!(
            self,
            Self::Chunk
                | Self::TreeNode
                | Self::ChunkReuseClaim
                | Self::ChunkReuseClaimV2
                | Self::DedupeQuarantine
        )
    }
}

pub(super) fn encode(kind: BlobRecordKind, payload: &[u8]) -> Result<Vec<u8>, BlobRecordDenial> {
    let length = BLOB_RECORD_HEADER_BYTES
        .checked_add(payload.len())
        .ok_or(BlobRecordDenial::FrameTooLarge)?;
    if length > kind.maximum_bytes() {
        return Err(BlobRecordDenial::FrameTooLarge);
    }
    let payload_len = u32::try_from(payload.len()).map_err(|_| BlobRecordDenial::FrameTooLarge)?;
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(&MAGIC);
    bytes.push(kind as u8);
    bytes.push(BLOB_RECORD_VERSION);
    bytes.extend_from_slice(&(kind.requires_claim() as u16 * CLAIM_PRESENT).to_le_bytes());
    bytes.extend_from_slice(&payload_len.to_le_bytes());
    bytes.extend_from_slice(&[0; 32]);
    bytes.extend_from_slice(payload);
    let digest = envelope_digest(&bytes[..16], payload);
    bytes[16..48].copy_from_slice(&digest);
    Ok(bytes)
}

/// Canonical framing over a repeatable, allocation-free payload emitter.
/// The same emitter can feed a frame Vec or a digest without constructing an
/// intermediate payload or frame. Callers own the order of their typed fields.
pub(super) fn visit_canonical_frame(
    kind: BlobRecordKind,
    payload_len: usize,
    visit_payload: impl Fn(&mut dyn FnMut(&[u8])),
    emit: &mut dyn FnMut(&[u8]),
) -> Result<(), BlobRecordDenial> {
    let frame_len = BLOB_RECORD_HEADER_BYTES
        .checked_add(payload_len)
        .ok_or(BlobRecordDenial::FrameTooLarge)?;
    if frame_len > kind.maximum_bytes() {
        return Err(BlobRecordDenial::FrameTooLarge);
    }
    let encoded_payload_len =
        u32::try_from(payload_len).map_err(|_| BlobRecordDenial::FrameTooLarge)?;
    let mut prefix = [0_u8; 16];
    prefix[..8].copy_from_slice(&MAGIC);
    prefix[8] = kind as u8;
    prefix[9] = BLOB_RECORD_VERSION;
    prefix[10..12].copy_from_slice(&(kind.requires_claim() as u16 * CLAIM_PRESENT).to_le_bytes());
    prefix[12..16].copy_from_slice(&encoded_payload_len.to_le_bytes());
    let mut inner = Sha256::new();
    inner.update(prefix);
    let mut observed_len = 0_usize;
    visit_payload(&mut |part| {
        observed_len = observed_len.saturating_add(part.len());
        inner.update(part);
    });
    if observed_len != payload_len {
        return Err(BlobRecordDenial::LengthMismatch);
    }
    let digest: [u8; 32] = inner.finalize().into();
    emit(&prefix);
    emit(&digest);
    visit_payload(emit);
    Ok(())
}

pub(super) fn canonical_frame_sha256(
    kind: BlobRecordKind,
    payload_len: usize,
    visit_payload: impl Fn(&mut dyn FnMut(&[u8])),
) -> [u8; 32] {
    let mut hash = Sha256::new();
    visit_canonical_frame(kind, payload_len, visit_payload, &mut |part| {
        hash.update(part)
    })
    .expect("typed canonical control frame fits its declared ceiling");
    hash.finalize().into()
}

pub(super) fn decode(bytes: &[u8]) -> Result<DecodedEnvelope<'_>, BlobRecordDenial> {
    if bytes.len() < BLOB_RECORD_HEADER_BYTES {
        return Err(BlobRecordDenial::Truncated);
    }
    if bytes[..8] != MAGIC {
        return Err(BlobRecordDenial::WrongMagic);
    }
    let kind = BlobRecordKind::from_byte(bytes[8])?;
    if bytes[9] != BLOB_RECORD_VERSION {
        return Err(BlobRecordDenial::UnsupportedVersion);
    }
    if bytes.len() > kind.maximum_bytes() {
        return Err(BlobRecordDenial::FrameTooLarge);
    }
    let flags = u16::from_le_bytes(bytes[10..12].try_into().expect("fixed header"));
    if flags & !CLAIM_PRESENT != 0 || (!kind.requires_claim() && flags != 0) {
        return Err(BlobRecordDenial::InvalidFlags);
    }
    if kind.requires_claim() && flags & CLAIM_PRESENT == 0 {
        return Err(BlobRecordDenial::MissingOccurrenceClaim);
    }
    let payload_len = u32::from_le_bytes(bytes[12..16].try_into().expect("fixed header"));
    if BLOB_RECORD_HEADER_BYTES.checked_add(payload_len as usize) != Some(bytes.len()) {
        return Err(BlobRecordDenial::LengthMismatch);
    }
    let payload = &bytes[BLOB_RECORD_HEADER_BYTES..];
    if bytes[16..48] != envelope_digest(&bytes[..16], payload) {
        return Err(BlobRecordDenial::IntegrityMismatch);
    }
    Ok(DecodedEnvelope {
        kind,
        flags,
        payload,
    })
}

pub(super) fn digest(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

fn envelope_digest(header: &[u8], payload: &[u8]) -> [u8; 32] {
    digest(&[header, payload])
}

pub(super) fn nonzero_16(value: [u8; 16]) -> Result<[u8; 16], BlobRecordDenial> {
    (value != [0; 16])
        .then_some(value)
        .ok_or(BlobRecordDenial::InvalidIdentity)
}

pub(super) fn nonzero_32(value: [u8; 32]) -> Result<[u8; 32], BlobRecordDenial> {
    (value != [0; 32])
        .then_some(value)
        .ok_or(BlobRecordDenial::InvalidIdentity)
}

pub(super) fn admitted_chunk_size(size: u32) -> Result<u32, BlobRecordDenial> {
    ((64_u32 << 10)..=(256_u32 << 10))
        .contains(&size)
        .then_some(size)
        .ok_or(BlobRecordDenial::InvalidChunkRule)
}
