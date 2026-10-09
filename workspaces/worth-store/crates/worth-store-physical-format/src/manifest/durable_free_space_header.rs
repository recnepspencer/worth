use crate::record_framing::decode_durable_frame;
use crate::{DurableFrameKind, PhysicalRecordFormatDeclaration};

use super::free_space_routing::{
    decode_reference, encode_reference, FreeSpaceBlockReference, FreeSpaceRoutingDenial,
};
use super::routing_tree_height::required_tree_level;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableFreeSpaceManifestHeader {
    generation: u64,
    tree_identity: u64,
    node_capacity: u16,
    segment_page_capacity: u32,
    entry_count: u64,
    next_segment: u64,
    next_page: u64,
    next_extent: u64,
    next_arena: u64,
    tier_epoch_start: Option<u64>,
    arena_capacity: u64,
    arena_alignment: u64,
    next_block: u64,
    root: Option<FreeSpaceBlockReference>,
}

impl DurableFreeSpaceManifestHeader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        generation: u64,
        tree_identity: u64,
        node_capacity: u16,
        segment_page_capacity: u32,
        entry_count: u64,
        next_segment: u64,
        next_page: u64,
        next_extent: u64,
        next_arena: u64,
        arena_capacity: u64,
        arena_alignment: u64,
        next_block: u64,
        root: Option<FreeSpaceBlockReference>,
    ) -> Option<Self> {
        Self::new_with_tier_epoch(
            generation,
            tree_identity,
            node_capacity,
            segment_page_capacity,
            entry_count,
            next_segment,
            next_page,
            next_extent,
            next_arena,
            None,
            arena_capacity,
            arena_alignment,
            next_block,
            root,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_tier_epoch(
        generation: u64,
        tree_identity: u64,
        node_capacity: u16,
        segment_page_capacity: u32,
        entry_count: u64,
        next_segment: u64,
        next_page: u64,
        next_extent: u64,
        next_arena: u64,
        tier_epoch_start: Option<u64>,
        arena_capacity: u64,
        arena_alignment: u64,
        next_block: u64,
        root: Option<FreeSpaceBlockReference>,
    ) -> Option<Self> {
        let shape = matches!((entry_count, root), (0, None) | (1.., Some(_)));
        (generation != 0
            && tree_identity != 0
            && node_capacity >= 2
            && segment_page_capacity != 0
            && next_segment != 0
            && next_page != 0
            && next_extent != 0
            && next_arena != 0
            && tier_epoch_start.is_none_or(|epoch| epoch != 0 && epoch <= next_arena)
            && arena_alignment.is_power_of_two()
            && arena_capacity != 0
            && arena_capacity % arena_alignment == 0
            && next_block != 0
            && shape
            && root.is_none_or(|reference| {
                required_tree_level(entry_count, node_capacity)
                    .is_some_and(|maximum| reference.level() <= maximum)
                    && reference.generation() <= generation
                    && reference.block() < next_block
            }))
        .then_some(Self {
            generation,
            tree_identity,
            node_capacity,
            segment_page_capacity,
            entry_count,
            next_segment,
            next_page,
            next_extent,
            next_arena,
            tier_epoch_start,
            arena_capacity,
            arena_alignment,
            next_block,
            root,
        })
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn tree_identity(&self) -> u64 {
        self.tree_identity
    }
    pub const fn node_capacity(&self) -> u16 {
        self.node_capacity
    }
    pub const fn segment_page_capacity(&self) -> u32 {
        self.segment_page_capacity
    }
    pub const fn entry_count(&self) -> u64 {
        self.entry_count
    }
    pub const fn next_segment(&self) -> u64 {
        self.next_segment
    }
    pub const fn next_page(&self) -> u64 {
        self.next_page
    }
    pub const fn next_extent(&self) -> u64 {
        self.next_extent
    }
    pub const fn next_arena(&self) -> u64 {
        self.next_arena
    }
    pub const fn tier_epoch_start(&self) -> Option<u64> {
        self.tier_epoch_start
    }
    pub const fn next_block(&self) -> u64 {
        self.next_block
    }
    pub const fn arena_capacity(&self) -> u64 {
        self.arena_capacity
    }
    pub const fn arena_alignment(&self) -> u64 {
        self.arena_alignment
    }
    pub const fn root(&self) -> Option<FreeSpaceBlockReference> {
        self.root
    }
    pub fn encode(&self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let length = self.encoded_frame_bytes();
        self.encode_in_reserved(format, Vec::with_capacity(length))
            .expect("reserved free-space header encoding")
    }

    pub fn encoded_frame_bytes(&self) -> usize {
        crate::record_framing::DURABLE_FRAME_HEADER_BYTES
            + if self.tier_epoch_start.is_some() {
                176
            } else {
                168
            }
    }

    pub fn encode_in_reserved(
        &self,
        format: PhysicalRecordFormatDeclaration,
        frame: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let payload_bytes =
            self.encoded_frame_bytes() - crate::record_framing::DURABLE_FRAME_HEADER_BYTES;
        crate::record_framing::encode_durable_frame_in_reserved(
            DurableFrameKind::FreeSpaceManifest,
            format,
            self.generation,
            payload_bytes,
            crate::record_framing::FRAME_SCHEMA,
            frame,
            |payload| {
                payload[..8].copy_from_slice(&self.generation.to_le_bytes());
                payload[8..16].copy_from_slice(&self.tree_identity.to_le_bytes());
                payload[16..18].copy_from_slice(&self.node_capacity.to_le_bytes());
                payload[18..22].copy_from_slice(&self.segment_page_capacity.to_le_bytes());
                payload[24..32].copy_from_slice(&self.entry_count.to_le_bytes());
                payload[32..40].copy_from_slice(&self.next_segment.to_le_bytes());
                payload[40..48].copy_from_slice(&self.next_page.to_le_bytes());
                payload[48..56].copy_from_slice(&self.next_extent.to_le_bytes());
                payload[56..64].copy_from_slice(&self.next_block.to_le_bytes());
                payload[144..152].copy_from_slice(&self.next_arena.to_le_bytes());
                payload[152..160].copy_from_slice(&self.arena_capacity.to_le_bytes());
                payload[160..168].copy_from_slice(&self.arena_alignment.to_le_bytes());
                if let Some(epoch) = self.tier_epoch_start {
                    payload[168..176].copy_from_slice(&epoch.to_le_bytes());
                }
                if let Some(root) = self.root {
                    payload[64] = 1;
                    encode_reference(&mut payload[72..144], root);
                }
            },
        )
    }
    pub fn decode(
        bytes: &[u8],
        maximum_capacity: u16,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), FreeSpaceRoutingDenial> {
        let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::FreeSpaceManifest)
            .map_err(FreeSpaceRoutingDenial::Frame)?;
        Self::project_payload(frame.payload, frame.identity, format, maximum_capacity)
            .map(|header| (header, format))
    }

    /// Projects header payload fields; grants no integrity or persisted-source authority.
    pub fn project_payload(
        payload: &[u8],
        frame_generation: u64,
        format: PhysicalRecordFormatDeclaration,
        maximum_capacity: u16,
    ) -> Result<Self, FreeSpaceRoutingDenial> {
        if !matches!(payload.len(), 168 | 176)
            || payload[22..24] != [0; 2]
            || payload[65..72] != [0; 7]
        {
            return Err(FreeSpaceRoutingDenial::Malformed);
        }
        let generation = read_u64(payload, 0);
        let capacity = u16::from_le_bytes(payload[16..18].try_into().unwrap());
        let segment_page_capacity = u32::from_le_bytes(payload[18..22].try_into().unwrap());
        let root = match payload[64] {
            0 if payload[72..144].iter().all(|byte| *byte == 0) => None,
            0 => return Err(FreeSpaceRoutingDenial::Malformed),
            1 => Some(
                decode_reference(&payload[72..144])
                    .ok_or(FreeSpaceRoutingDenial::InvalidReference)?,
            ),
            _ => return Err(FreeSpaceRoutingDenial::Malformed),
        };
        if generation != frame_generation
            || capacity > maximum_capacity
            || segment_page_capacity > crate::maximum_segment_manifest_pages(format)
        {
            return Err(FreeSpaceRoutingDenial::IdentityOrCapacity);
        }
        Self::new_with_tier_epoch(
            generation,
            read_u64(payload, 8),
            capacity,
            segment_page_capacity,
            read_u64(payload, 24),
            read_u64(payload, 32),
            read_u64(payload, 40),
            read_u64(payload, 48),
            read_u64(payload, 144),
            (payload.len() == 176).then(|| read_u64(payload, 168)),
            read_u64(payload, 152),
            read_u64(payload, 160),
            read_u64(payload, 56),
            root,
        )
        .ok_or(FreeSpaceRoutingDenial::Malformed)
    }
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(epoch: Option<u64>) -> Option<DurableFreeSpaceManifestHeader> {
        DurableFreeSpaceManifestHeader::new_with_tier_epoch(
            1,
            1,
            2,
            4,
            0,
            1,
            1,
            1,
            7,
            epoch,
            64 << 20,
            4096,
            1,
            None,
        )
    }

    #[test]
    fn tier_epoch_is_versioned_and_survives_exact_header_roundtrip() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        for (epoch, payload_len) in [(None, 168), (Some(5), 176)] {
            let original = header(epoch).unwrap();
            let bytes = original.encode(format);
            let (_, frame) =
                decode_durable_frame(&bytes, DurableFrameKind::FreeSpaceManifest).unwrap();
            assert_eq!(frame.payload.len(), payload_len);
            let (reopened, reopened_format) =
                DurableFreeSpaceManifestHeader::decode(&bytes, 2).unwrap();
            assert_eq!(reopened, original);
            assert_eq!(reopened_format, format);
        }
        assert!(header(Some(0)).is_none());
        assert!(header(Some(8)).is_none());
    }

    #[test]
    fn reserved_tier_header_frame_requires_full_backing_and_roundtrips() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let header = header(Some(5)).unwrap();
        let required = header.encoded_frame_bytes();
        assert_eq!(required, 224);
        assert!(header
            .encode_in_reserved(format, Vec::with_capacity(required - 1))
            .is_none());
        let bytes = header
            .encode_in_reserved(format, Vec::with_capacity(required))
            .unwrap();
        assert_eq!(bytes[8], DurableFrameKind::FreeSpaceManifest as u8);
        assert_eq!(bytes.len(), required);
        assert_eq!(
            DurableFreeSpaceManifestHeader::decode(&bytes, 2),
            Ok((header, format))
        );
    }
}
