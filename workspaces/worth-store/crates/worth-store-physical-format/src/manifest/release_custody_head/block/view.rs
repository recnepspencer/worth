use sha2::{Digest, Sha256};

use crate::record_framing::decode_durable_frame;
use crate::{DurableFrameKind, PhysicalRecordFormatDeclaration};

use super::super::entry::{ReleaseCustodyHeadEntryV1, ENTRY_BYTES};
use super::super::ReleaseCustodyHeadDenial;
use super::{
    max_entries, ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadBlockV1, PREFIX_BYTES,
    REFERENCE_BYTES, VERSION,
};

#[derive(Debug, Clone, Copy)]
enum BlockItems<'a> {
    Entries(&'a [u8]),
    Children(&'a [u8]),
}

/// Borrowed, fully grammar-validated node. No decoded entry vector or
/// canonical re-encoding is needed for a rooted read.
#[derive(Debug, Clone, Copy)]
pub struct ReleaseCustodyHeadBlockViewV1<'a> {
    tree_identity: u64,
    generation: u64,
    block: u64,
    level: u16,
    count: usize,
    items: BlockItems<'a>,
}

impl<'a> ReleaseCustodyHeadBlockViewV1<'a> {
    pub fn decode(
        bytes: &'a [u8],
        expected: ReleaseCustodyHeadBlockReferenceV1,
        expected_tree: u64,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), ReleaseCustodyHeadDenial> {
        let (format, frame) =
            decode_durable_frame(bytes, DurableFrameKind::ReleaseCustodyHeadBlock)
                .map_err(ReleaseCustodyHeadDenial::Frame)?;
        let payload = frame.payload;
        if payload.len() < PREFIX_BYTES {
            return Err(ReleaseCustodyHeadDenial::Malformed);
        }
        if payload[0] != VERSION {
            return Err(ReleaseCustodyHeadDenial::UnsupportedVersion);
        }
        if payload[6..8] != [0; 2] || payload[32..40] != [0; 8] {
            return Err(ReleaseCustodyHeadDenial::Malformed);
        }
        let count = u16::from_le_bytes(payload[2..4].try_into().unwrap()) as usize;
        let level = u16::from_le_bytes(payload[4..6].try_into().unwrap());
        let tree_identity = u64::from_le_bytes(payload[8..16].try_into().unwrap());
        let generation = u64::from_le_bytes(payload[16..24].try_into().unwrap());
        let block = u64::from_le_bytes(payload[24..32].try_into().unwrap());
        let width = match payload[1] {
            1 if level == 0 => ENTRY_BYTES,
            2 if level > 0 => REFERENCE_BYTES,
            _ => return Err(ReleaseCustodyHeadDenial::Malformed),
        };
        if count == 0
            || count > max_entries(format, width)
            || payload.len() != PREFIX_BYTES + count * width
        {
            return Err(ReleaseCustodyHeadDenial::Capacity);
        }
        if tree_identity == 0
            || generation == 0
            || block == 0
            || frame.identity != block
            || tree_identity != expected_tree
            || generation != expected.generation()
            || block != expected.block()
            || level != expected.level()
        {
            return Err(ReleaseCustodyHeadDenial::Identity);
        }
        if Sha256::digest(bytes).as_slice() != expected.frame_sha256() {
            return Err(ReleaseCustodyHeadDenial::Digest);
        }
        let data = &payload[PREFIX_BYTES..];
        let (items, first, last) = if level == 0 {
            let mut previous = None;
            for part in data.chunks_exact(ENTRY_BYTES) {
                let entry = ReleaseCustodyHeadEntryV1::decode(part)?;
                if previous.is_some_and(|key| key >= entry.key()) {
                    return Err(ReleaseCustodyHeadDenial::CanonicalOrder);
                }
                previous = Some(entry.key());
            }
            let first = ReleaseCustodyHeadEntryV1::decode(&data[..ENTRY_BYTES])?.key();
            (BlockItems::Entries(data), first, previous.unwrap())
        } else {
            let mut previous = None;
            for part in data.chunks_exact(REFERENCE_BYTES) {
                let child = ReleaseCustodyHeadBlockReferenceV1::decode(part)?;
                if child.level().checked_add(1) != Some(level)
                    || child.generation() > generation
                    || previous.is_some_and(|last| last >= child.first())
                {
                    return Err(ReleaseCustodyHeadDenial::CanonicalOrder);
                }
                previous = Some(child.last());
            }
            let first =
                ReleaseCustodyHeadBlockReferenceV1::decode(&data[..REFERENCE_BYTES])?.first();
            (BlockItems::Children(data), first, previous.unwrap())
        };
        // All frame, prefix, entry, and reference fields have one accepted
        // encoding. Exact SHA of those validated bytes is therefore the SHA
        // of the typed canonical re-encoding used by the owned reference().
        if first != expected.first() || last != expected.last() {
            return Err(ReleaseCustodyHeadDenial::Reference);
        }
        Ok((
            Self {
                tree_identity,
                generation,
                block,
                level,
                count,
                items,
            },
            format,
        ))
    }

    pub const fn count(self) -> usize {
        self.count
    }
    pub const fn level(self) -> u16 {
        self.level
    }

    pub fn entries(
        &self,
    ) -> Option<impl DoubleEndedIterator<Item = ReleaseCustodyHeadEntryV1> + ExactSizeIterator + '_>
    {
        match self.items {
            BlockItems::Entries(bytes) => Some(bytes.chunks_exact(ENTRY_BYTES).map(|part| {
                ReleaseCustodyHeadEntryV1::decode(part).expect("validated head entry")
            })),
            BlockItems::Children(_) => None,
        }
    }

    pub fn children(
        &self,
    ) -> Option<
        impl DoubleEndedIterator<Item = ReleaseCustodyHeadBlockReferenceV1> + ExactSizeIterator + '_,
    > {
        match self.items {
            BlockItems::Children(bytes) => Some(bytes.chunks_exact(REFERENCE_BYTES).map(|part| {
                ReleaseCustodyHeadBlockReferenceV1::decode(part).expect("validated child reference")
            })),
            BlockItems::Entries(_) => None,
        }
    }

    pub(super) fn into_owned(
        self,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadDenial> {
        match self.items {
            BlockItems::Entries(_) => ReleaseCustodyHeadBlockV1::leaf(
                self.tree_identity,
                self.generation,
                self.block,
                self.entries().unwrap().collect(),
                format,
            ),
            BlockItems::Children(_) => ReleaseCustodyHeadBlockV1::branch(
                self.tree_identity,
                self.generation,
                self.block,
                self.level,
                self.children().unwrap().collect(),
                format,
            ),
        }
    }
}
