//! Physical head and history images of one chunk.
//!
//! The head image lists every occupied slot of the chunk with its head
//! version and inline adjacency tail. The history image carries inline
//! history tails in one or more layers: a slot's inline history is its rows
//! concatenated in layer order. Copy-on-write edits add layers; consolidation
//! flattens them into one. Both encodings decode to the same canonical meaning.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::member_frame::{decode_member, encode_member, ChunkMemberKind, EncodedChunkMember};
use super::page_cache::ResidentPage;
use super::slot_projection::{AdjacencyEdge, SlotVersion};
use super::slot_range::{SlotDomain, SlotRange};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkHeadImage {
    pub domain: SlotDomain,
    pub slot_range: SlotRange,
    /// Occupied slots, strictly ascending.
    pub slots: Vec<HeadSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeadSlot {
    pub slot: u64,
    pub head: Option<SlotVersion>,
    /// The adjacency tail kept inline; its spilled prefix is in continuation pages.
    pub adjacency: Vec<AdjacencyEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkHistoryImage {
    pub domain: SlotDomain,
    pub slot_range: SlotRange,
    /// Oldest layer first.
    pub layers: Vec<HistoryLayer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryLayer {
    /// Rows with strictly ascending slots.
    pub rows: Vec<HistoryRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRow {
    pub slot: u64,
    /// Oldest to newest within this layer.
    pub versions: Vec<SlotVersion>,
}

impl ChunkHeadImage {
    pub fn encode(&self, limit: usize) -> Result<EncodedChunkMember, Cause> {
        encode_member(ChunkMemberKind::Head, self, limit)
    }

    /// Admits a decoded image as the head of the bound `domain` chunk.
    pub fn admit_for(&self, domain: SlotDomain, slot_range: SlotRange) -> Result<(), Cause> {
        let slots_canonical = ascending_within(self.slots.iter().map(|slot| slot.slot), slot_range);
        if self.domain != domain || self.slot_range != slot_range || !slots_canonical {
            return Err(Cause::Damaged);
        }
        Ok(())
    }
}

impl ChunkHistoryImage {
    pub fn encode(&self, limit: usize) -> Result<EncodedChunkMember, Cause> {
        encode_member(ChunkMemberKind::History, self, limit)
    }

    /// Admits a decoded image as the history of `head`'s chunk: every row
    /// names an occupied head slot.
    pub fn admit_for(&self, head: &ChunkHeadImage) -> Result<(), Cause> {
        let image = self;
        let occupied = |slot: u64| {
            head.slots
                .binary_search_by_key(&slot, |occupied| occupied.slot)
                .is_ok()
        };
        let rows_canonical = image.layers.iter().all(|layer| {
            ascending_within(layer.rows.iter().map(|row| row.slot), head.slot_range)
                && layer.rows.iter().all(|row| occupied(row.slot))
        });
        if image.domain != head.domain || image.slot_range != head.slot_range || !rows_canonical {
            return Err(Cause::Damaged);
        }
        Ok(())
    }

    /// The single-layer image with the same canonical meaning.
    pub fn flattened(&self) -> Self {
        let mut merged = BTreeMap::<u64, Vec<SlotVersion>>::new();
        for row in self.layers.iter().flat_map(|layer| &layer.rows) {
            merged
                .entry(row.slot)
                .or_default()
                .extend(row.versions.iter().cloned());
        }
        let rows = merged
            .into_iter()
            .filter(|(_, versions)| !versions.is_empty())
            .map(|(slot, versions)| HistoryRow { slot, versions })
            .collect();
        Self {
            domain: self.domain,
            slot_range: self.slot_range,
            layers: vec![HistoryLayer { rows }],
        }
    }

    /// One slot's inline history across all layers, oldest first.
    pub fn slot_versions(&self, slot: u64) -> impl Iterator<Item = &SlotVersion> {
        self.layers.iter().flat_map(move |layer| {
            layer
                .rows
                .binary_search_by_key(&slot, |row| row.slot)
                .ok()
                .map(|index| layer.rows[index].versions.iter())
                .into_iter()
                .flatten()
        })
    }
}

impl ResidentPage for ChunkHeadImage {
    fn decode(bytes: &[u8], locator: RelationalDurableMemberLocator) -> Result<Self, Cause> {
        decode_member(ChunkMemberKind::Head, bytes, locator)
    }

    fn resident_bytes(&self) -> u64 {
        let slots = self.slots.iter().map(|slot| {
            slot.head.as_ref().map_or(0, |head| head.payload.capacity())
                + slot.adjacency.capacity() * std::mem::size_of::<AdjacencyEdge>()
        });
        (std::mem::size_of::<Self>()
            + self.slots.capacity() * std::mem::size_of::<HeadSlot>()
            + slots.sum::<usize>()) as u64
    }
}

impl ResidentPage for ChunkHistoryImage {
    fn decode(bytes: &[u8], locator: RelationalDurableMemberLocator) -> Result<Self, Cause> {
        decode_member(ChunkMemberKind::History, bytes, locator)
    }

    fn resident_bytes(&self) -> u64 {
        let rows = self.layers.iter().flat_map(|layer| &layer.rows).map(|row| {
            std::mem::size_of::<HistoryRow>()
                + row.versions.capacity() * std::mem::size_of::<SlotVersion>()
                + row
                    .versions
                    .iter()
                    .map(|version| version.payload.capacity())
                    .sum::<usize>()
        });
        (std::mem::size_of::<Self>()
            + self.layers.capacity() * std::mem::size_of::<HistoryLayer>()
            + rows.sum::<usize>()) as u64
    }
}

fn ascending_within(slots: impl Iterator<Item = u64>, range: SlotRange) -> bool {
    let mut previous = None;
    for slot in slots {
        if !range.contains(slot) || previous.is_some_and(|previous| previous >= slot) {
            return false;
        }
        previous = Some(slot);
    }
    true
}
