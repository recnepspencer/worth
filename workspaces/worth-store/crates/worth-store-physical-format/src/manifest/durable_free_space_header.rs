use crate::record_framing::{decode_durable_frame, encode_durable_frame};
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
            && next_block != 0
            && shape
            && root.is_none_or(|reference| {
                required_tree_level(next_segment, node_capacity)
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
    pub const fn next_block(&self) -> u64 {
        self.next_block
    }
    pub const fn root(&self) -> Option<FreeSpaceBlockReference> {
        self.root
    }
    pub fn encode(&self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let mut payload = vec![0_u8; 128];
        payload[..8].copy_from_slice(&self.generation.to_le_bytes());
        payload[8..16].copy_from_slice(&self.tree_identity.to_le_bytes());
        payload[16..18].copy_from_slice(&self.node_capacity.to_le_bytes());
        payload[18..22].copy_from_slice(&self.segment_page_capacity.to_le_bytes());
        payload[24..32].copy_from_slice(&self.entry_count.to_le_bytes());
        payload[32..40].copy_from_slice(&self.next_segment.to_le_bytes());
        payload[40..48].copy_from_slice(&self.next_page.to_le_bytes());
        payload[48..56].copy_from_slice(&self.next_extent.to_le_bytes());
        payload[56..64].copy_from_slice(&self.next_block.to_le_bytes());
        if let Some(root) = self.root {
            payload[64] = 1;
            encode_reference(&mut payload[72..128], root);
        }
        encode_durable_frame(
            DurableFrameKind::FreeSpaceManifest,
            format,
            self.generation,
            &payload,
        )
    }
    pub fn decode(
        bytes: &[u8],
        maximum_capacity: u16,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), FreeSpaceRoutingDenial> {
        let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::FreeSpaceManifest)
            .map_err(FreeSpaceRoutingDenial::Frame)?;
        if frame.payload.len() != 128
            || frame.payload[22..24] != [0; 2]
            || frame.payload[65..72] != [0; 7]
        {
            return Err(FreeSpaceRoutingDenial::Malformed);
        }
        let generation = read_u64(frame.payload, 0);
        let capacity = u16::from_le_bytes(frame.payload[16..18].try_into().unwrap());
        let segment_page_capacity = u32::from_le_bytes(frame.payload[18..22].try_into().unwrap());
        let root = match frame.payload[64] {
            0 if frame.payload[72..128].iter().all(|byte| *byte == 0) => None,
            0 => return Err(FreeSpaceRoutingDenial::Malformed),
            1 => Some(
                decode_reference(&frame.payload[72..128])
                    .ok_or(FreeSpaceRoutingDenial::InvalidReference)?,
            ),
            _ => return Err(FreeSpaceRoutingDenial::Malformed),
        };
        if generation != frame.identity
            || capacity > maximum_capacity
            || segment_page_capacity > crate::maximum_segment_manifest_pages(format)
        {
            return Err(FreeSpaceRoutingDenial::IdentityOrCapacity);
        }
        Self::new(
            generation,
            read_u64(frame.payload, 8),
            capacity,
            segment_page_capacity,
            read_u64(frame.payload, 24),
            read_u64(frame.payload, 32),
            read_u64(frame.payload, 40),
            read_u64(frame.payload, 48),
            read_u64(frame.payload, 56),
            root,
        )
        .map(|header| (header, format))
        .ok_or(FreeSpaceRoutingDenial::Malformed)
    }
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
