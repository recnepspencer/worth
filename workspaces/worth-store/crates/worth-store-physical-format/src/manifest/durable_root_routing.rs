mod payload_projection;

use std::collections::BTreeSet;

use crate::record_framing::{decode_durable_frame, encode_durable_frame};
use crate::{
    DurableFrameDenial, DurableFrameKind, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
};

use super::durable_root::RootManifestDenial;
use super::durable_root_entry::{decode_entry, encode_entry};
use super::durable_root_placement::CurrentPhysicalRecordPlacement;

#[cfg(test)]
mod bounded_decode_tests;
mod decode_limits;

pub use decode_limits::{BoundedRootRoutingBlockDecodeDenial, RootRoutingBlockDecodeLimits};

const ROUTING_BLOCK_PREFIX_BYTES: usize = 40;
const ROUTING_REFERENCE_BYTES: usize = 72;
const ROUTING_LEAF_ENTRY_BYTES: usize = 88;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestBlockReference {
    generation: u64,
    block: u64,
    level: u16,
    checksum: u32,
    first: PersistedRecordIdentity,
    last: PersistedRecordIdentity,
}

impl ManifestBlockReference {
    pub fn new(
        generation: u64,
        block: u64,
        level: u16,
        checksum: u32,
        first: PersistedRecordIdentity,
        last: PersistedRecordIdentity,
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
    pub const fn first(self) -> PersistedRecordIdentity {
        self.first
    }
    pub const fn last(self) -> PersistedRecordIdentity {
        self.last
    }

    pub fn contains(self, record: PersistedRecordIdentity) -> bool {
        self.first <= record && record <= self.last
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRootRoutingBlock {
    Leaf {
        tree_identity: u64,
        generation: u64,
        block: u64,
        entries: Vec<CurrentPhysicalRecordPlacement>,
    },
    Branch {
        tree_identity: u64,
        generation: u64,
        block: u64,
        level: u16,
        children: Vec<ManifestBlockReference>,
    },
}

impl PhysicalRootRoutingBlock {
    pub fn leaf(
        tree_identity: u64,
        generation: u64,
        block: u64,
        entries: Vec<CurrentPhysicalRecordPlacement>,
        capacity: u16,
    ) -> Option<Self> {
        (tree_identity != 0
            && generation != 0
            && block != 0
            && !entries.is_empty()
            && entries.len() <= usize::from(capacity)
            && entries
                .windows(2)
                .all(|pair| pair[0].record() < pair[1].record())
            && placements_are_unique(&entries))
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
        children: Vec<ManifestBlockReference>,
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
    pub const fn block(&self) -> u64 {
        match self {
            Self::Leaf { block, .. } | Self::Branch { block, .. } => *block,
        }
    }
    pub const fn generation(&self) -> u64 {
        match self {
            Self::Leaf { generation, .. } | Self::Branch { generation, .. } => *generation,
        }
    }
    pub const fn level(&self) -> u16 {
        match self {
            Self::Leaf { .. } => 0,
            Self::Branch { level, .. } => *level,
        }
    }
    pub fn entries(&self) -> Option<&[CurrentPhysicalRecordPlacement]> {
        match self {
            Self::Leaf { entries, .. } => Some(entries),
            Self::Branch { .. } => None,
        }
    }
    pub fn children(&self) -> Option<&[ManifestBlockReference]> {
        match self {
            Self::Branch { children, .. } => Some(children),
            Self::Leaf { .. } => None,
        }
    }

    pub fn reference(&self, checksum: u32) -> ManifestBlockReference {
        let (first, last) = match self {
            Self::Leaf { entries, .. } => (
                entries.first().unwrap().record(),
                entries.last().unwrap().record(),
            ),
            Self::Branch { children, .. } => (
                children.first().unwrap().first(),
                children.last().unwrap().last(),
            ),
        };
        ManifestBlockReference::new(
            self.generation(),
            self.block(),
            self.level(),
            checksum,
            first,
            last,
        )
        .expect("a validated routing block has a valid reference")
    }

    pub fn encode(&self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let (kind, count, entry_bytes) = match self {
            Self::Leaf { entries, .. } => (1_u8, entries.len(), ROUTING_LEAF_ENTRY_BYTES),
            Self::Branch { children, .. } => (2_u8, children.len(), ROUTING_REFERENCE_BYTES),
        };
        let mut payload = vec![0_u8; ROUTING_BLOCK_PREFIX_BYTES + count * entry_bytes];
        payload[..8].copy_from_slice(&self.tree_identity().to_le_bytes());
        payload[8..16].copy_from_slice(&self.block().to_le_bytes());
        payload[16..18].copy_from_slice(&self.level().to_le_bytes());
        payload[18..20].copy_from_slice(&(count as u16).to_le_bytes());
        payload[20] = kind;
        payload[24..32].copy_from_slice(&self.generation().to_le_bytes());
        match self {
            Self::Leaf { entries, .. } => entries.iter().enumerate().for_each(|(index, entry)| {
                encode_entry(
                    &mut payload[ROUTING_BLOCK_PREFIX_BYTES + index * entry_bytes..],
                    *entry,
                );
            }),
            Self::Branch { children, .. } => {
                children.iter().enumerate().for_each(|(index, child)| {
                    encode_reference(
                        &mut payload[ROUTING_BLOCK_PREFIX_BYTES + index * entry_bytes..],
                        *child,
                    );
                })
            }
        }
        encode_durable_frame(
            DurableFrameKind::RootRoutingBlock,
            format,
            self.block(),
            &payload,
        )
    }

    pub fn decode(
        bytes: &[u8],
        capacity: u16,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), RootRoutingBlockDenial> {
        match Self::decode_bounded(
            bytes,
            capacity,
            RootRoutingBlockDecodeLimits {
                leaf_entries: u64::MAX,
                branch_children: u64::MAX,
            },
        ) {
            Ok(decoded) => Ok(decoded),
            Err(BoundedRootRoutingBlockDecodeDenial::Format(denial)) => Err(denial),
            Err(
                BoundedRootRoutingBlockDecodeDenial::LeafEntries { .. }
                | BoundedRootRoutingBlockDecodeDenial::BranchChildren { .. },
            ) => {
                unreachable!("unbounded routing decode cannot exceed its cardinality")
            }
        }
    }

    pub fn decode_bounded(
        bytes: &[u8],
        capacity: u16,
        limits: RootRoutingBlockDecodeLimits,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), BoundedRootRoutingBlockDecodeDenial> {
        let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::RootRoutingBlock)
            .map_err(RootRoutingBlockDenial::Frame)?;
        Self::project_payload(frame.payload, frame.identity, capacity, limits)
            .map(|block| (block, format))
    }
}

fn placements_are_unique(entries: &[CurrentPhysicalRecordPlacement]) -> bool {
    let mut placements = BTreeSet::new();
    entries.iter().all(|entry| {
        placements.insert(match entry {
            CurrentPhysicalRecordPlacement::Inline(value) => (
                1,
                value.segment().get(),
                value.page().get(),
                u64::from(value.slot().get()),
            ),
            CurrentPhysicalRecordPlacement::Extent(value) => (2, 0, value.extent().get(), 0),
        })
    })
}

pub(super) fn encode_reference(target: &mut [u8], reference: ManifestBlockReference) {
    target[..8].copy_from_slice(&reference.generation().to_le_bytes());
    target[8..16].copy_from_slice(&reference.block().to_le_bytes());
    target[16..18].copy_from_slice(&reference.level().to_le_bytes());
    target[20..24].copy_from_slice(&reference.checksum().to_le_bytes());
    encode_identity(&mut target[24..48], reference.first());
    encode_identity(&mut target[48..72], reference.last());
}

pub(super) fn decode_reference(bytes: &[u8]) -> Option<ManifestBlockReference> {
    if bytes[18..20] != [0; 2] {
        return None;
    }
    ManifestBlockReference::new(
        u64::from_le_bytes(bytes[..8].try_into().ok()?),
        u64::from_le_bytes(bytes[8..16].try_into().ok()?),
        u16::from_le_bytes(bytes[16..18].try_into().ok()?),
        u32::from_le_bytes(bytes[20..24].try_into().ok()?),
        decode_identity(&bytes[24..48])?,
        decode_identity(&bytes[48..72])?,
    )
}

pub(super) fn encode_identity(target: &mut [u8], identity: PersistedRecordIdentity) {
    target[..16].copy_from_slice(&identity.allocation_epoch());
    target[16..24].copy_from_slice(&identity.ordinal().to_le_bytes());
}

pub(super) fn decode_identity(bytes: &[u8]) -> Option<PersistedRecordIdentity> {
    PersistedRecordIdentity::new(
        bytes[..16].try_into().ok()?,
        u64::from_le_bytes(bytes[16..24].try_into().ok()?),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootRoutingBlockDenial {
    Frame(DurableFrameDenial),
    MalformedPrefix,
    IdentityOrCapacity,
    LevelOrKind,
    MalformedLength,
    Placement(RootManifestDenial),
    InvalidReference,
    CanonicalOrder,
}
