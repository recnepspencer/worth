use super::root::{RelationalBranchRoot, RelationalBranchRootState};
use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};

impl PartitionAccess for RelationalBranchRootState {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.root.partition_state(partition_id)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.root.partition_ids()
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.root.partition_ids_iter())
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        use crate::storage::overlay::collect_partition_ids;
        collect_partition_ids(
            self.root.region_count(),
            self.root.partition_ids_iter(),
            memory,
        )
    }
}

impl PartitionAccess for RelationalBranchRoot {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.partition_state(partition_id)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.partition_ids()
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(RelationalBranchRoot::partition_ids_iter(self))
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        use crate::storage::overlay::collect_partition_ids;
        collect_partition_ids(
            self.region_count(),
            RelationalBranchRoot::partition_ids_iter(self),
            memory,
        )
    }
}
