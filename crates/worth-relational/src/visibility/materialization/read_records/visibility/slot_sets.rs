use crate::identity::data::{PartitionId, VersionId};
use crate::storage::overlay::{PartitionAccess, PartitionState};
use crate::storage::partition::DenseSlotBitSet;
use crate::storage::substrate::{HistoricalMetadata, RecordArena, RecordKind};

use super::{visible_metadata, visible_relation_metadata};

pub(in super::super) fn entity_visible_in_partition_at_version(
    partition: &PartitionState,
    slot: usize,
    version_id: VersionId,
) -> bool {
    partition
        .entity_arena
        .metadata_history_at(slot)
        .and_then(|history| visible_metadata(history, version_id))
        .is_some()
}

pub(in super::super) fn relation_visible_in_partition_at_version(
    partition: &PartitionState,
    slot: usize,
    version_id: VersionId,
) -> bool {
    visible_relation_metadata(partition, slot, version_id).is_some()
}

pub(in super::super) fn visible_relation_slots_in_partition_from_state(
    current_version: VersionId,
    state: &(impl PartitionAccess + ?Sized),
    partition_id: PartitionId,
    version_id: VersionId,
    count_scans: impl Fn(usize),
) -> Option<DenseSlotBitSet> {
    let partition = state.get_partition(partition_id)?;
    let arena = &partition.relation_arena;
    let mut visible_slots = DenseSlotBitSet::with_capacity(arena.slot_count());
    if version_id == current_version {
        for slot in arena.live_bitset.iter_set_slots() {
            visible_slots.set(slot, true);
        }
    } else {
        count_scans(arena.slot_count());
        for slot in arena.occupied_slots() {
            if visible_relation_metadata(partition, slot, version_id).is_some() {
                visible_slots.set(slot, true);
            }
        }
    }
    (visible_slots.count_ones() > 0).then_some(visible_slots)
}

pub(in super::super) fn record_visible_in_arena_at_version<K: RecordKind>(
    arena: &RecordArena<K>,
    slot: usize,
    version_id: VersionId,
) -> bool
where
    K::Meta: HistoricalMetadata,
{
    arena
        .metadata_history_at(slot)
        .and_then(|history| visible_metadata(history, version_id))
        .is_some()
}

pub(in super::super) fn visible_slots_in_partition_from_state<K: RecordKind>(
    current_version: VersionId,
    state: &(impl PartitionAccess + ?Sized),
    partition_id: PartitionId,
    version_id: VersionId,
    count_scans: impl Fn(usize),
) -> Option<DenseSlotBitSet>
where
    K::Meta: HistoricalMetadata,
{
    let partition = state.get_partition(partition_id)?;
    let arena = K::arena(partition);
    let mut visible_slots = DenseSlotBitSet::with_capacity(arena.slot_count());
    if version_id == current_version {
        for slot in arena.live_bitset.iter_set_slots() {
            visible_slots.set(slot, true);
        }
    } else {
        count_scans(arena.slot_count());
        for slot in arena.occupied_slots() {
            if record_visible_in_arena_at_version(arena, slot, version_id) {
                visible_slots.set(slot, true);
            }
        }
    }
    (visible_slots.count_ones() > 0).then_some(visible_slots)
}
