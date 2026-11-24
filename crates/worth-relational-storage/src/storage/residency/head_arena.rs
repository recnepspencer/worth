//! Current records can be inspected without lending their historical columns.
use crate::identity::data::{RecordId, VersionId};
use crate::storage::substrate::{
    AspectVersions, FieldRevisions, RecordArena, RecordKind, SlotView,
};

pub struct RelationalRecordHeadArenaView<'lease, K: RecordKind> {
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

    pub fn get(&self, id: &RecordId<K::Domain>) -> Option<SlotView<'lease, K>> {
        self.arena.get(id)
    }

    pub fn extra_at(&self, slot: usize) -> Option<&'lease K::Extra> {
        self.arena.extra_at(slot)
    }

    pub fn contains_live_id(&self, id: &RecordId<K::Domain>) -> bool {
        self.arena.contains_live_id(id)
    }
    pub fn aspect_versions_at(&self, slot: usize) -> Option<&'lease AspectVersions> {
        self.arena.aspect_versions_at(slot)
    }
    pub fn field_revisions_at(&self, slot: usize) -> Option<&'lease FieldRevisions> {
        self.arena.field_revisions_at(slot)
    }

    pub fn created_at_for_slot(&self, slot: usize) -> Option<VersionId> {
        self.arena.created_at_for_slot(slot)
    }

    pub fn get_slot(&self, slot: usize) -> Option<SlotView<'lease, K>> {
        self.arena.get_slot(slot)
    }

    pub fn slot_count(&self) -> usize {
        self.arena.slot_count()
    }

    pub fn live_slots(&self) -> impl Iterator<Item = usize> + 'lease {
        self.arena.live_bitset.set_slots()
    }

    pub fn try_live_slots<Stop>(
        &self,
        prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<crate::storage::partition::DenseSlotIterator<'lease>, Stop> {
        self.arena.live_bitset.try_set_slots(prepare)
    }

    pub fn live_count(&self) -> usize {
        self.arena.live_bitset.count_ones()
    }

    pub fn try_visit_occupied<Stop>(
        &self,
        visit: impl FnMut(crate::storage::substrate::RecordSlotVisit<'_, K>) -> Result<(), Stop>,
    ) -> Result<(), Stop> {
        self.arena.try_visit_occupied(visit)
    }

    pub fn try_occupied_slots<Stop>(
        &self,
        prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        std::iter::Map<
            crate::storage::substrate::SharedMapIter<'lease, u64, usize>,
            fn((&u64, &usize)) -> usize,
        >,
        Stop,
    > {
        self.arena.try_occupied_slots(prepare)
    }

    pub fn occupied_slots_iter(&self) -> impl Iterator<Item = usize> + 'lease + use<'lease, K> {
        self.arena.occupied_slots_iter()
    }

    pub fn current_metadata_at(&self, slot: usize) -> Option<&'lease K::Meta> {
        let resident = self.arena.metadata_history_at(slot)?;
        resident
            .len()
            .checked_sub(1)
            .and_then(|last| resident.get(last))
    }
}
