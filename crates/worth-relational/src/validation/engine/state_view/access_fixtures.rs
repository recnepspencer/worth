//! Access fixtures expose accidental world and touched-slot enumeration.
use super::*;

pub(super) struct NoWorldPartitionEnumeration<'a> {
    pub(super) state: &'a OverlayStateView<'a, WorkingState>,
}

impl crate::storage::overlay::CurrentPartitionAccess for NoWorldPartitionEnumeration<'_> {
    fn get_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.state.get_current_partition(id)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        panic!("touched-scope discovery must not enumerate all partitions")
    }

    fn partition_ids_with_memory(
        &self,
        _: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        panic!("touched-scope discovery must not enumerate all partitions")
    }

    fn touched_partition_ids(&self) -> Option<Vec<PartitionId>> {
        self.state.touched_partition_ids()
    }

    fn touched_entity_slots(&self, partition_id: PartitionId) -> Option<Vec<usize>> {
        self.state.touched_entity_slots(partition_id)
    }
}

pub(super) struct NoSlotEnumerationForRead<'a> {
    pub(super) state: &'a OverlayStateView<'a, WorkingState>,
    pub(super) entity_slot_checks: AtomicUsize,
    pub(super) entity_presence_checks: AtomicUsize,
    pub(super) relation_slot_checks: AtomicUsize,
    pub(super) relation_presence_checks: AtomicUsize,
}

impl<'a> NoSlotEnumerationForRead<'a> {
    pub(super) fn new(state: &'a OverlayStateView<'a, WorkingState>) -> Self {
        Self {
            state,
            entity_slot_checks: AtomicUsize::new(0),
            entity_presence_checks: AtomicUsize::new(0),
            relation_slot_checks: AtomicUsize::new(0),
            relation_presence_checks: AtomicUsize::new(0),
        }
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for NoSlotEnumerationForRead<'_> {
    fn get_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.state.get_current_partition(id)
    }

    fn base_current_source(&self) -> Option<&dyn crate::storage::overlay::CurrentPartitionAccess> {
        self.state.base_current_source()
    }
    fn base_current_partition(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.state.base_current_partition(id)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        panic!("aspect reads must not enumerate partitions")
    }

    fn partition_ids_with_memory(
        &self,
        _: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        panic!("aspect reads must not enumerate partitions")
    }

    fn touched_entity_slots(&self, _: PartitionId) -> Option<Vec<usize>> {
        panic!("aspect reads must not allocate touched entity slots")
    }

    fn touched_relation_slots(&self, _: PartitionId) -> Option<Vec<usize>> {
        panic!("aspect reads must not allocate touched relation slots")
    }

    fn has_touched_entity_slots(&self, partition_id: PartitionId) -> bool {
        self.entity_presence_checks.fetch_add(1, Ordering::Relaxed);
        self.state.has_touched_entity_slots(partition_id)
    }

    fn has_touched_relation_slots(&self, partition_id: PartitionId) -> bool {
        self.relation_presence_checks
            .fetch_add(1, Ordering::Relaxed);
        self.state.has_touched_relation_slots(partition_id)
    }

    fn entity_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.entity_slot_checks.fetch_add(1, Ordering::Relaxed);
        self.state.entity_slot_is_touched(partition_id, slot)
    }

    fn relation_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.relation_slot_checks.fetch_add(1, Ordering::Relaxed);
        self.state.relation_slot_is_touched(partition_id, slot)
    }
}
