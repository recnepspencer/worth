//! Borrowed record view for one selected branch preparation round.
use crate::branch::root_selection::SelectedRelationalBranchState;
use crate::identity::data::{PartitionId, VersionId};
use crate::storage::overlay::{CurrentPartitionAccess, PartitionIdMemory, PartitionIdsDenial};

#[derive(Clone, Copy)]
pub struct SelectedRelationalBranchRead<'round> {
    selected: &'round SelectedRelationalBranchState,
    partitions: &'round dyn CurrentPartitionAccess,
}

impl<'round> SelectedRelationalBranchRead<'round> {
    pub fn in_round(
        selected: &'round SelectedRelationalBranchState,
        partitions: &'round dyn CurrentPartitionAccess,
    ) -> Self {
        Self {
            selected,
            partitions,
        }
    }

    pub fn selected(&self) -> &'round SelectedRelationalBranchState {
        self.selected
    }

    pub fn version_id(&self) -> VersionId {
        self.selected.version_id()
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for SelectedRelationalBranchRead<'_> {
    fn get_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.partitions.get_current_partition(id)
    }
    fn probe_head_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        self.partitions.probe_head_partition(id)
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: crate::identity::data::PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.partitions.demand_head_partition(execution, id)
    }

    fn has_read_misses(&self) -> bool {
        self.partitions.has_read_misses()
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.partitions.partition_ids()
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        self.partitions.partition_ids_iter()
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
        self.partitions.partition_ids_with_memory(memory)
    }
}

#[cfg(test)]
#[test]
fn selected_branch_read_cannot_implement_the_full_history_source_contract() {
    trait WithoutFull<A> {
        fn witness() {}
    }
    impl<T: ?Sized> WithoutFull<()> for T {}
    impl<T: ?Sized + crate::storage::overlay::PartitionAccess> WithoutFull<u8> for T {}
    let _ = <SelectedRelationalBranchRead<'_> as WithoutFull<_>>::witness;
}
