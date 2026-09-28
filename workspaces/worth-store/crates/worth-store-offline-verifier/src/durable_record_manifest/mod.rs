mod arena_ranges;
mod free_space_tree;
mod independent_frame;
mod observation;
mod payload_validation;
mod root_tree;
mod segment_tree;

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use worth_store_physical_format::PhysicalRecordFormatDeclaration;

pub use observation::{
    OfflineAllocationClass, OfflineFreeSpaceMembership, OfflineRecordIdentity,
    OfflineRecordPayloadObservation, OfflineRecordPlacement, OfflineSegmentPageMembership,
};

use independent_frame::decode_frame;
use root_tree::{decode_root_header, read_u32, read_u64, walk_root_tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineDurableManifestDenial {
    Io(std::io::ErrorKind),
    TruncatedFrame,
    FrameDeclarationMismatch,
    FrameLengthMismatch,
    FrameIntegrityMismatch,
    MalformedCatalog,
    MalformedRoot,
    MalformedBlock,
    MalformedReference,
    MalformedPlacement,
    MalformedMembership,
    MalformedFreeSpace,
    ReferenceMismatch,
    InvalidTreeShape,
    ReachabilityMismatch,
    MalformedPayloadFrame,
    CurrentRootRequestedAsResidue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineDurableManifestWalk {
    store_identity: [u8; 16],
    format_identity: [u8; 10],
    root_generation: u64,
    tree_identity: u64,
    node_capacity: u16,
    routing_level: Option<u16>,
    placements: Vec<OfflineRecordPlacement>,
    segment_pages: Vec<OfflineSegmentPageMembership>,
    free_space: Vec<OfflineFreeSpaceMembership>,
    manifest_blocks: u64,
    manifest_bytes: u64,
    payload_frames: u64,
    payload_bytes: u64,
    payload_digest: [u8; 32],
    record_payloads: Vec<OfflineRecordPayloadObservation>,
}

impl OfflineDurableManifestWalk {
    pub const fn store_identity(&self) -> [u8; 16] {
        self.store_identity
    }
    pub const fn format_identity(&self) -> [u8; 10] {
        self.format_identity
    }
    pub const fn root_generation(&self) -> u64 {
        self.root_generation
    }
    pub const fn tree_identity(&self) -> u64 {
        self.tree_identity
    }
    pub const fn node_capacity(&self) -> u16 {
        self.node_capacity
    }
    pub const fn routing_level(&self) -> Option<u16> {
        self.routing_level
    }
    pub fn placements(&self) -> &[OfflineRecordPlacement] {
        &self.placements
    }
    pub fn segment_pages(&self) -> &[OfflineSegmentPageMembership] {
        &self.segment_pages
    }
    pub fn free_space(&self) -> &[OfflineFreeSpaceMembership] {
        &self.free_space
    }
    pub const fn manifest_blocks(&self) -> u64 {
        self.manifest_blocks
    }
    pub const fn manifest_bytes(&self) -> u64 {
        self.manifest_bytes
    }
    pub const fn payload_frames(&self) -> u64 {
        self.payload_frames
    }
    pub const fn payload_bytes(&self) -> u64 {
        self.payload_bytes
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub fn record_payloads(&self) -> &[OfflineRecordPayloadObservation] {
        &self.record_payloads
    }
}

pub fn walk_current_durable_record_manifest(
    store_root: &Path,
    expected_format: PhysicalRecordFormatDeclaration,
) -> Result<OfflineDurableManifestWalk, OfflineDurableManifestDenial> {
    let catalog_path = store_root.join("families/records/bootstrap.catalog");
    let catalog_bytes = read_artifact(&catalog_path)?;
    let (store_identity, root_generation) = decode_catalog(&catalog_bytes, expected_format)?;
    walk_selected_durable_record_manifest(
        store_root,
        expected_format,
        store_identity,
        root_generation,
        catalog_bytes.len(),
    )
}

pub fn walk_non_current_durable_record_manifest(
    store_root: &Path,
    expected_format: PhysicalRecordFormatDeclaration,
    root_generation: u64,
) -> Result<OfflineDurableManifestWalk, OfflineDurableManifestDenial> {
    let catalog_path = store_root.join("families/records/bootstrap.catalog");
    let catalog_bytes = read_artifact(&catalog_path)?;
    let (store_identity, current_generation) = decode_catalog(&catalog_bytes, expected_format)?;
    if root_generation == current_generation {
        return Err(OfflineDurableManifestDenial::CurrentRootRequestedAsResidue);
    }
    walk_selected_durable_record_manifest(
        store_root,
        expected_format,
        store_identity,
        root_generation,
        catalog_bytes.len(),
    )
}

fn walk_selected_durable_record_manifest(
    store_root: &Path,
    expected_format: PhysicalRecordFormatDeclaration,
    store_identity: [u8; 16],
    root_generation: u64,
    catalog_bytes: usize,
) -> Result<OfflineDurableManifestWalk, OfflineDurableManifestDenial> {
    let root_path = store_root.join(format!(
        "families/records/roots/root-{root_generation:016x}.manifest"
    ));
    let root_bytes = read_artifact(&root_path)?;
    let root = decode_root_header(&root_bytes, root_generation, expected_format)?;
    let (placements, routing_blocks, routing_bytes) =
        walk_root_tree(store_root, &root, expected_format)?;
    let (segment_pages, segment_blocks, segment_bytes) =
        segment_tree::walk_segment_tree(store_root, &root, expected_format)?;
    let (free_space, free_blocks, free_bytes, arena) =
        free_space_tree::walk_free_space_tree(store_root, &root, expected_format)?;
    arena_ranges::validate(&placements, &free_space, arena)?;
    validate_reachable_membership(store_root, expected_format, &placements, &segment_pages)?;
    let (extent_bytes, extent_geometry) =
        validate_extent_manifests(store_root, expected_format, &placements)?;
    let payload_validation::OfflinePayloadWalk {
        frames_read,
        payload_bytes,
        payload_digest,
        records: record_payloads,
    } = payload_validation::validate_payloads(
        store_root,
        expected_format,
        &placements,
        &segment_pages,
        &extent_geometry,
    )?;
    Ok(OfflineDurableManifestWalk {
        store_identity,
        format_identity: expected_format.canonical_identity_bytes(),
        root_generation,
        tree_identity: root.tree_identity,
        node_capacity: root.node_capacity,
        routing_level: root.routing_root.map(|reference| reference.level),
        placements,
        segment_pages,
        free_space,
        manifest_blocks: routing_blocks
            .saturating_add(segment_blocks)
            .saturating_add(free_blocks),
        manifest_bytes: catalog_bytes
            .saturating_add(root_bytes.len())
            .saturating_add(routing_bytes as usize)
            .saturating_add(segment_bytes as usize)
            .saturating_add(free_bytes as usize)
            .saturating_add(extent_bytes as usize) as u64,
        payload_frames: frames_read,
        payload_bytes,
        payload_digest,
        record_payloads,
    })
}

fn decode_catalog(
    bytes: &[u8],
    format: PhysicalRecordFormatDeclaration,
) -> Result<([u8; 16], u64), OfflineDurableManifestDenial> {
    let frame = decode_frame(bytes, 1, format)?;
    if frame.payload.len() != 34
        || frame.payload[24..34] != format.canonical_identity_bytes()
        || frame.payload[..16] == [0; 16]
    {
        return Err(OfflineDurableManifestDenial::MalformedCatalog);
    }
    let generation = read_u64(frame.payload, 16);
    if generation == 0 || generation != frame.identity {
        return Err(OfflineDurableManifestDenial::MalformedCatalog);
    }
    Ok((frame.payload[..16].try_into().unwrap(), generation))
}

fn validate_reachable_membership(
    store_root: &Path,
    format: PhysicalRecordFormatDeclaration,
    placements: &[OfflineRecordPlacement],
    segment_pages: &[OfflineSegmentPageMembership],
) -> Result<(), OfflineDurableManifestDenial> {
    let expected = placements
        .iter()
        .filter_map(|placement| match placement {
            OfflineRecordPlacement::Inline {
                segment,
                page,
                segment_generation,
                page_generation,
                ..
            } => Some((*segment, *page, *page_generation, *segment_generation)),
            OfflineRecordPlacement::Extent { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    let actual = segment_pages
        .iter()
        .map(|entry| {
            (
                entry.segment,
                entry.page,
                entry.page_generation,
                entry.data_generation,
            )
        })
        .collect::<BTreeSet<_>>();
    if expected != actual {
        return Err(OfflineDurableManifestDenial::ReachabilityMismatch);
    }
    for entry in segment_pages {
        let path = store_root.join(format!(
            "families/records/segments/segment-{:016x}-{:016x}.pages",
            entry.segment, entry.data_generation
        ));
        let length = std::fs::metadata(path)
            .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?
            .len();
        if length != u64::from(entry.data_page_count) * u64::from(format.page_size().bytes()) {
            return Err(OfflineDurableManifestDenial::ReachabilityMismatch);
        }
    }
    Ok(())
}

fn validate_extent_manifests(
    store_root: &Path,
    format: PhysicalRecordFormatDeclaration,
    placements: &[OfflineRecordPlacement],
) -> Result<(u64, BTreeMap<(u64, u64), (u64, u64)>), OfflineDurableManifestDenial> {
    let mut bytes_read = 0_u64;
    let mut geometry = BTreeMap::new();
    for placement in placements {
        let OfflineRecordPlacement::Extent {
            record,
            extent,
            generation,
            payload_bytes,
            arena,
            arena_offset,
            arena_length,
        } = placement
        else {
            continue;
        };
        let path = store_root.join(format!("families/records/arenas/arena-{arena:016x}.data"));
        let mut file = std::fs::File::open(&path)
            .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?;
        file.seek(SeekFrom::Start(*arena_offset))
            .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?;
        let mut bytes = [0_u8; 104];
        file.read_exact(&mut bytes)
            .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?;
        bytes_read = bytes_read.saturating_add(bytes.len() as u64);
        let frame = decode_frame(&bytes, 6, format)?;
        let chunk_payload_bytes = format.page_size().bytes() as u64 - 48 - 64;
        let expected_chunks = payload_bytes.div_ceil(chunk_payload_bytes);
        if frame.payload.len() != 56
            || frame.identity != *generation
            || observation::OfflineRecordIdentity::decode(&frame.payload[..24]) != Some(*record)
            || read_u64(frame.payload, 24) != *extent
            || read_u64(frame.payload, 32) != *payload_bytes
            || read_u32(frame.payload, 40) != format.page_size().bytes()
            || u64::from(read_u32(frame.payload, 44)) != expected_chunks
        {
            return Err(OfflineDurableManifestDenial::ReachabilityMismatch);
        }
        let alignment = read_u64(frame.payload, 48);
        if !alignment.is_power_of_two() || !arena_offset.is_multiple_of(alignment) {
            return Err(OfflineDurableManifestDenial::ReachabilityMismatch);
        }
        let aligned = |bytes: u64| {
            bytes
                .checked_add(alignment - 1)
                .map(|bytes| bytes / alignment * alignment)
        };
        let manifest_stride =
            aligned(104).ok_or(OfflineDurableManifestDenial::ReachabilityMismatch)?;
        let chunk_stride = aligned(format.page_size().bytes() as u64)
            .ok_or(OfflineDurableManifestDenial::ReachabilityMismatch)?;
        if chunk_stride
            .checked_mul(expected_chunks)
            .and_then(|chunks| manifest_stride.checked_add(chunks))
            != Some(*arena_length)
            || arena_offset.checked_add(*arena_length).is_none()
            || geometry
                .insert((*arena, *arena_offset), (manifest_stride, chunk_stride))
                .is_some()
        {
            return Err(OfflineDurableManifestDenial::ReachabilityMismatch);
        }
    }
    Ok((bytes_read, geometry))
}

pub(super) fn read_artifact(path: &PathBuf) -> Result<Vec<u8>, OfflineDurableManifestDenial> {
    std::fs::read(path).map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))
}
