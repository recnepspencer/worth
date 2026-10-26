//! Borrowed record view for one selected branch preparation round.
use super::SelectedRelationalBranchState;
use crate::identity::data::{PartitionId, VersionId};
use crate::storage::overlay::{
    PartitionAccess, PartitionIdMemory, PartitionIdsDenial, PartitionState,
};

#[derive(Clone, Copy)]
pub(crate) struct SelectedRelationalBranchRead<'round> {
    selected: &'round SelectedRelationalBranchState,
    partitions: &'round dyn PartitionAccess,
}

impl<'round> SelectedRelationalBranchRead<'round> {
    pub(crate) fn in_round(
        selected: &'round SelectedRelationalBranchState,
        partitions: &'round dyn PartitionAccess,
    ) -> Self {
        Self {
            selected,
            partitions,
        }
    }

    pub(crate) fn selected(&self) -> &'round SelectedRelationalBranchState {
        self.selected
    }

    pub(crate) fn version_id(&self) -> VersionId {
        self.selected.version_id()
    }
}

impl PartitionAccess for SelectedRelationalBranchRead<'_> {
    fn has_read_misses(&self) -> bool {
        self.partitions.has_read_misses()
    }

    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.partitions.get_partition(id)
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
