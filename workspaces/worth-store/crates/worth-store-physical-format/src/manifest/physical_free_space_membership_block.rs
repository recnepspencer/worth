mod payload_projection;

use crate::record_framing::{decode_durable_frame, encode_durable_frame};
use crate::{
    DurableFrameKind, PhysicalRecordFormatDeclaration, RecordAllocationClass,
    RecordFreeSpaceManifestEntry,
};

use super::free_space_routing::{
    decode_reference, encode_reference, FreeSpaceBlockReference, FreeSpaceKey,
    FreeSpaceRoutingDenial,
};

const BLOCK_PREFIX_BYTES: usize = 40;
const REFERENCE_BYTES: usize = 56;
const ENTRY_BYTES: usize = 40;

#[cfg(test)]
#[path = "physical_free_space_membership_block/bounded_decode_tests.rs"]
mod bounded_decode_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeSpaceMembershipBlockDecodeLimits {
    pub leaf_entries: u64,
    pub branch_children: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundedFreeSpaceMembershipBlockDecodeDenial {
    Format(FreeSpaceRoutingDenial),
    LeafEntries { observed: u64, admitted: u64 },
    BranchChildren { observed: u64, admitted: u64 },
}

impl From<FreeSpaceRoutingDenial> for BoundedFreeSpaceMembershipBlockDecodeDenial {
    fn from(value: FreeSpaceRoutingDenial) -> Self {
        Self::Format(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalFreeSpaceMembershipBlock {
    Leaf {
        tree_identity: u64,
        generation: u64,
        block: u64,
        entries: Vec<RecordFreeSpaceManifestEntry>,
    },
    Branch {
        tree_identity: u64,
        generation: u64,
        block: u64,
        level: u16,
        children: Vec<FreeSpaceBlockReference>,
    },
}

impl PhysicalFreeSpaceMembershipBlock {
    pub fn leaf(
        tree_identity: u64,
        generation: u64,
        block: u64,
        entries: Vec<RecordFreeSpaceManifestEntry>,
        capacity: u16,
    ) -> Option<Self> {
        (tree_identity != 0
            && generation != 0
            && block != 0
            && !entries.is_empty()
            && entries.len() <= usize::from(capacity)
            && entries
                .windows(2)
                .all(|pair| FreeSpaceKey::from(pair[0]) < FreeSpaceKey::from(pair[1])))
        .then_some(Self::Leaf {
            tree_identity,
            generation,
            block,
            entries,
        })
    }
    pub fn branch(
        tree_identity: u64,
        generation: u64,
        block: u64,
        level: u16,
        children: Vec<FreeSpaceBlockReference>,
        capacity: u16,
    ) -> Option<Self> {
        (tree_identity != 0
            && generation != 0
            && block != 0
            && level != 0
            && !children.is_empty()
            && children.len() <= usize::from(capacity)
            && children.iter().all(|child| {
                child.level().checked_add(1) == Some(level) && child.generation() <= generation
            })
            && children
                .windows(2)
                .all(|pair| pair[0].last() < pair[1].first()))
        .then_some(Self::Branch {
            tree_identity,
            generation,
            block,
            level,
            children,
        })
    }
    pub const fn tree_identity(&self) -> u64 {
        match self {
            Self::Leaf { tree_identity, .. } | Self::Branch { tree_identity, .. } => *tree_identity,
        }
    }
    pub const fn generation(&self) -> u64 {
        match self {
            Self::Leaf { generation, .. } | Self::Branch { generation, .. } => *generation,
        }
    }
    pub const fn block(&self) -> u64 {
        match self {
            Self::Leaf { block, .. } | Self::Branch { block, .. } => *block,
        }
    }
    pub const fn level(&self) -> u16 {
        match self {
            Self::Leaf { .. } => 0,
            Self::Branch { level, .. } => *level,
        }
    }
    pub fn reference(&self, checksum: u32) -> FreeSpaceBlockReference {
        let (first, last) = match self {
            Self::Leaf { entries, .. } => (
                FreeSpaceKey::from(*entries.first().expect("validated leaf")),
                FreeSpaceKey::from(*entries.last().expect("validated leaf")),
            ),
            Self::Branch { children, .. } => (
                children.first().expect("validated branch").first(),
                children.last().expect("validated branch").last(),
            ),
        };
        FreeSpaceBlockReference::new(
            self.generation(),
            self.block(),
            self.level(),
            checksum,
            first,
            last,
        )
        .expect("validated block")
    }
    pub fn encode(&self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let (kind, count, width) = match self {
            Self::Leaf { entries, .. } => (1, entries.len(), ENTRY_BYTES),
            Self::Branch { children, .. } => (2, children.len(), REFERENCE_BYTES),
        };
        let mut payload = vec![0_u8; BLOCK_PREFIX_BYTES + count * width];
        payload[..8].copy_from_slice(&self.tree_identity().to_le_bytes());
        payload[8..16].copy_from_slice(&self.block().to_le_bytes());
        payload[16..18].copy_from_slice(&self.level().to_le_bytes());
        payload[18..20].copy_from_slice(&(count as u16).to_le_bytes());
        payload[20] = kind;
        payload[24..32].copy_from_slice(&self.generation().to_le_bytes());
        match self {
            Self::Leaf { entries, .. } => entries.iter().enumerate().for_each(|(index, entry)| {
                encode_entry(&mut payload[BLOCK_PREFIX_BYTES + index * width..], *entry);
            }),
            Self::Branch { children, .. } => {
                children.iter().enumerate().for_each(|(index, child)| {
                    encode_reference(&mut payload[BLOCK_PREFIX_BYTES + index * width..], *child);
                })
            }
        }
        encode_durable_frame(
            DurableFrameKind::FreeSpaceMembershipBlock,
            format,
            self.block(),
            &payload,
        )
    }
    pub fn decode(
        bytes: &[u8],
        capacity: u16,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), FreeSpaceRoutingDenial> {
        match Self::decode_bounded(
            bytes,
            capacity,
            FreeSpaceMembershipBlockDecodeLimits {
                leaf_entries: u64::MAX,
                branch_children: u64::MAX,
            },
        ) {
            Ok(decoded) => Ok(decoded),
            Err(BoundedFreeSpaceMembershipBlockDecodeDenial::Format(denial)) => Err(denial),
            Err(
                BoundedFreeSpaceMembershipBlockDecodeDenial::LeafEntries { .. }
                | BoundedFreeSpaceMembershipBlockDecodeDenial::BranchChildren { .. },
            ) => unreachable!("unbounded free-space decode cannot exceed cardinality"),
        }
    }

    pub fn decode_bounded(
        bytes: &[u8],
        capacity: u16,
        limits: FreeSpaceMembershipBlockDecodeLimits,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), BoundedFreeSpaceMembershipBlockDecodeDenial>
    {
        let (format, frame) =
            decode_durable_frame(bytes, DurableFrameKind::FreeSpaceMembershipBlock)
                .map_err(FreeSpaceRoutingDenial::Frame)?;
        Self::project_payload(frame.payload, frame.identity, capacity, limits)
            .map(|block| (block, format))
    }
    pub fn entries(&self) -> Option<&[RecordFreeSpaceManifestEntry]> {
        match self {
            Self::Leaf { entries, .. } => Some(entries),
            Self::Branch { .. } => None,
        }
    }
    pub fn children(&self) -> Option<&[FreeSpaceBlockReference]> {
        match self {
            Self::Branch { children, .. } => Some(children),
            Self::Leaf { .. } => None,
        }
    }
}

fn encode_entry(target: &mut [u8], entry: RecordFreeSpaceManifestEntry) {
    target[0] = entry.class() as u8;
    target[8..16].copy_from_slice(&entry.owner().to_le_bytes());
    target[16..24].copy_from_slice(&entry.first_unallocated().to_le_bytes());
    target[24..32].copy_from_slice(&entry.unallocated_count().to_le_bytes());
    target[32..40].copy_from_slice(&entry.generation().to_le_bytes());
}

fn decode_entry(bytes: &[u8]) -> Option<RecordFreeSpaceManifestEntry> {
    if bytes[1..8] != [0; 7] {
        return None;
    }
    let class = match bytes[0] {
        1 => RecordAllocationClass::InlinePage,
        2 => RecordAllocationClass::Extent,
        _ => return None,
    };
    RecordFreeSpaceManifestEntry::new(
        class,
        read_u64(bytes, 8),
        read_u64(bytes, 16),
        read_u64(bytes, 24),
        read_u64(bytes, 32),
    )
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
