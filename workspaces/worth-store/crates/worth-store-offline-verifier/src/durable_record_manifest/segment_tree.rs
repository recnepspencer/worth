use std::collections::BTreeSet;
use std::path::Path;

use worth_store_physical_format::PhysicalRecordFormatDeclaration;

use super::independent_frame::{artifact_checksum, decode_frame};
use super::observation::OfflineSegmentPageMembership;
use super::root_tree::{read_u16, read_u32, read_u64, OfflineRootHeader};
use super::{read_artifact, OfflineDurableManifestDenial};

const BLOCK_PREFIX_BYTES: usize = 40;
const REFERENCE_BYTES: usize = 56;
const ENTRY_BYTES: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SegmentPageKey {
    segment: u64,
    page: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SegmentBlockReference {
    pub(super) generation: u64,
    pub(super) block: u64,
    pub(super) level: u16,
    checksum: u32,
    first: SegmentPageKey,
    last: SegmentPageKey,
}

pub(super) fn optional_reference(
    flag: u8,
    bytes: &[u8],
) -> Result<Option<SegmentBlockReference>, OfflineDurableManifestDenial> {
    match flag {
        0 if bytes == [0; REFERENCE_BYTES] => Ok(None),
        1 => decode_reference(bytes).map(Some),
        _ => Err(OfflineDurableManifestDenial::MalformedRoot),
    }
}

pub(super) fn walk_segment_tree(
    store_root: &Path,
    header: &OfflineRootHeader,
    format: PhysicalRecordFormatDeclaration,
) -> Result<(Vec<OfflineSegmentPageMembership>, u64, u64), OfflineDurableManifestDenial> {
    let Some(root) = header.segment_root else {
        return Ok((Vec::new(), 0, 0));
    };
    let mut pending = vec![root];
    let mut visited = BTreeSet::new();
    let mut entries = Vec::new();
    let mut blocks = 0_u64;
    let mut bytes_read = 0_u64;
    while let Some(reference) = pending.pop() {
        if reference.block >= header.next_segment_block
            || !visited.insert((reference.generation, reference.block))
        {
            return Err(OfflineDurableManifestDenial::InvalidTreeShape);
        }
        let path = store_root.join(format!(
            "families/records/segment-manifests/segments-{:016x}-block-{:016x}.manifest",
            reference.generation, reference.block
        ));
        let bytes = read_artifact(&path)?;
        blocks = blocks.saturating_add(1);
        bytes_read = bytes_read.saturating_add(bytes.len() as u64);
        match decode_block(&bytes, reference, header, format)? {
            DecodedSegmentBlock::Leaf(mut found) => entries.append(&mut found),
            DecodedSegmentBlock::Branch(children) => {
                pending.extend(children.into_iter().rev());
            }
        }
    }
    if !entries.windows(2).all(|pair| key(pair[0]) < key(pair[1])) {
        return Err(OfflineDurableManifestDenial::InvalidTreeShape);
    }
    Ok((entries, blocks, bytes_read))
}

enum DecodedSegmentBlock {
    Leaf(Vec<OfflineSegmentPageMembership>),
    Branch(Vec<SegmentBlockReference>),
}

fn decode_block(
    bytes: &[u8],
    expected: SegmentBlockReference,
    header: &OfflineRootHeader,
    format: PhysicalRecordFormatDeclaration,
) -> Result<DecodedSegmentBlock, OfflineDurableManifestDenial> {
    if artifact_checksum(bytes) != expected.checksum {
        return Err(OfflineDurableManifestDenial::ReferenceMismatch);
    }
    let frame = decode_frame(bytes, 9, format)?;
    let payload = frame.payload;
    if payload.len() < BLOCK_PREFIX_BYTES || payload[21..24] != [0; 3] || payload[32..40] != [0; 8]
    {
        return Err(OfflineDurableManifestDenial::MalformedBlock);
    }
    let level = read_u16(payload, 16);
    let count = read_u16(payload, 18);
    let generation = read_u64(payload, 24);
    if read_u64(payload, 0) != header.tree_identity
        || read_u64(payload, 8) != frame.identity
        || frame.identity != expected.block
        || generation != expected.generation
        || generation > header.generation
        || level != expected.level
        || count == 0
        || count > header.node_capacity
    {
        return Err(OfflineDurableManifestDenial::ReferenceMismatch);
    }
    let (width, decoded) = match payload[20] {
        1 if level == 0 => (
            ENTRY_BYTES,
            DecodedSegmentBlock::Leaf(
                payload[BLOCK_PREFIX_BYTES..]
                    .as_chunks::<ENTRY_BYTES>()
                    .0
                    .iter()
                    .map(|bytes| decode_entry(bytes))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        2 if level != 0 => {
            let children = payload[BLOCK_PREFIX_BYTES..]
                .as_chunks::<REFERENCE_BYTES>()
                .0
                .iter()
                .map(|bytes| decode_reference(bytes))
                .collect::<Result<Vec<_>, _>>()?;
            if !children.iter().all(|child| {
                child.level.checked_add(1) == Some(level) && child.generation <= generation
            }) {
                return Err(OfflineDurableManifestDenial::InvalidTreeShape);
            }
            (REFERENCE_BYTES, DecodedSegmentBlock::Branch(children))
        }
        _ => return Err(OfflineDurableManifestDenial::MalformedBlock),
    };
    if payload.len() != BLOCK_PREFIX_BYTES + usize::from(count) * width
        || decoded.first_last() != Some((expected.first, expected.last))
    {
        return Err(OfflineDurableManifestDenial::ReferenceMismatch);
    }
    Ok(decoded)
}

impl DecodedSegmentBlock {
    fn first_last(&self) -> Option<(SegmentPageKey, SegmentPageKey)> {
        match self {
            Self::Leaf(entries) => Some((key(*entries.first()?), key(*entries.last()?))),
            Self::Branch(children) => Some((children.first()?.first, children.last()?.last)),
        }
    }
}

fn decode_reference(bytes: &[u8]) -> Result<SegmentBlockReference, OfflineDurableManifestDenial> {
    let reference = SegmentBlockReference {
        generation: read_u64(bytes, 0),
        block: read_u64(bytes, 8),
        level: read_u16(bytes, 16),
        checksum: read_u32(bytes, 20),
        first: decode_key(&bytes[24..40])?,
        last: decode_key(&bytes[40..56])?,
    };
    if bytes[18..20] != [0; 2]
        || reference.generation == 0
        || reference.block == 0
        || reference.checksum == 0
        || reference.first > reference.last
    {
        return Err(OfflineDurableManifestDenial::MalformedReference);
    }
    Ok(reference)
}

fn decode_key(bytes: &[u8]) -> Result<SegmentPageKey, OfflineDurableManifestDenial> {
    let key = SegmentPageKey {
        segment: read_u64(bytes, 0),
        page: read_u64(bytes, 8),
    };
    if key.segment == 0 || key.page == 0 {
        return Err(OfflineDurableManifestDenial::MalformedReference);
    }
    Ok(key)
}

fn decode_entry(
    bytes: &[u8],
) -> Result<OfflineSegmentPageMembership, OfflineDurableManifestDenial> {
    let entry = OfflineSegmentPageMembership {
        segment: read_u64(bytes, 0),
        page: read_u64(bytes, 8),
        page_generation: read_u64(bytes, 16),
        data_generation: read_u64(bytes, 24),
        data_page_count: read_u32(bytes, 32),
        frame_index: read_u32(bytes, 36),
    };
    if entry.segment == 0
        || entry.page == 0
        || entry.page_generation == 0
        || entry.data_generation == 0
        || entry.data_page_count == 0
        || entry.frame_index >= entry.data_page_count
    {
        return Err(OfflineDurableManifestDenial::MalformedMembership);
    }
    Ok(entry)
}

fn key(entry: OfflineSegmentPageMembership) -> SegmentPageKey {
    SegmentPageKey {
        segment: entry.segment,
        page: entry.page,
    }
}
