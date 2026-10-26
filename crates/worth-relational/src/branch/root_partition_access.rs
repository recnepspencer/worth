use super::root::{RelationalBranchRoot, RelationalBranchRootState};
use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};

impl PartitionAccess for RelationalBranchRootState {
    fn probe_full_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        self.root.probe_full_partition(id)
    }

    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.root.demand_full_partition(execution, id)
    }
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.root.partition_state(partition_id)
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for RelationalBranchRootState {
    fn get_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.root.get_current_partition(id)
    }
    fn probe_head_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        self.root.probe_head_partition(id)
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: crate::identity::data::PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.root.demand_head_partition(execution, id)
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
    fn probe_full_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        self.probe_full_region(id)
    }

    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_full_region_cached(execution, id)
    }
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.partition_state(partition_id)
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for RelationalBranchRoot {
    fn get_current_partition(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.admitted_head_region(id)
    }
    fn probe_head_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        self.probe_head_region(id)
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_region_head_cached(execution, id)
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
