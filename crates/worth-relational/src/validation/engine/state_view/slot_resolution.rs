use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::InvariantStateView;

#[derive(Clone, Copy)]
pub(crate) enum PartitionSource {
    Visible,
    Base,
}

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
        let partition = self.state.get_partition(partition_id)?;
        if self.state.entity_slot_is_touched(partition_id, slot)
            || !self.state.has_touched_entity_slots(partition_id)
        {
            return Some((PartitionSource::Visible, partition));
        }
        Some(match self.state.base_partition(partition_id) {
            Some(base) => (PartitionSource::Base, base),
            None => (PartitionSource::Visible, partition),
        })
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
        let partition = self.state.get_partition(partition_id)?;
        if self.state.relation_slot_is_touched(partition_id, slot)
            || (!self.state.has_touched_relation_slots(partition_id)
                && partition.relation_arena.get_slot(slot).is_some())
            || (partition.relation_arena.get_slot(slot).is_some()
                && !partition.relation_overlay_is_sparse)
        {
            return Some((PartitionSource::Visible, partition));
        }
        Some(match self.state.base_partition(partition_id) {
            Some(base) => (PartitionSource::Base, base),
            None => (PartitionSource::Visible, partition),
        })
    }
}
