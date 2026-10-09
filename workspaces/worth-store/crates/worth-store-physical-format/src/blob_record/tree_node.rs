use super::envelope::{digest, encode, nonzero_16, nonzero_32};
use super::{BlobRecordDenial, BlobRecordKind};
use crate::PersistedRecordIdentity;

const OCCURRENCE_BYTES: usize = 80;
const CONTENT_PREFIX_BYTES: usize = 8;
const ENTRY_BYTES: usize = 64;
const MAX_ENTRIES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlobTreeNodeKind {
    Leaf = 1,
    Interior = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobTreeOccurrenceV1 {
    store: [u8; 16],
    session: [u8; 16],
    kind: BlobTreeNodeKind,
    level: u8,
    index: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobTreeEntryV1 {
    digest: [u8; 32],
    record: PersistedRecordIdentity,
    covered_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobTreeNodeV1 {
    occurrence: BlobTreeOccurrenceV1,
    covered_bytes: u64,
    canonical_digest: [u8; 32],
    entries: Vec<BlobTreeEntryV1>,
}

impl BlobTreeOccurrenceV1 {
    pub fn new(
        store: [u8; 16],
        session: [u8; 16],
        kind: BlobTreeNodeKind,
        level: u8,
        index: u64,
    ) -> Result<Self, BlobRecordDenial> {
        if (kind == BlobTreeNodeKind::Leaf && level != 0)
            || (kind == BlobTreeNodeKind::Interior && level == 0)
        {
            return Err(BlobRecordDenial::InvalidTreeShape);
        }
        Ok(Self {
            store: nonzero_16(store)?,
            session: nonzero_16(session)?,
            kind,
            level,
            index,
        })
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn session(self) -> [u8; 16] {
        self.session
    }
    pub const fn kind(self) -> BlobTreeNodeKind {
        self.kind
    }
    pub const fn level(self) -> u8 {
        self.level
    }
    pub const fn index(self) -> u64 {
        self.index
    }
}

impl BlobTreeEntryV1 {
    pub fn new(
        digest: [u8; 32],
        record: PersistedRecordIdentity,
        covered_bytes: u64,
    ) -> Result<Self, BlobRecordDenial> {
        if covered_bytes == 0 {
            return Err(BlobRecordDenial::InvalidTreeShape);
        }
        Ok(Self {
            digest: nonzero_32(digest)?,
            record,
            covered_bytes,
        })
    }

    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }
    pub const fn covered_bytes(self) -> u64 {
        self.covered_bytes
    }
}

impl BlobTreeNodeV1 {
    /// Exact maximum encoded size under this version's fanout and entry format.
    pub const fn maximum_encoded_bytes() -> usize {
        super::BLOB_RECORD_HEADER_BYTES
            + OCCURRENCE_BYTES
            + CONTENT_PREFIX_BYTES
            + MAX_ENTRIES * ENTRY_BYTES
    }

    pub fn new(
        occurrence: BlobTreeOccurrenceV1,
        entries: Vec<BlobTreeEntryV1>,
    ) -> Result<Self, BlobRecordDenial> {
        let (covered_bytes, canonical_digest, _) = validated_content(occurrence, &entries)?;
        Ok(Self {
            occurrence,
            covered_bytes,
            canonical_digest,
            entries,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let (_, _, content) = validated_content(self.occurrence, &self.entries)
            .expect("constructed tree node remains valid");
        let mut payload = Vec::with_capacity(OCCURRENCE_BYTES + content.len());
        payload.extend_from_slice(&self.occurrence.store);
        payload.extend_from_slice(&self.occurrence.session);
        payload.extend_from_slice(&self.occurrence.index.to_le_bytes());
        payload.extend_from_slice(&self.covered_bytes.to_le_bytes());
        payload.extend_from_slice(&self.canonical_digest);
        payload.extend_from_slice(&content);
        encode(BlobRecordKind::TreeNode, &payload).expect("bounded tree node fits frame ceiling")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::TreeNode {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload, frame.flags)
    }

    pub(super) fn decode_payload(payload: &[u8], flags: u16) -> Result<Self, BlobRecordDenial> {
        if flags != 1 || payload.len() < OCCURRENCE_BYTES + CONTENT_PREFIX_BYTES {
            return Err(BlobRecordDenial::MissingOccurrenceClaim);
        }
        let content = &payload[OCCURRENCE_BYTES..];
        let kind = match content[0] {
            1 => BlobTreeNodeKind::Leaf,
            2 => BlobTreeNodeKind::Interior,
            _ => return Err(BlobRecordDenial::InvalidTreeShape),
        };
        let occurrence = BlobTreeOccurrenceV1::new(
            payload[0..16].try_into().expect("fixed field"),
            payload[16..32].try_into().expect("fixed field"),
            kind,
            content[1],
            u64::from_le_bytes(payload[32..40].try_into().expect("fixed field")),
        )?;
        if content[2..4] != [0; 2] {
            return Err(BlobRecordDenial::InvalidTreeShape);
        }
        let count = u32::from_le_bytes(content[4..8].try_into().expect("fixed field")) as usize;
        if count == 0
            || count > MAX_ENTRIES
            || content.len() != CONTENT_PREFIX_BYTES + count * ENTRY_BYTES
        {
            return Err(BlobRecordDenial::InvalidTreeShape);
        }
        let mut entries = Vec::with_capacity(count);
        for entry in content[CONTENT_PREFIX_BYTES..].chunks_exact(ENTRY_BYTES) {
            let record = PersistedRecordIdentity::new(
                entry[32..48].try_into().expect("fixed field"),
                u64::from_le_bytes(entry[48..56].try_into().expect("fixed field")),
            )
            .ok_or(BlobRecordDenial::InvalidIdentity)?;
            entries.push(BlobTreeEntryV1::new(
                entry[0..32].try_into().expect("fixed field"),
                record,
                u64::from_le_bytes(entry[56..64].try_into().expect("fixed field")),
            )?);
        }
        let node = Self::new(occurrence, entries)?;
        if u64::from_le_bytes(payload[40..48].try_into().expect("fixed field"))
            != node.covered_bytes
            || payload[48..80] != node.canonical_digest
            || digest(&[content]) != node.canonical_digest
        {
            return Err(BlobRecordDenial::IntegrityMismatch);
        }
        Ok(node)
    }

    pub const fn occurrence(&self) -> BlobTreeOccurrenceV1 {
        self.occurrence
    }
    pub const fn covered_bytes(&self) -> u64 {
        self.covered_bytes
    }
    pub const fn canonical_digest(&self) -> [u8; 32] {
        self.canonical_digest
    }
    /// Physical-layout root identity (D5): SHA-256 of the complete encoded
    /// node frame, including its separately authenticated occurrence claim.
    /// Interior edges instead use `canonical_digest`, excluding occurrence.
    pub fn frame_digest(&self) -> [u8; 32] {
        digest(&[&self.encode()])
    }
    pub fn entries(&self) -> &[BlobTreeEntryV1] {
        &self.entries
    }
}

fn validated_content(
    occurrence: BlobTreeOccurrenceV1,
    entries: &[BlobTreeEntryV1],
) -> Result<(u64, [u8; 32], Vec<u8>), BlobRecordDenial> {
    if entries.is_empty() || entries.len() > MAX_ENTRIES {
        return Err(BlobRecordDenial::InvalidTreeShape);
    }
    let mut content = Vec::with_capacity(CONTENT_PREFIX_BYTES + entries.len() * ENTRY_BYTES);
    content.push(occurrence.kind as u8);
    content.push(occurrence.level);
    content.extend_from_slice(&[0; 2]);
    content.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    let mut covered_bytes = 0_u64;
    for entry in entries {
        covered_bytes = covered_bytes
            .checked_add(entry.covered_bytes)
            .ok_or(BlobRecordDenial::InvalidTreeShape)?;
        content.extend_from_slice(&entry.digest);
        content.extend_from_slice(&entry.record.allocation_epoch());
        content.extend_from_slice(&entry.record.ordinal().to_le_bytes());
        content.extend_from_slice(&entry.covered_bytes.to_le_bytes());
    }
    Ok((covered_bytes, digest(&[&content]), content))
}
