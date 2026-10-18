use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::InvariantStateView;

pub(crate) use crate::storage::overlay::PartitionSource;

#[derive(Clone, Copy)]
pub(crate) struct AspectLocation {
    pub(crate) source: PartitionSource,
    pub(crate) history_index: usize,
}

impl<'state> InvariantStateView<'state> {
    pub(super) fn partition_from_source(
        &self,
        partition_id: PartitionId,
        source: PartitionSource,
    ) -> Option<&'state PartitionState> {
        match source {
            PartitionSource::Visible => self.state.get_partition(partition_id),
            PartitionSource::Base => self.state.base_partition(partition_id),
        }
    }

    pub(super) fn entity_partition_for_slot(
        &self,
        partition_id: PartitionId,
        slot: usize,
    ) -> Option<&'state PartitionState> {
        self.entity_partition_source_for_slot(partition_id, slot)
            .map(|(_, partition)| partition)
    }

    pub(super) fn entity_partition_source_for_slot(
        &self,
        partition_id: PartitionId,
        slot: usize,
    ) -> Option<(PartitionSource, &'state PartitionState)> {
        crate::storage::overlay::entity_partition_source_for_slot(self.state, partition_id, slot)
    }

    pub(super) fn relation_partition_for_slot(
        &self,
        partition_id: PartitionId,
        slot: usize,
    ) -> Option<&'state PartitionState> {
        self.relation_partition_source_for_slot(partition_id, slot)
            .map(|(_, partition)| partition)
    }

    pub(super) fn relation_partition_source_for_slot(
        &self,
        partition_id: PartitionId,
        slot: usize,
    ) -> Option<(PartitionSource, &'state PartitionState)> {
        crate::storage::overlay::relation_partition_source_for_slot(self.state, partition_id, slot)
    }
}
