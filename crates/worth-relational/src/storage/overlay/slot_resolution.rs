//! One sparse overlay/base row selection rule for native reads and preparation.
use super::{PartitionAccess, PartitionState};
use crate::identity::data::PartitionId;

#[derive(Clone, Copy)]
pub(crate) enum PartitionSource {
    Visible,
    Base,
}

pub(crate) fn entity_partition_source_for_slot(
    state: &dyn PartitionAccess,
    partition_id: PartitionId,
    slot: usize,
) -> Option<(PartitionSource, &PartitionState)> {
    let partition = state.get_partition(partition_id)?;
    if state.entity_slot_is_touched(partition_id, slot)
        || !state.has_touched_entity_slots(partition_id)
    {
        return Some((PartitionSource::Visible, partition));
    }
    Some(match state.base_partition(partition_id) {
        Some(base) => (PartitionSource::Base, base),
        None => (PartitionSource::Visible, partition),
    })
}

pub(crate) fn relation_partition_source_for_slot(
    state: &dyn PartitionAccess,
    partition_id: PartitionId,
    slot: usize,
) -> Option<(PartitionSource, &PartitionState)> {
    let partition = state.get_partition(partition_id)?;
    if state.relation_slot_is_touched(partition_id, slot)
        || (!state.has_touched_relation_slots(partition_id)
            && partition.relation_arena.get_slot(slot).is_some())
        || (partition.relation_arena.get_slot(slot).is_some()
            && !partition.relation_overlay_is_sparse)
    {
        return Some((PartitionSource::Visible, partition));
    }
    Some(match state.base_partition(partition_id) {
        Some(base) => (PartitionSource::Base, base),
        None => (PartitionSource::Visible, partition),
    })
}
