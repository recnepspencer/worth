use crate::identity::data::PartitionId;
use crate::storage::residency::RelationalRegionHeadView;

use super::StructuralStateView;

pub use crate::storage::overlay::PartitionSource;

#[derive(Clone, Copy)]
pub struct AspectLocation {
    pub source: PartitionSource,
}

impl<'state> StructuralStateView<'state> {
    pub fn partition_from_source(
        &self,
        partition_id: PartitionId,
        source: PartitionSource,
    ) -> Option<RelationalRegionHeadView<'state>> {
        match source {
            PartitionSource::Visible => self.state.get_current_partition(partition_id),
            PartitionSource::Base => self.state.base_current_partition(partition_id),
        }
    }

    pub(super) fn entity_partition_for_slot(
        &self,
        partition_id: PartitionId,
        slot: usize,
    ) -> Option<RelationalRegionHeadView<'state>> {
        self.entity_partition_source_for_slot(partition_id, slot)
            .map(|(_, partition)| partition)
    }

    pub fn entity_partition_source_for_slot(
        &self,
        partition_id: PartitionId,
        slot: usize,
    ) -> Option<(PartitionSource, RelationalRegionHeadView<'state>)> {
        crate::storage::overlay::current_entity_partition_source_for_slot(
            self.state,
            partition_id,
            slot,
        )
    }

    pub(super) fn relation_partition_for_slot(
        &self,
        partition_id: PartitionId,
        slot: usize,
    ) -> Option<RelationalRegionHeadView<'state>> {
        self.relation_partition_source_for_slot(partition_id, slot)
            .map(|(_, partition)| partition)
    }

    pub fn relation_partition_source_for_slot(
        &self,
        partition_id: PartitionId,
        slot: usize,
    ) -> Option<(PartitionSource, RelationalRegionHeadView<'state>)> {
        crate::storage::overlay::current_relation_partition_source_for_slot(
            self.state,
            partition_id,
            slot,
        )
    }
}
