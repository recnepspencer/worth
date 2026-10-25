use super::*;

impl RelationalBranchRoot {
    pub(crate) fn partition_state(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.regions
            .get(partition_id)
            .map(|region| region.partition.as_ref())
    }

    pub(crate) fn visit_content_cache_allocations(
        &self,
        partition_id: PartitionId,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        if let Some(region) = self.regions.get(partition_id) {
            region
                .partition
                .full_view()
                .commitment()
                .visit_allocations(visitor);
        }
    }

    pub(crate) fn partition_ids(&self) -> Vec<PartitionId> {
        self.regions.materialize().keys().copied().collect()
    }

    pub(crate) fn partition_ids_iter(&self) -> impl Iterator<Item = PartitionId> + '_ {
        self.regions.partition_ids_iter()
    }

    pub(crate) fn try_visit_partitions<E>(
        &self,
        visit: impl FnMut(crate::branch::RelationalPartitionVisit) -> Result<(), E>,
    ) -> Result<(), E> {
        self.regions.try_visit_partitions(visit)
    }

    pub(crate) fn try_for_each_partition_id<E>(
        &self,
        visit: impl FnMut(PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.regions.try_for_each_partition_id(visit)
    }

    pub(crate) fn checkpoint_regions(
        &self,
    ) -> Vec<super::super::root_checkpoint::RelationalRootRegionCheckpoint> {
        self.regions
            .values()
            .map(
                |region| super::super::root_checkpoint::RelationalRootRegionCheckpoint {
                    partition_id: region.partition_id,
                    content_digest: region.content_digest,
                    entity_slots: region.entity_slots,
                    relation_slots: region.relation_slots,
                    unavailable_records: region.materialization_unavailable_records(),
                    cell: region.partition.cell().clone(),
                },
            )
            .collect()
    }
}
