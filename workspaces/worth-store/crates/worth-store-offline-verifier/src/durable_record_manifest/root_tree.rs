use std::collections::BTreeSet;
use std::path::Path;

use worth_store_physical_format::PhysicalRecordFormatDeclaration;

use super::independent_frame::{artifact_checksum, decode_frame};
use super::observation::{OfflineRecordIdentity, OfflineRecordPlacement};
use super::{read_artifact, OfflineDurableManifestDenial};

mod placement;
use placement::decode_placement;

const ROOT_PAYLOAD_BYTES: usize = 336;
const BLOCK_PREFIX_BYTES: usize = 40;
const REFERENCE_BYTES: usize = 72;
const PLACEMENT_BYTES: usize = 88;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RecordBlockReference {
    pub(super) generation: u64,
    pub(super) block: u64,
    pub(super) level: u16,
    pub(super) checksum: u32,
    pub(super) first: OfflineRecordIdentity,
    pub(super) last: OfflineRecordIdentity,
}

pub(super) struct OfflineRootHeader {
    pub(super) generation: u64,
    pub(super) tree_identity: u64,
    pub(super) node_capacity: u16,
    pub(super) record_count: u64,
    pub(super) next_block: u64,
    pub(super) next_segment_block: u64,
    pub(super) routing_root: Option<RecordBlockReference>,
    pub(super) segment_root: Option<super::segment_tree::SegmentBlockReference>,
    pub(super) free_space_root: Option<super::free_space_tree::FreeSpaceBlockReference>,
    pub(super) free_space_checksum: u32,
    pub(super) last_inline_record: Option<OfflineRecordIdentity>,
    pub(super) last_inline_segment: Option<(u64, u64)>,
}

pub(super) fn decode_root_header(
    bytes: &[u8],
    expected_generation: u64,
    format: PhysicalRecordFormatDeclaration,
) -> Result<OfflineRootHeader, OfflineDurableManifestDenial> {
    let frame = decode_frame(bytes, 2, format)?;
    let payload = frame.payload;
    if payload.len() != ROOT_PAYLOAD_BYTES
        || payload[18..24] != [0; 6]
        || payload[41..48] != [0; 7]
        || payload[121..128] != [0; 7]
        || payload[156..160] != [0; 4]
        || payload[161..168] != [0; 7]
        || payload[233..240] != [0; 7]
        || payload[313..320] != [0; 7]
    {
        return Err(OfflineDurableManifestDenial::MalformedRoot);
    }
    let generation = read_u64(payload, 0);
    let tree_identity = read_u64(payload, 8);
    let node_capacity = read_u16(payload, 16);
    let record_count = read_u64(payload, 24);
    let next_block = read_u64(payload, 32);
    let next_segment_block = read_u64(payload, 224);
    let maximum_capacity = ((format.page_size().bytes() as usize - 48 - 24) / 88) as u16;
    if generation != expected_generation
        || generation != frame.identity
        || tree_identity == 0
        || node_capacity < 2
        || node_capacity > maximum_capacity
        || next_block == 0
        || next_segment_block == 0
    {
        return Err(OfflineDurableManifestDenial::MalformedRoot);
    }
    let routing_root = optional_reference(payload[40], &payload[48..120])?;
    let segment_root = super::segment_tree::optional_reference(payload[160], &payload[168..224])?;
    let last_inline_record = match payload[120] {
        0 if payload[128..152] == [0; 24] => None,
        1 => Some(
            OfflineRecordIdentity::decode(&payload[128..152])
                .ok_or(OfflineDurableManifestDenial::MalformedRoot)?,
        ),
        _ => return Err(OfflineDurableManifestDenial::MalformedRoot),
    };
    let last_inline_segment = match payload[312] {
        0 if payload[320..336] == [0; 16] => None,
        1 if read_u64(payload, 320) != 0 && read_u64(payload, 328) != 0 => {
            Some((read_u64(payload, 320), read_u64(payload, 328)))
        }
        _ => return Err(OfflineDurableManifestDenial::MalformedRoot),
    };
    if (record_count == 0) != routing_root.is_none()
        || last_inline_record.is_some() != last_inline_segment.is_some()
        || routing_root.is_some_and(|reference| {
            reference.generation > generation
                || reference.block >= next_block
                || required_tree_level(record_count, node_capacity) != Some(reference.level)
        })
        || segment_root.is_some_and(|reference| {
            reference.generation > generation
                || reference.block >= next_segment_block
                || required_tree_level(record_count, node_capacity)
                    .is_none_or(|maximum| reference.level > maximum)
        })
        || read_u32(payload, 152) == 0
    {
        return Err(OfflineDurableManifestDenial::MalformedRoot);
    }
    Ok(OfflineRootHeader {
        generation,
        tree_identity,
        node_capacity,
        record_count,
        next_block,
        next_segment_block,
        routing_root,
        segment_root,
        free_space_root: super::free_space_tree::optional_reference(
            payload[232],
            &payload[240..312],
        )?,
        free_space_checksum: read_u32(payload, 152),
        last_inline_record,
        last_inline_segment,
    })
}

pub(super) fn walk_root_tree(
    store_root: &Path,
    header: &OfflineRootHeader,
    format: PhysicalRecordFormatDeclaration,
) -> Result<(Vec<OfflineRecordPlacement>, u64, u64), OfflineDurableManifestDenial> {
    let Some(root) = header.routing_root else {
        return Ok((Vec::new(), 0, 0));
    };
    let mut pending = vec![root];
    let mut visited = BTreeSet::new();
    let mut placements = Vec::new();
    let mut blocks = 0_u64;
    let mut bytes_read = 0_u64;
    while let Some(reference) = pending.pop() {
        if reference.block >= header.next_block
            || !visited.insert((reference.generation, reference.block))
        {
            return Err(OfflineDurableManifestDenial::InvalidTreeShape);
        }
        let path = store_root.join(format!(
            "families/records/roots/root-{:016x}-block-{:016x}.manifest",
            reference.generation, reference.block
        ));
        let bytes = read_artifact(&path)?;
        bytes_read = bytes_read.saturating_add(bytes.len() as u64);
        blocks = blocks.saturating_add(1);
        let decoded = decode_block(&bytes, reference, header, format)?;
        match decoded {
            DecodedRecordBlock::Leaf(mut entries) => placements.append(&mut entries),
            DecodedRecordBlock::Branch(children) => {
                pending.extend(children.into_iter().rev());
            }
        }
    }
    if placements.len() as u64 != header.record_count
        || !placements
            .windows(2)
            .all(|pair| pair[0].record() < pair[1].record())
        || !placements_have_unique_coordinates(&placements)
        || !inline_tail_matches(header, &placements)
    {
        return Err(OfflineDurableManifestDenial::InvalidTreeShape);
    }
    Ok((placements, blocks, bytes_read))
}

fn inline_tail_matches(header: &OfflineRootHeader, placements: &[OfflineRecordPlacement]) -> bool {
    match (header.last_inline_record, header.last_inline_segment) {
        (None, None) => true,
        (Some(record), Some((expected_segment, expected_generation))) => placements
            .iter()
            .find(|placement| placement.record() == record)
            .is_some_and(|placement| {
                matches!(
                    placement,
                    OfflineRecordPlacement::Inline {
                        segment,
                        segment_generation,
                        ..
                    } if *segment == expected_segment
                        && *segment_generation == expected_generation
                )
            }),
        _ => false,
    }
}

enum DecodedRecordBlock {
    Leaf(Vec<OfflineRecordPlacement>),
    Branch(Vec<RecordBlockReference>),
}

fn decode_block(
    bytes: &[u8],
    expected: RecordBlockReference,
    header: &OfflineRootHeader,
    format: PhysicalRecordFormatDeclaration,
) -> Result<DecodedRecordBlock, OfflineDurableManifestDenial> {
    if artifact_checksum(bytes) != expected.checksum {
        return Err(OfflineDurableManifestDenial::ReferenceMismatch);
    }
    let frame = decode_frame(bytes, 8, format)?;
    let payload = frame.payload;
    if payload.len() < BLOCK_PREFIX_BYTES || payload[21..24] != [0; 3] || payload[32..40] != [0; 8]
    {
        return Err(OfflineDurableManifestDenial::MalformedBlock);
    }
    let block = read_u64(payload, 8);
    let level = read_u16(payload, 16);
    let count = read_u16(payload, 18);
    let generation = read_u64(payload, 24);
    if frame.schema == 3
        && (level != 0
            || !payload[BLOCK_PREFIX_BYTES..]
                .chunks_exact(PLACEMENT_BYTES)
                .any(|entry| entry[25..32] != [0; 7]))
    {
        return Err(OfflineDurableManifestDenial::MalformedBlock);
    }
    if read_u64(payload, 0) != header.tree_identity
        || block != frame.identity
        || block != expected.block
        || generation != expected.generation
        || generation > header.generation
        || level != expected.level
        || count == 0
        || count > header.node_capacity
    {
        return Err(OfflineDurableManifestDenial::ReferenceMismatch);
    }
    let (width, decoded) = match payload[20] {
        1 if level == 0 => {
            let entries = payload[BLOCK_PREFIX_BYTES..]
                .chunks_exact(PLACEMENT_BYTES)
                .map(|entry| decode_placement(entry, frame.schema))
                .collect::<Result<Vec<_>, _>>()?;
            (PLACEMENT_BYTES, DecodedRecordBlock::Leaf(entries))
        }
        2 if level != 0 => {
            let children = payload[BLOCK_PREFIX_BYTES..]
                .chunks_exact(REFERENCE_BYTES)
                .map(decode_reference)
                .collect::<Result<Vec<_>, _>>()?;
            if !children.iter().all(|child| {
                child.level.checked_add(1) == Some(level) && child.generation <= generation
            }) {
                return Err(OfflineDurableManifestDenial::InvalidTreeShape);
            }
            (REFERENCE_BYTES, DecodedRecordBlock::Branch(children))
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

impl DecodedRecordBlock {
    fn first_last(&self) -> Option<(OfflineRecordIdentity, OfflineRecordIdentity)> {
        match self {
            Self::Leaf(entries) => Some((entries.first()?.record(), entries.last()?.record())),
            Self::Branch(children) => Some((children.first()?.first, children.last()?.last)),
        }
    }
}

fn optional_reference(
    flag: u8,
    bytes: &[u8],
) -> Result<Option<RecordBlockReference>, OfflineDurableManifestDenial> {
    match flag {
        0 if bytes == [0; REFERENCE_BYTES] => Ok(None),
        1 => decode_reference(bytes).map(Some),
        _ => Err(OfflineDurableManifestDenial::MalformedRoot),
    }
}

fn decode_reference(bytes: &[u8]) -> Result<RecordBlockReference, OfflineDurableManifestDenial> {
    let first = OfflineRecordIdentity::decode(&bytes[24..48])
        .ok_or(OfflineDurableManifestDenial::MalformedReference)?;
    let last = OfflineRecordIdentity::decode(&bytes[48..72])
        .ok_or(OfflineDurableManifestDenial::MalformedReference)?;
    let reference = RecordBlockReference {
        generation: read_u64(bytes, 0),
        block: read_u64(bytes, 8),
        level: read_u16(bytes, 16),
        checksum: read_u32(bytes, 20),
        first,
        last,
    };
    if bytes[18..20] != [0; 2]
        || reference.generation == 0
        || reference.block == 0
        || reference.checksum == 0
        || first > last
    {
        return Err(OfflineDurableManifestDenial::MalformedReference);
    }
    Ok(reference)
}

fn placements_have_unique_coordinates(placements: &[OfflineRecordPlacement]) -> bool {
    let mut coordinates = BTreeSet::new();
    placements.iter().all(|placement| {
        coordinates.insert(match placement {
            OfflineRecordPlacement::Inline {
                segment,
                page,
                slot,
                ..
            } => (1, *segment, *page, u64::from(*slot)),
            OfflineRecordPlacement::Extent { extent, .. } => (2, 0, *extent, 0),
        })
    })
}

pub(super) fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

pub(super) fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

pub(super) fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn required_tree_level(entry_count: u64, capacity: u16) -> Option<u16> {
    if entry_count == 0 || capacity < 2 {
        return None;
    }
    let mut nodes = entry_count.div_ceil(u64::from(capacity));
    let mut level = 0_u16;
    while nodes > 1 {
        nodes = nodes.div_ceil(u64::from(capacity));
        level = level.checked_add(1)?;
    }
    Some(level)
}
