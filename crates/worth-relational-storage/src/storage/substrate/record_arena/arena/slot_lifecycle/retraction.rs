//! Retract one sparse overlay slot of an unpublished candidate (#119).
//!
//! A coupled successor round supersedes its chain's earlier rounds by
//! returning every record they changed to its image before the chain's head
//! (`B`), then applying the new round as written. Each slot either takes
//! `B`'s exact stored row, or leaves the overlay so the base shows through.
use super::super::{RecordArena, RecordKind};

impl<K: RecordKind> RecordArena<K> {
    /// Withdraw `slot` from this overlay, so reads fall through to the base
    /// partition: a chain-fresh slot becomes absent (invisible to occupancy,
    /// `get` and `live_bitset`), a committed one reads its basis image. A
    /// later write of the same slot starts a fresh row with a one-entry
    /// metadata history. Returns whether the overlay held the slot. O(log n).
    pub fn retract_uncommitted_slot_for_request(&mut self, slot: usize) -> bool {
        if self.slots.remove(slot as u64).is_none() {
            return false;
        }
        self.live_bitset.set(slot, false);
        self.reclaimable_bitset.set(slot, false);
        true
    }

    /// Restore `slot` to `base`'s exact row: every truth column, the live and
    /// reclaimable bits. `base` must hold the slot. O(log n + row payload).
    pub fn restore_slot_from_base_for_request(&mut self, base: &Self, slot: usize) -> bool {
        if base.physical_index(slot).is_none() {
            return false;
        }
        self.move_slot_from_overlay(base, slot);
        true
    }
}
