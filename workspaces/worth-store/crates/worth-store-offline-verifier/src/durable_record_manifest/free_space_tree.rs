use std::collections::BTreeSet;
use std::path::Path;

use worth_store_physical_format::{
    maximum_segment_manifest_pages, PhysicalRecordFormatDeclaration,
};

use super::independent_frame::{artifact_checksum, decode_frame};
use super::observation::{OfflineAllocationClass, OfflineFreeSpaceMembership};
use super::root_tree::{read_u16, read_u32, read_u64, OfflineRootHeader};
use super::{read_artifact, OfflineDurableManifestDenial};

const HEADER_PAYLOAD_BYTES: usize = 168;
const BLOCK_PREFIX_BYTES: usize = 40;
const REFERENCE_BYTES: usize = 72;
const ENTRY_BYTES: usize = 40;

#[derive(Clone, Copy)]
pub(super) struct ArenaGeometry {
    pub(super) next_arena: u64,
    pub(super) capacity: u64,
    pub(super) alignment: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FreeSpaceKey {
    class: OfflineAllocationClass,
    owner: u64,
    offset: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FreeSpaceBlockReference {
    pub(super) generation: u64,
    pub(super) block: u64,
    pub(super) level: u16,
    checksum: u32,
    first: FreeSpaceKey,
    last: FreeSpaceKey,
}

pub(super) fn optional_reference(
    flag: u8,
    bytes: &[u8],
) -> Result<Option<FreeSpaceBlockReference>, OfflineDurableManifestDenial> {
    match flag {
        0 if bytes == [0; REFERENCE_BYTES] => Ok(None),
        1 => decode_reference(bytes).map(Some),
        _ => Err(OfflineDurableManifestDenial::MalformedRoot),
    }
}

pub(super) fn walk_free_space_tree(
    store_root: &Path,
    header: &OfflineRootHeader,
    format: PhysicalRecordFormatDeclaration,
) -> Result<(Vec<OfflineFreeSpaceMembership>, u64, u64, ArenaGeometry), OfflineDurableManifestDenial>
{
    let path = store_root.join(format!(
        "families/records/free-space/free-space-{:016x}.manifest",
        header.generation
    ));
    let header_bytes = read_artifact(&path)?;
    if artifact_checksum(&header_bytes) != header.free_space_checksum {
        return Err(OfflineDurableManifestDenial::ReferenceMismatch);
    }
    let free = decode_header(&header_bytes, header, format)?;
    let Some(root) = free.root else {
        return Ok((Vec::new(), 0, header_bytes.len() as u64, free.arena));
    };
    let mut pending = vec![root];
    let mut visited = BTreeSet::new();
    let mut entries = Vec::new();
    let mut blocks = 0_u64;
    let mut bytes_read = header_bytes.len() as u64;
    while let Some(reference) = pending.pop() {
        if reference.block >= free.next_block
            || !visited.insert((reference.generation, reference.block))
        {
            return Err(OfflineDurableManifestDenial::InvalidTreeShape);
        }
        let path = store_root.join(format!(
            "families/records/free-space/free-space-{:016x}-block-{:016x}.manifest",
            reference.generation, reference.block
        ));
        let bytes = read_artifact(&path)?;
        blocks = blocks.saturating_add(1);
        bytes_read = bytes_read.saturating_add(bytes.len() as u64);
        match decode_block(&bytes, reference, &free, format)? {
            DecodedFreeSpaceBlock::Leaf(mut found) => entries.append(&mut found),
            DecodedFreeSpaceBlock::Branch(children) => {
                pending.extend(children.into_iter().rev());
            }
        }
    }
    if entries.len() as u64 != free.entry_count
        || !entries.windows(2).all(|pair| {
            key(pair[0]) < key(pair[1])
                && !(pair[0].class == OfflineAllocationClass::ExtentArena
                    && pair[1].class == OfflineAllocationClass::ExtentArena
                    && pair[0].owner == pair[1].owner
                    && pair[0].first_unallocated + pair[0].unallocated_count
                        >= pair[1].first_unallocated)
        })
    {
        return Err(OfflineDurableManifestDenial::InvalidTreeShape);
    }
    Ok((entries, blocks, bytes_read, free.arena))
}

struct OfflineFreeSpaceHeader {
    arena: ArenaGeometry,
    generation: u64,
    tree_identity: u64,
    node_capacity: u16,
    segment_page_capacity: u32,
    entry_count: u64,
    next_block: u64,
    root: Option<FreeSpaceBlockReference>,
}

fn decode_header(
    bytes: &[u8],
    root: &OfflineRootHeader,
    format: PhysicalRecordFormatDeclaration,
) -> Result<OfflineFreeSpaceHeader, OfflineDurableManifestDenial> {
    let frame = decode_frame(bytes, 7, format)?;
    let payload = frame.payload;
    if payload.len() != HEADER_PAYLOAD_BYTES
        || payload[22..24] != [0; 2]
        || payload[65..72] != [0; 7]
    {
        return Err(OfflineDurableManifestDenial::MalformedFreeSpace);
    }
    let header = OfflineFreeSpaceHeader {
        arena: ArenaGeometry {
            next_arena: read_u64(payload, 144),
            capacity: read_u64(payload, 152),
            alignment: read_u64(payload, 160),
        },
        generation: read_u64(payload, 0),
        tree_identity: read_u64(payload, 8),
        node_capacity: read_u16(payload, 16),
        segment_page_capacity: read_u32(payload, 18),
        entry_count: read_u64(payload, 24),
        next_block: read_u64(payload, 56),
        root: optional_reference(payload[64], &payload[72..144])?,
    };
    if header.generation != root.generation
        || header.generation != frame.identity
        || header.tree_identity != root.tree_identity
        || header.node_capacity != root.node_capacity
        || header.segment_page_capacity == 0
        || header.segment_page_capacity > maximum_segment_manifest_pages(format)
        || header.next_block == 0
        || (header.entry_count == 0) != header.root.is_none()
        || header.root != root.free_space_root
        || header.root.is_some_and(|reference| {
            reference.generation > header.generation
                || reference.block >= header.next_block
                || required_tree_level(header.entry_count, header.node_capacity)
                    .is_none_or(|maximum| reference.level > maximum)
        })
        || read_u64(payload, 32) == 0
        || read_u64(payload, 40) == 0
        || read_u64(payload, 48) == 0
        || read_u64(payload, 144) == 0
        || read_u64(payload, 152) == 0
        || !read_u64(payload, 160).is_power_of_two()
        || !read_u64(payload, 152).is_multiple_of(read_u64(payload, 160))
    {
        return Err(OfflineDurableManifestDenial::MalformedFreeSpace);
    }
    Ok(header)
}

enum DecodedFreeSpaceBlock {
    Leaf(Vec<OfflineFreeSpaceMembership>),
    Branch(Vec<FreeSpaceBlockReference>),
}

fn decode_block(
    bytes: &[u8],
    expected: FreeSpaceBlockReference,
    header: &OfflineFreeSpaceHeader,
    format: PhysicalRecordFormatDeclaration,
) -> Result<DecodedFreeSpaceBlock, OfflineDurableManifestDenial> {
    if artifact_checksum(bytes) != expected.checksum {
        return Err(OfflineDurableManifestDenial::ReferenceMismatch);
    }
    let frame = decode_frame(bytes, 10, format)?;
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
            DecodedFreeSpaceBlock::Leaf(
                payload[BLOCK_PREFIX_BYTES..]
                    .chunks_exact(ENTRY_BYTES)
                    .map(decode_entry)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
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
            (REFERENCE_BYTES, DecodedFreeSpaceBlock::Branch(children))
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

impl DecodedFreeSpaceBlock {
    fn first_last(&self) -> Option<(FreeSpaceKey, FreeSpaceKey)> {
        match self {
            Self::Leaf(entries) => Some((key(*entries.first()?), key(*entries.last()?))),
            Self::Branch(children) => Some((children.first()?.first, children.last()?.last)),
        }
    }
}

fn decode_reference(bytes: &[u8]) -> Result<FreeSpaceBlockReference, OfflineDurableManifestDenial> {
    let reference = FreeSpaceBlockReference {
        generation: read_u64(bytes, 0),
        block: read_u64(bytes, 8),
        level: read_u16(bytes, 16),
        checksum: read_u32(bytes, 20),
        first: decode_key(&bytes[24..48])?,
        last: decode_key(&bytes[48..72])?,
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

fn decode_key(bytes: &[u8]) -> Result<FreeSpaceKey, OfflineDurableManifestDenial> {
    if bytes[1..8] != [0; 7] {
        return Err(OfflineDurableManifestDenial::MalformedReference);
    }
    let class = decode_class(bytes[0])?;
    let owner = read_u64(bytes, 8);
    if owner == 0 {
        return Err(OfflineDurableManifestDenial::MalformedReference);
    }
    let offset = read_u64(bytes, 16);
    if class == OfflineAllocationClass::InlinePage && offset != 0 {
        return Err(OfflineDurableManifestDenial::MalformedReference);
    }
    Ok(FreeSpaceKey {
        class,
        owner,
        offset,
    })
}

fn decode_entry(bytes: &[u8]) -> Result<OfflineFreeSpaceMembership, OfflineDurableManifestDenial> {
    if bytes[1..8] != [0; 7] {
        return Err(OfflineDurableManifestDenial::MalformedMembership);
    }
    let entry = OfflineFreeSpaceMembership {
        class: decode_class(bytes[0])?,
        owner: read_u64(bytes, 8),
        first_unallocated: read_u64(bytes, 16),
        unallocated_count: read_u64(bytes, 24),
        generation: read_u64(bytes, 32),
    };
    if entry.owner == 0
        || (entry.class == OfflineAllocationClass::InlinePage && entry.first_unallocated == 0)
        || entry.unallocated_count == 0
        || entry.generation == 0
        || entry
            .first_unallocated
            .checked_add(entry.unallocated_count)
            .is_none()
    {
        return Err(OfflineDurableManifestDenial::MalformedMembership);
    }
    Ok(entry)
}

fn decode_class(value: u8) -> Result<OfflineAllocationClass, OfflineDurableManifestDenial> {
    match value {
        1 => Ok(OfflineAllocationClass::InlinePage),
        2 => Ok(OfflineAllocationClass::ExtentArena),
        _ => Err(OfflineDurableManifestDenial::MalformedMembership),
    }
}

fn key(entry: OfflineFreeSpaceMembership) -> FreeSpaceKey {
    FreeSpaceKey {
        class: entry.class,
        owner: entry.owner,
        offset: if entry.class == OfflineAllocationClass::ExtentArena {
            entry.first_unallocated
        } else {
            0
        },
    }
}

fn required_tree_level(entry_universe: u64, capacity: u16) -> Option<u16> {
    if entry_universe == 0 || capacity < 2 {
        return None;
    }
    let mut nodes = entry_universe.div_ceil(u64::from(capacity));
    let mut level = 0_u16;
    while nodes > 1 {
        nodes = nodes.div_ceil(u64::from(capacity));
        level = level.checked_add(1)?;
    }
    Some(level)
}
