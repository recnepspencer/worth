use sha2::{Digest, Sha256};

use crate::record_framing::{encode_durable_frame_schema, DURABLE_FRAME_HEADER_BYTES};
use crate::{DurableFrameKind, PhysicalRecordFormatDeclaration};

use super::entry::{
    decode_key, encode_key, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, ENTRY_BYTES,
};
use super::ReleaseCustodyHeadDenial;

#[path = "block/view.rs"]
mod view;
pub use view::ReleaseCustodyHeadBlockViewV1;

const VERSION: u8 = 1;
const PREFIX_BYTES: usize = 40;
pub(super) const REFERENCE_BYTES: usize = 104;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCustodyHeadBlockReferenceV1 {
    generation: u64,
    block: u64,
    level: u16,
    first: ReleaseCustodyHeadKeyV1,
    last: ReleaseCustodyHeadKeyV1,
    frame_sha256: [u8; 32],
}

impl ReleaseCustodyHeadBlockReferenceV1 {
    pub const ENCODED_BYTES: usize = REFERENCE_BYTES;
    pub fn new(
        generation: u64,
        block: u64,
        level: u16,
        first: ReleaseCustodyHeadKeyV1,
        last: ReleaseCustodyHeadKeyV1,
        frame_sha256: [u8; 32],
    ) -> Option<Self> {
        (generation != 0 && block != 0 && first <= last && frame_sha256 != [0; 32]).then_some(
            Self {
                generation,
                block,
                level,
                first,
                last,
                frame_sha256,
            },
        )
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
    pub const fn first(self) -> ReleaseCustodyHeadKeyV1 {
        self.first
    }
    pub const fn last(self) -> ReleaseCustodyHeadKeyV1 {
        self.last
    }
    pub const fn frame_sha256(self) -> [u8; 32] {
        self.frame_sha256
    }

    pub fn encode_into(self, target: &mut [u8; REFERENCE_BYTES]) {
        *target = [0; REFERENCE_BYTES];
        target[..8].copy_from_slice(&self.generation.to_le_bytes());
        target[8..16].copy_from_slice(&self.block.to_le_bytes());
        target[16..18].copy_from_slice(&self.level.to_le_bytes());
        encode_key(self.first, (&mut target[24..48]).try_into().unwrap());
        encode_key(self.last, (&mut target[48..72]).try_into().unwrap());
        target[72..104].copy_from_slice(&self.frame_sha256);
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ReleaseCustodyHeadDenial> {
        if bytes.len() != REFERENCE_BYTES || bytes[18..24] != [0; 6] {
            return Err(ReleaseCustodyHeadDenial::Malformed);
        }
        Self::new(
            u64::from_le_bytes(bytes[..8].try_into().unwrap()),
            u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            u16::from_le_bytes(bytes[16..18].try_into().unwrap()),
            decode_key(&bytes[24..48])?,
            decode_key(&bytes[48..72])?,
            bytes[72..104].try_into().unwrap(),
        )
        .ok_or(ReleaseCustodyHeadDenial::Reference)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseCustodyHeadBlockV1 {
    Leaf {
        tree_identity: u64,
        generation: u64,
        block: u64,
        entries: Vec<ReleaseCustodyHeadEntryV1>,
    },
    Branch {
        tree_identity: u64,
        generation: u64,
        block: u64,
        level: u16,
        children: Vec<ReleaseCustodyHeadBlockReferenceV1>,
    },
}

impl ReleaseCustodyHeadBlockV1 {
    pub fn leaf(
        tree_identity: u64,
        generation: u64,
        block: u64,
        entries: Vec<ReleaseCustodyHeadEntryV1>,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<Self, ReleaseCustodyHeadDenial> {
        if tree_identity == 0
            || generation == 0
            || block == 0
            || entries.is_empty()
            || entries.len() > max_entries(format, ENTRY_BYTES)
            || !entries.windows(2).all(|pair| pair[0].key() < pair[1].key())
        {
            return Err(ReleaseCustodyHeadDenial::CanonicalOrder);
        }
        Ok(Self::Leaf {
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
        children: Vec<ReleaseCustodyHeadBlockReferenceV1>,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<Self, ReleaseCustodyHeadDenial> {
        if tree_identity == 0
            || generation == 0
            || block == 0
            || level == 0
            || children.is_empty()
            || children.len() > max_entries(format, REFERENCE_BYTES)
            || !children.iter().all(|child| {
                child.level.checked_add(1) == Some(level) && child.generation <= generation
            })
            || !children.windows(2).all(|pair| pair[0].last < pair[1].first)
        {
            return Err(ReleaseCustodyHeadDenial::CanonicalOrder);
        }
        Ok(Self::Branch {
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
    pub fn entries(&self) -> Option<&[ReleaseCustodyHeadEntryV1]> {
        match self {
            Self::Leaf { entries, .. } => Some(entries),
            _ => None,
        }
    }
    pub fn children(&self) -> Option<&[ReleaseCustodyHeadBlockReferenceV1]> {
        match self {
            Self::Branch { children, .. } => Some(children),
            _ => None,
        }
    }

    pub fn reference(
        &self,
        format: PhysicalRecordFormatDeclaration,
    ) -> ReleaseCustodyHeadBlockReferenceV1 {
        let (first, last) = match self {
            Self::Leaf { entries, .. } => (
                entries.first().unwrap().key(),
                entries.last().unwrap().key(),
            ),
            Self::Branch { children, .. } => (
                children.first().unwrap().first(),
                children.last().unwrap().last(),
            ),
        };
        ReleaseCustodyHeadBlockReferenceV1::new(
            self.generation(),
            self.block(),
            self.level(),
            first,
            last,
            Sha256::digest(self.encode(format)).into(),
        )
        .unwrap()
    }

    pub fn encode(&self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let (kind, count, width) = match self {
            Self::Leaf { entries, .. } => (1_u8, entries.len(), ENTRY_BYTES),
            Self::Branch { children, .. } => (2_u8, children.len(), REFERENCE_BYTES),
        };
        let mut payload = vec![0_u8; PREFIX_BYTES + count * width];
        payload[0] = VERSION;
        payload[1] = kind;
        payload[2..4].copy_from_slice(&(count as u16).to_le_bytes());
        payload[4..6].copy_from_slice(&self.level().to_le_bytes());
        payload[8..16].copy_from_slice(&self.tree_identity().to_le_bytes());
        payload[16..24].copy_from_slice(&self.generation().to_le_bytes());
        payload[24..32].copy_from_slice(&self.block().to_le_bytes());
        match self {
            Self::Leaf { entries, .. } => {
                for (index, entry) in entries.iter().enumerate() {
                    entry.encode_into(
                        (&mut payload
                            [PREFIX_BYTES + index * width..PREFIX_BYTES + (index + 1) * width])
                            .try_into()
                            .unwrap(),
                    );
                }
            }
            Self::Branch { children, .. } => {
                for (index, child) in children.iter().enumerate() {
                    child.encode_into(
                        (&mut payload
                            [PREFIX_BYTES + index * width..PREFIX_BYTES + (index + 1) * width])
                            .try_into()
                            .unwrap(),
                    );
                }
            }
        }
        encode_durable_frame_schema(
            DurableFrameKind::ReleaseCustodyHeadBlock,
            format,
            self.block(),
            &payload,
            crate::record_framing::FRAME_SCHEMA,
        )
    }

    pub fn decode(
        bytes: &[u8],
        expected: ReleaseCustodyHeadBlockReferenceV1,
        expected_tree: u64,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), ReleaseCustodyHeadDenial> {
        let (view, format) = ReleaseCustodyHeadBlockViewV1::decode(bytes, expected, expected_tree)?;
        Ok((view.into_owned(format)?, format))
    }
}

pub(super) fn max_entries(format: PhysicalRecordFormatDeclaration, width: usize) -> usize {
    (format.page_size().bytes() as usize - DURABLE_FRAME_HEADER_BYTES - PREFIX_BYTES) / width
}
