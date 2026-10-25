//! Whole-edition Full custody, issued only after every declared demand succeeds.
use super::{PartitionAccess, PartitionEdition, PartitionId, PartitionState};

/// The retained edition owns every counted Full lease that backs these reads.
pub(crate) struct ResidentPartitionEdition {
    edition: PartitionEdition,
}

impl ResidentPartitionEdition {
    pub(super) fn new(edition: PartitionEdition) -> Self {
        Self { edition }
    }

    pub(crate) fn partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.edition.partition(id)
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (PartitionId, &PartitionState)> {
        self.edition.entries()
    }

    pub(crate) fn partitions(&self) -> impl Iterator<Item = &PartitionState> {
        self.edition.partitions()
    }
}

impl PartitionAccess for ResidentPartitionEdition {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.partition(id)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.edition.partition_ids()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        self.edition.partition_ids_iter()
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        self.edition.partition_ids_with_memory(memory)
    }
}
