//! Only an operation-scoped root read lends partitions; the root itself and
//! its branch state hold custody alone (Store M4 D1).
use super::root::RelationalRootRead;
use crate::identity::data::PartitionId;
use crate::storage::overlay::{
    CurrentPartitionAccess, PartitionAccess, PartitionIdMemory, PartitionIdsDenial, PartitionState,
};
use crate::storage::residency::{
    PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView, RelationalResidencyDemandFailure,
};
use worth_execution::ExecutionRequest;

impl PartitionAccess for super::root::RelationalBranchRootState {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        match self.probe_full_partition(id) {
            PartitionProbe::Resident(partition) => Some(partition),
            _ => None,
        }
    }
    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        self.root
            .regions_for_projection()
            .probe_full_region(id, &self.residency)
    }
    fn demand_full_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.root
            .regions_for_projection()
            .demand_full_region_cached(execution, id, &self.residency)
    }
}

impl CurrentPartitionAccess for super::root::RelationalBranchRootState {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.root
            .regions_for_projection()
            .accepted_head_region_view(id, &self.residency)
    }
    fn probe_head_partition(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        self.root
            .regions_for_projection()
            .probe_head_region(id, &self.residency)
    }
    fn demand_head_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.root
            .regions_for_projection()
            .demand_region_head_cached(execution, id, &self.residency)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.root.partition_ids()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.root.partition_ids_iter())
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
        crate::storage::overlay::collect_partition_ids(
            self.root.region_count(),
            self.root.partition_ids_iter(),
            memory,
        )
    }
}

impl PartitionAccess for RelationalRootRead<'_> {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        match self.probe_full_region(id) {
            PartitionProbe::Resident(partition) => Some(partition),
            _ => None,
        }
    }
    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        self.probe_full_region(id)
    }
    fn demand_full_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.demand_region_full(execution, id).map(drop)
    }
}

impl CurrentPartitionAccess for RelationalRootRead<'_> {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        // Fresh admission belongs to probe_head_partition; a paid read consumes
        // the immutable counted custody this operation already accepted.
        self.accepted_head_region_view(id)
    }
    fn probe_head_partition(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        self.probe_head_region(id)
    }
    fn demand_head_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.demand_region_head_cached(execution, id)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.root().partition_ids()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.root().partition_ids_iter())
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
        crate::storage::overlay::collect_partition_ids(
            self.root().region_count(),
            self.root().partition_ids_iter(),
            memory,
        )
    }
}
