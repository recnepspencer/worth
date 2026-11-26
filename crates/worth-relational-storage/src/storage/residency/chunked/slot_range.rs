//! Fixed, non-overlapping slot partitions of one slot domain.
use serde::{Deserialize, Serialize};

/// Which slot space of a placement group a chunk partitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SlotDomain {
    Entity,
    Relation,
}

impl SlotDomain {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::Entity => 0,
            Self::Relation => 1,
        }
    }
}

/// A non-empty half-open slot range `[start_slot_inclusive, end_slot_exclusive)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "EncodedSlotRange", into = "EncodedSlotRange")]
pub struct SlotRange {
    start_slot_inclusive: u64,
    end_slot_exclusive: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotRangeError {
    /// `start_slot_inclusive >= end_slot_exclusive`.
    Empty { start: u64, end: u64 },
}

/// `serde`'s `try_from` decoding reports refusals through `Display`.
impl std::fmt::Display for SlotRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self::Empty { start, end } = self;
        write!(f, "slot range [{start}, {end}) is empty")
    }
}

impl SlotRange {
    pub const fn new(
        start_slot_inclusive: u64,
        end_slot_exclusive: u64,
    ) -> Result<Self, SlotRangeError> {
        if start_slot_inclusive < end_slot_exclusive {
            Ok(Self {
                start_slot_inclusive,
                end_slot_exclusive,
            })
        } else {
            Err(SlotRangeError::Empty {
                start: start_slot_inclusive,
                end: end_slot_exclusive,
            })
        }
    }

    pub const fn start_slot_inclusive(self) -> u64 {
        self.start_slot_inclusive
    }

    pub const fn end_slot_exclusive(self) -> u64 {
        self.end_slot_exclusive
    }

    pub const fn contains(self, slot: u64) -> bool {
        self.start_slot_inclusive <= slot && slot < self.end_slot_exclusive
    }

    /// The fixed chunk of a `total_slots` domain that holds `slot`.
    pub fn fixed_chunk_of(slot: u64, total_slots: u64, chunk_slots: u64) -> Option<Self> {
        if slot >= total_slots || chunk_slots == 0 {
            return None;
        }
        let start = slot - slot % chunk_slots;
        Self::new(start, start.saturating_add(chunk_slots).min(total_slots)).ok()
    }

    /// Whether this range is exactly a fixed chunk of a `total_slots` domain.
    pub fn is_fixed_chunk(self, total_slots: u64, chunk_slots: u64) -> bool {
        Self::fixed_chunk_of(self.start_slot_inclusive, total_slots, chunk_slots) == Some(self)
    }

    pub(super) fn commit_bytes(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(&self.start_slot_inclusive.to_be_bytes());
        bytes[8..].copy_from_slice(&self.end_slot_exclusive.to_be_bytes());
        bytes
    }
}

/// The ascending fixed partition of `[0, total_slots)`. A domain no larger
/// than one chunk is one chunk; an empty domain or `chunk_slots == 0` has none.
pub fn fixed_partition(total_slots: u64, chunk_slots: u64) -> impl Iterator<Item = SlotRange> {
    let chunks = if chunk_slots == 0 {
        0
    } else {
        total_slots.div_ceil(chunk_slots)
    };
    (0..chunks).filter_map(move |index| {
        SlotRange::fixed_chunk_of(index * chunk_slots, total_slots, chunk_slots)
    })
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EncodedSlotRange {
    start_slot_inclusive: u64,
    end_slot_exclusive: u64,
}

impl TryFrom<EncodedSlotRange> for SlotRange {
    type Error = SlotRangeError;

    fn try_from(encoded: EncodedSlotRange) -> Result<Self, Self::Error> {
        Self::new(encoded.start_slot_inclusive, encoded.end_slot_exclusive)
    }
}

impl From<SlotRange> for EncodedSlotRange {
    fn from(range: SlotRange) -> Self {
        Self {
            start_slot_inclusive: range.start_slot_inclusive,
            end_slot_exclusive: range.end_slot_exclusive,
        }
    }
}
