//! Current records can be inspected without lending their historical columns.
use crate::identity::data::{RecordId, VersionId};
use crate::storage::data::RelationalFieldRevision;
use crate::storage::substrate::{RecordArena, RecordKind, SlotView};
use crate::symbols::data::Symbol;
use std::collections::BTreeMap;

pub(crate) struct RelationalRecordHeadArenaView<'lease, K: RecordKind> {
    arena: &'lease RecordArena<K>,
}

impl<K: RecordKind> Copy for RelationalRecordHeadArenaView<'_, K> {}
impl<K: RecordKind> Clone for RelationalRecordHeadArenaView<'_, K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'lease, K: RecordKind> RelationalRecordHeadArenaView<'lease, K> {
    pub(super) fn new(arena: &'lease RecordArena<K>) -> Self {
        Self { arena }
    }

    pub(crate) fn get(&self, id: &RecordId<K::Domain>) -> Option<SlotView<'lease, K>> {
        self.arena.get(id)
    }

    pub(crate) fn extra_at(&self, slot: usize) -> Option<&'lease K::Extra> {
        self.arena.extra_at(slot)
    }

    pub(crate) fn contains_live_id(&self, id: &RecordId<K::Domain>) -> bool {
        self.arena.contains_live_id(id)
    }
    pub(crate) fn aspect_versions_at(&self, slot: usize) -> Option<&'lease BTreeMap<Symbol, u64>> {
        self.arena.aspect_versions_at(slot)
    }
    pub(crate) fn field_revisions_at(
        &self,
        slot: usize,
    ) -> Option<&'lease BTreeMap<(Symbol, Symbol), RelationalFieldRevision>> {
        self.arena.field_revisions_at(slot)
    }

    pub(crate) fn created_at_for_slot(&self, slot: usize) -> Option<VersionId> {
        self.arena.created_at_for_slot(slot)
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
