mod payload_projection;

use crate::record_framing::{decode_durable_frame, encode_durable_frame};
use crate::{
    DurableFrameDenial, DurableFrameKind, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordFormatDeclaration, PhysicalSegmentId,
    RecordSegmentPageManifestEntry,
};

const BLOCK_PREFIX_BYTES: usize = 40;
const REFERENCE_BYTES: usize = 56;
const LEAF_ENTRY_BYTES: usize = 40;

#[cfg(test)]
#[path = "durable_segment_routing/bounded_decode_tests.rs"]
mod bounded_decode_tests;
#[path = "durable_segment_routing/codec_primitives.rs"]
mod codec_primitives;
use codec_primitives::{decode_entry, encode_entry};
pub(crate) use codec_primitives::{decode_reference, encode_reference};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SegmentPageKey {
    segment: PhysicalSegmentId,
    page: PhysicalPageId,
}

impl SegmentPageKey {
    pub const fn new(segment: PhysicalSegmentId, page: PhysicalPageId) -> Self {
        Self { segment, page }
    }

    pub const fn segment(self) -> PhysicalSegmentId {
        self.segment
    }

    pub const fn page(self) -> PhysicalPageId {
        self.page
    }
}

impl From<RecordSegmentPageManifestEntry> for SegmentPageKey {
    fn from(entry: RecordSegmentPageManifestEntry) -> Self {
        Self::new(entry.page_cell().segment_id(), entry.page())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentManifestBlockReference {
    generation: u64,
    block: u64,
    level: u16,
    checksum: u32,
    first: SegmentPageKey,
    last: SegmentPageKey,
}

impl SegmentManifestBlockReference {
    pub fn new(
        generation: u64,
        block: u64,
        level: u16,
        checksum: u32,
        first: SegmentPageKey,
        last: SegmentPageKey,
    ) -> Option<Self> {
        (generation != 0 && block != 0 && first <= last).then_some(Self {
            generation,
            block,
            level,
            checksum,
            first,
            last,
        })
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn block(self) -> u64 {
        self.block
    }

    pub const fn level(self) -> u16 {
        self.level
    }
    pub const fn checksum(self) -> u32 {
        self.checksum
    }

    pub const fn first(self) -> SegmentPageKey {
        self.first
    }

    pub const fn last(self) -> SegmentPageKey {
        self.last
    }

    pub fn contains(self, key: SegmentPageKey) -> bool {
        self.first <= key && key <= self.last
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalSegmentMembershipBlock {
    Leaf {
        tree_identity: u64,
        generation: u64,
        block: u64,
        entries: Vec<RecordSegmentPageManifestEntry>,
    },
    Branch {
        tree_identity: u64,
        generation: u64,
        block: u64,
        level: u16,
        children: Vec<SegmentManifestBlockReference>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentMembershipBlockDecodeLimits {
    pub leaf_entries: u64,
    pub branch_children: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundedSegmentMembershipBlockDecodeDenial {
    Format(SegmentMembershipBlockDenial),
    LeafEntries { observed: u64, admitted: u64 },
    BranchChildren { observed: u64, admitted: u64 },
}

impl From<SegmentMembershipBlockDenial> for BoundedSegmentMembershipBlockDecodeDenial {
    fn from(value: SegmentMembershipBlockDenial) -> Self {
        Self::Format(value)
    }
}

impl PhysicalSegmentMembershipBlock {
    pub fn leaf(
        tree_identity: u64,
        generation: u64,
        block: u64,
        entries: Vec<RecordSegmentPageManifestEntry>,
        capacity: u16,
    ) -> Option<Self> {
        (tree_identity != 0
            && generation != 0
            && block != 0
            && !entries.is_empty()
            && entries.len() <= usize::from(capacity)
            && entries
                .windows(2)
                .all(|pair| SegmentPageKey::from(pair[0]) < SegmentPageKey::from(pair[1])))
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
        children: Vec<SegmentManifestBlockReference>,
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

    pub fn entries(&self) -> Option<&[RecordSegmentPageManifestEntry]> {
        match self {
            Self::Leaf { entries, .. } => Some(entries),
            Self::Branch { .. } => None,
        }
    }

    pub fn children(&self) -> Option<&[SegmentManifestBlockReference]> {
        match self {
            Self::Branch { children, .. } => Some(children),
            Self::Leaf { .. } => None,
        }
    }

    pub fn reference(&self, checksum: u32) -> SegmentManifestBlockReference {
        let (first, last) = match self {
            Self::Leaf { entries, .. } => (
                SegmentPageKey::from(*entries.first().expect("validated leaf")),
                SegmentPageKey::from(*entries.last().expect("validated leaf")),
            ),
            Self::Branch { children, .. } => (
                children.first().expect("validated branch").first(),
                children.last().expect("validated branch").last(),
            ),
        };
        SegmentManifestBlockReference::new(
            self.generation(),
            self.block(),
            self.level(),
            checksum,
            first,
            last,
        )
        .expect("validated membership block")
    }

    pub fn encode(&self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let (kind, count, entry_bytes) = match self {
            Self::Leaf { entries, .. } => (1_u8, entries.len(), LEAF_ENTRY_BYTES),
            Self::Branch { children, .. } => (2_u8, children.len(), REFERENCE_BYTES),
        };
        let mut payload = vec![0_u8; BLOCK_PREFIX_BYTES + count * entry_bytes];
        payload[..8].copy_from_slice(&self.tree_identity().to_le_bytes());
        payload[8..16].copy_from_slice(&self.block().to_le_bytes());
        payload[16..18].copy_from_slice(&self.level().to_le_bytes());
        payload[18..20].copy_from_slice(&(count as u16).to_le_bytes());
        payload[20] = kind;
        payload[24..32].copy_from_slice(&self.generation().to_le_bytes());
        match self {
            Self::Leaf { entries, .. } => entries.iter().enumerate().for_each(|(index, entry)| {
                encode_entry(
                    &mut payload[BLOCK_PREFIX_BYTES + index * entry_bytes..],
                    *entry,
                );
            }),
            Self::Branch { children, .. } => {
                children.iter().enumerate().for_each(|(index, child)| {
                    encode_reference(
                        &mut payload[BLOCK_PREFIX_BYTES + index * entry_bytes..],
                        *child,
                    );
                });
            }
        }
        encode_durable_frame(
            DurableFrameKind::SegmentMembershipBlock,
            format,
            self.block(),
            &payload,
        )
    }

    pub fn decode(
        bytes: &[u8],
        capacity: u16,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), SegmentMembershipBlockDenial> {
        match Self::decode_bounded(
            bytes,
            capacity,
            SegmentMembershipBlockDecodeLimits {
                leaf_entries: u64::MAX,
                branch_children: u64::MAX,
            },
        ) {
            Ok(decoded) => Ok(decoded),
            Err(BoundedSegmentMembershipBlockDecodeDenial::Format(denial)) => Err(denial),
            Err(
                BoundedSegmentMembershipBlockDecodeDenial::LeafEntries { .. }
                | BoundedSegmentMembershipBlockDecodeDenial::BranchChildren { .. },
            ) => unreachable!("unbounded segment-membership decode cannot exceed cardinality"),
        }
    }

    pub fn decode_bounded(
        bytes: &[u8],
        capacity: u16,
        limits: SegmentMembershipBlockDecodeLimits,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), BoundedSegmentMembershipBlockDecodeDenial>
    {
        let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::SegmentMembershipBlock)
            .map_err(SegmentMembershipBlockDenial::Frame)?;
        Self::project_payload(frame.payload, frame.identity, capacity, limits)
            .map(|block| (block, format))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentMembershipBlockDenial {
    Frame(DurableFrameDenial),
    MalformedPrefix,
    IdentityOrCapacity,
    LevelOrKind,
    MalformedLength,
    InvalidEntry,
    InvalidReference,
    CanonicalOrder,
}
