//! Current records can be inspected without lending their historical columns.
use crate::identity::data::RecordId;
use crate::storage::substrate::{RecordArena, RecordKind, SlotView};

pub(crate) struct RelationalRecordHeadArenaView<'lease, K: RecordKind> {
    arena: &'lease RecordArena<K>,
}

impl<'lease, K: RecordKind> RelationalRecordHeadArenaView<'lease, K> {
    pub(super) fn new(arena: &'lease RecordArena<K>) -> Self {
        Self { arena }
    }

    pub(crate) fn get(&self, id: &RecordId<K::Domain>) -> Option<SlotView<'lease, K>> {
        self.arena.get(id)
    }

    pub(crate) fn get_slot(&self, slot: usize) -> Option<SlotView<'lease, K>> {
        self.arena.get_slot(slot)
    }

    pub(crate) fn slot_count(&self) -> usize {
        self.arena.slot_count()
    }

    pub(crate) fn live_slots(&self) -> impl Iterator<Item = usize> + 'lease {
        self.arena.live_bitset.set_slots()
    }

    pub(crate) fn live_count(&self) -> usize {
        self.arena.live_bitset.count_ones()
    }

    pub(crate) fn occupied_slots_iter(&self) -> impl Iterator<Item = usize> + 'lease {
        self.arena.occupied_slots_iter()
    }

    pub(crate) fn current_metadata_at(&self, slot: usize) -> Option<&'lease K::Meta> {
        let resident = self.arena.metadata_history_at(slot)?;
        resident
            .len()
            .checked_sub(1)
            .and_then(|last| resident.get(last))
    }
}
