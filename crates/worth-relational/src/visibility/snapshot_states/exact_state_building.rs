use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::DenseSlotBitSet;

use super::pin_assembly::SelectedRelationSlots;

pub(in crate::visibility) fn exact_entity_slots(
    state: &impl PartitionAccess,
) -> Vec<(crate::identity::data::PartitionId, DenseSlotBitSet)> {
    state
        .partition_ids()
        .into_iter()
        .filter_map(|partition_id| {
            state
                .get_partition(partition_id)
                .map(|partition| (partition_id, partition.entity_arena.live_bitset.clone()))
        })
        .collect()
}

pub(in crate::visibility) fn exact_relation_slots(
    root: &impl PartitionAccess,
) -> Vec<SelectedRelationSlots> {
    root.partition_ids()
        .into_iter()
        .filter_map(|partition_id| {
            let visible = root
                .get_partition(partition_id)?
                .relation_arena
                .live_bitset
                .clone();
            Some(SelectedRelationSlots {
                partition_id,
                visible,
                // Exact reads obtain lifecycle from this same immutable root.
                // Only historical reconstruction needs a captured override.
                retained: DenseSlotBitSet::with_capacity(0),
            })
        })
        .collect()
}
