//! Current-source facts and Head views never grant historical access.
use super::access::{PartitionIdMemory, PartitionIdsDenial};
use crate::identity::data::PartitionId;
use crate::storage::residency::{PartitionHeadProbe, RelationalRegionHeadView};

pub(crate) trait CurrentPartitionAccess: Sync {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>>;
    fn base_current_source(&self) -> Option<&dyn CurrentPartitionAccess> {
        None
    }
    fn base_current_partition(&self, _id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        None
    }
    fn probe_head_partition(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        match self.get_current_partition(id) {
            Some(view) => PartitionHeadProbe::Resident(view),
            None => PartitionHeadProbe::Absent,
        }
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)
    }
    fn has_read_misses(&self) -> bool {
        false
    }

    fn partition_ids(&self) -> Vec<PartitionId>;

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial>;

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.partition_ids().into_iter())
    }

    fn touched_partition_ids(&self) -> Option<Vec<PartitionId>> {
        None
    }

    fn touched_entity_slots(&self, _partition_id: PartitionId) -> Option<Vec<usize>> {
        None
    }

    fn touched_relation_slots(&self, _partition_id: PartitionId) -> Option<Vec<usize>> {
        None
    }

    fn has_touched_entity_slots(&self, partition_id: PartitionId) -> bool {
        self.touched_entity_slots(partition_id).is_some()
    }

    fn has_touched_relation_slots(&self, partition_id: PartitionId) -> bool {
        self.touched_relation_slots(partition_id).is_some()
    }

    fn entity_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.touched_entity_slots(partition_id)
            .is_some_and(|slots| slots.contains(&slot))
    }

    fn relation_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.touched_relation_slots(partition_id)
            .is_some_and(|slots| slots.contains(&slot))
    }
}
