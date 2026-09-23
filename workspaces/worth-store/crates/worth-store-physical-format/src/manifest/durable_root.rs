use crate::record_framing::{decode_durable_frame, encode_durable_frame_schema, FRAME_SCHEMA};
use crate::{
    DurableFrameDenial, DurableFrameKind, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration, PhysicalRootReference,
    PhysicalSegmentId, RootPublicationCell, SegmentGenerationCell,
};

use super::durable_root_routing::{
    decode_identity, decode_reference, encode_identity, encode_reference, ManifestBlockReference,
};
use super::durable_segment_routing::{
    decode_reference as decode_segment_reference, encode_reference as encode_segment_reference,
    SegmentManifestBlockReference,
};
use super::free_space_routing::{
    decode_reference as decode_free_space_reference,
    encode_reference as encode_free_space_reference, FreeSpaceBlockReference,
};
use super::routing_tree_height::required_tree_level;

pub const CURRENT_ROOT_MANIFEST_PREFIX_BYTES: usize = 24;
pub const CURRENT_ROOT_MANIFEST_ENTRY_BYTES: usize = 88;

pub const fn maximum_current_root_entries(format: PhysicalRecordFormatDeclaration) -> u16 {
    let available = format.page_size().bytes() as usize
        - crate::record_framing::DURABLE_FRAME_HEADER_BYTES
        - CURRENT_ROOT_MANIFEST_PREFIX_BYTES;
    let entries = available / CURRENT_ROOT_MANIFEST_ENTRY_BYTES;
    if entries > u16::MAX as usize {
        u16::MAX
    } else {
        entries as u16
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurablePhysicalRootManifest {
    root: RootPublicationCell,
    tree_identity: u64,
    node_capacity: u16,
    record_count: u64,
    next_block: u64,
    next_segment_block: u64,
    free_space_checksum: u32,
    routing_root: Option<ManifestBlockReference>,
    segment_root: Option<SegmentManifestBlockReference>,
    free_space_root: Option<FreeSpaceBlockReference>,
    last_inline_record: Option<PersistedRecordIdentity>,
    last_inline_segment: Option<SegmentGenerationCell>,
    requires_maintenance_protocol: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurablePhysicalRootManifestBuilder {
    generation: u64,
    tree_identity: u64,
    node_capacity: u16,
    free_space_checksum: u32,
    record_count: u64,
    next_block: u64,
    next_segment_block: u64,
    routing_root: Option<ManifestBlockReference>,
    segment_root: Option<SegmentManifestBlockReference>,
    free_space_root: Option<FreeSpaceBlockReference>,
    last_inline_record: Option<PersistedRecordIdentity>,
    last_inline_segment: Option<SegmentGenerationCell>,
}

impl DurablePhysicalRootManifest {
    pub const fn builder(
        generation: u64,
        tree_identity: u64,
        node_capacity: u16,
        free_space_checksum: u32,
    ) -> DurablePhysicalRootManifestBuilder {
        DurablePhysicalRootManifestBuilder {
            generation,
            tree_identity,
            node_capacity,
            free_space_checksum,
            record_count: 0,
            next_block: 1,
            next_segment_block: 1,
            routing_root: None,
            segment_root: None,
            free_space_root: None,
            last_inline_record: None,
            last_inline_segment: None,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.root.generation().get()
    }
    pub const fn root_cell(&self) -> RootPublicationCell {
        self.root
    }
    pub const fn node_capacity(&self) -> u16 {
        self.node_capacity
    }
    pub const fn tree_identity(&self) -> u64 {
        self.tree_identity
    }
    pub const fn record_count(&self) -> u64 {
        self.record_count
    }
    pub const fn next_block(&self) -> u64 {
        self.next_block
    }
    pub const fn next_segment_block(&self) -> u64 {
        self.next_segment_block
    }
    pub const fn free_space_checksum(&self) -> u32 {
        self.free_space_checksum
    }
    pub const fn routing_root(&self) -> Option<ManifestBlockReference> {
        self.routing_root
    }
    pub const fn segment_root(&self) -> Option<SegmentManifestBlockReference> {
        self.segment_root
    }
    pub const fn free_space_root(&self) -> Option<FreeSpaceBlockReference> {
        self.free_space_root
    }
    pub const fn last_inline_record(&self) -> Option<PersistedRecordIdentity> {
        self.last_inline_record
    }
    pub const fn last_inline_segment(&self) -> Option<SegmentGenerationCell> {
        self.last_inline_segment
    }

    pub fn encode(&self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let mut payload = vec![0_u8; 320];
        payload[..8].copy_from_slice(&self.generation().to_le_bytes());
        payload[8..16].copy_from_slice(&self.tree_identity.to_le_bytes());
        payload[16..18].copy_from_slice(&self.node_capacity.to_le_bytes());
        payload[24..32].copy_from_slice(&self.record_count.to_le_bytes());
        payload[32..40].copy_from_slice(&self.next_block.to_le_bytes());
        payload[152..156].copy_from_slice(&self.free_space_checksum.to_le_bytes());
        payload[224..232].copy_from_slice(&self.next_segment_block.to_le_bytes());
        if let Some(reference) = self.routing_root {
            payload[40] = 1;
            encode_reference(&mut payload[48..120], reference);
        }
        if let Some(record) = self.last_inline_record {
            payload[120] = 1;
            encode_identity(&mut payload[128..152], record);
        }
        if let Some(reference) = self.segment_root {
            payload[160] = 1;
            encode_segment_reference(&mut payload[168..224], reference);
        }
        if let Some(reference) = self.free_space_root {
            payload[232] = 1;
            encode_free_space_reference(&mut payload[240..296], reference);
        }
        if let Some(segment) = self.last_inline_segment {
            payload[296] = 1;
            payload[304..312].copy_from_slice(&segment.segment_id().get().to_le_bytes());
            payload[312..320].copy_from_slice(&segment.generation().get().to_le_bytes());
        }
        encode_durable_frame_schema(
            DurableFrameKind::RootManifest,
            format,
            self.generation(),
            &payload,
            FRAME_SCHEMA + u8::from(self.requires_maintenance_protocol),
        )
    }

    pub fn decode(
        bytes: &[u8],
        max_entries: u16,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), RootManifestDenial> {
        let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::RootManifest)
            .map_err(RootManifestDenial::Frame)?;
        if frame.payload.len() != 320
            || frame.payload[18..24] != [0; 6]
            || frame.payload[41..48] != [0; 7]
            || frame.payload[121..128] != [0; 7]
            || frame.payload[156..160] != [0; 4]
            || frame.payload[161..168] != [0; 7]
            || frame.payload[233..240] != [0; 7]
            || frame.payload[297..304] != [0; 7]
        {
            return Err(RootManifestDenial::MalformedPrefix);
        }
        let generation = u64::from_le_bytes(frame.payload[..8].try_into().unwrap());
        let tree_identity = u64::from_le_bytes(frame.payload[8..16].try_into().unwrap());
        let node_capacity = u16::from_le_bytes(frame.payload[16..18].try_into().unwrap());
        let record_count = u64::from_le_bytes(frame.payload[24..32].try_into().unwrap());
        let next_block = u64::from_le_bytes(frame.payload[32..40].try_into().unwrap());
        let free_space_checksum = u32::from_le_bytes(frame.payload[152..156].try_into().unwrap());
        let next_segment_block = u64::from_le_bytes(frame.payload[224..232].try_into().unwrap());
        if generation == 0 || generation != frame.identity {
            return Err(RootManifestDenial::IdentityMismatch);
        }
        if node_capacity < 2 || node_capacity > max_entries {
            return Err(RootManifestDenial::EntryLimitExceeded);
        }
        let routing_root = match frame.payload[40] {
            0 => None,
            1 => Some(
                decode_reference(&frame.payload[48..120])
                    .ok_or(RootManifestDenial::InvalidPlacement)?,
            ),
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        let last_inline_record = match frame.payload[120] {
            0 => None,
            1 => Some(
                decode_identity(&frame.payload[128..152])
                    .ok_or(RootManifestDenial::InvalidRecordIdentity)?,
            ),
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        let segment_root = match frame.payload[160] {
            0 => None,
            1 => Some(
                decode_segment_reference(&frame.payload[168..224])
                    .ok_or(RootManifestDenial::InvalidPlacement)?,
            ),
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        let free_space_root = match frame.payload[232] {
            0 => None,
            1 => Some(
                decode_free_space_reference(&frame.payload[240..296])
                    .ok_or(RootManifestDenial::InvalidPlacement)?,
            ),
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        let last_inline_segment = match frame.payload[296] {
            0 => None,
            1 => {
                let segment = PhysicalSegmentId::from_raw(u64::from_le_bytes(
                    frame.payload[304..312].try_into().unwrap(),
                ))
                .map_err(|_| RootManifestDenial::InvalidPlacement)?;
                let generation = PhysicalGeneration::from_raw(u64::from_le_bytes(
                    frame.payload[312..320].try_into().unwrap(),
                ))
                .map_err(|_| RootManifestDenial::InvalidPlacement)?;
                Some(
                    PhysicalGenerationAuthority::for_canonical_physical_format()
                        .segment_cell(segment)
                        .with_segment_generation(generation),
                )
            }
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        Self::builder(
            generation,
            tree_identity,
            node_capacity,
            free_space_checksum,
        )
        .record_count(record_count)
        .next_block(next_block)
        .next_segment_block(next_segment_block)
        .routing_root(routing_root)
        .segment_root(segment_root)
        .free_space_root(free_space_root)
        .last_inline_record(last_inline_record)
        .last_inline_segment(last_inline_segment)
        .admit()
        .map(|manifest| (manifest.with_root_schema(frame.schema), format))
        .ok_or(RootManifestDenial::InvalidPlacement)
    }
}

impl DurablePhysicalRootManifestBuilder {
    pub const fn record_count(mut self, record_count: u64) -> Self {
        self.record_count = record_count;
        self
    }
    pub const fn next_block(mut self, next_block: u64) -> Self {
        self.next_block = next_block;
        self
    }
    pub const fn next_segment_block(mut self, next_segment_block: u64) -> Self {
        self.next_segment_block = next_segment_block;
        self
    }
    pub const fn routing_root(mut self, routing_root: Option<ManifestBlockReference>) -> Self {
        self.routing_root = routing_root;
        self
    }
    pub const fn segment_root(
        mut self,
        segment_root: Option<SegmentManifestBlockReference>,
    ) -> Self {
        self.segment_root = segment_root;
        self
    }
    pub const fn free_space_root(
        mut self,
        free_space_root: Option<FreeSpaceBlockReference>,
    ) -> Self {
        self.free_space_root = free_space_root;
        self
    }
    pub const fn last_inline_record(
        mut self,
        last_inline_record: Option<PersistedRecordIdentity>,
    ) -> Self {
        self.last_inline_record = last_inline_record;
        self
    }
    pub const fn last_inline_segment(
        mut self,
        last_inline_segment: Option<SegmentGenerationCell>,
    ) -> Self {
        self.last_inline_segment = last_inline_segment;
        self
    }

    pub fn admit(self) -> Option<DurablePhysicalRootManifest> {
        let Self {
            generation,
            tree_identity,
            node_capacity,
            free_space_checksum,
            record_count,
            next_block,
            next_segment_block,
            routing_root,
            segment_root,
            free_space_root,
            last_inline_record,
            last_inline_segment,
        } = self;
        let generation = PhysicalGeneration::from_raw(generation).ok()?;
        let root_reference = PhysicalRootReference::from_raw(generation.get()).ok()?;
        let tail_shape_is_valid = last_inline_record.is_some() == last_inline_segment.is_some();
        let shape_is_valid = match (record_count, routing_root) {
            (0, None) => last_inline_record.is_none() && last_inline_segment.is_none(),
            (0, Some(_)) | (_, None) => false,
            (_, Some(reference)) => {
                required_tree_level(record_count, node_capacity) == Some(reference.level())
                    && reference.generation() <= generation.get()
                    && reference.block() < next_block
                    && last_inline_record.is_none_or(|record| reference.contains(record))
            }
        };
        let segment_shape_is_valid = segment_root.is_none_or(|reference| {
            required_tree_level(record_count, node_capacity)
                .is_some_and(|maximum| reference.level() <= maximum)
                && reference.generation() <= generation.get()
                && reference.block() < next_segment_block
        });
        let free_space_shape_is_valid =
            free_space_root.is_some_and(|reference| reference.generation() <= generation.get());
        if tree_identity == 0
            || node_capacity < 2
            || next_block == 0
            || next_segment_block == 0
            || free_space_checksum == 0
            || !shape_is_valid
            || !tail_shape_is_valid
            || !segment_shape_is_valid
            || !free_space_shape_is_valid
        {
            return None;
        }
        Some(DurablePhysicalRootManifest {
            root: PhysicalGenerationAuthority::for_canonical_physical_format()
                .root_publication_cell(root_reference)
                .with_root_publication_generation(generation),
            tree_identity,
            node_capacity,
            record_count,
            next_block,
            next_segment_block,
            free_space_checksum,
            routing_root,
            segment_root,
            free_space_root,
            last_inline_record,
            last_inline_segment,
            requires_maintenance_protocol: false,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootManifestDenial {
    Frame(DurableFrameDenial),
    MalformedPrefix,
    IdentityMismatch,
    EntryLimitExceeded,
    MalformedEntryLength,
    ReservedFieldNonZero,
    InvalidRecordIdentity,
    InvalidPlacement,
}

#[path = "maintenance.rs"]
mod maintenance;
