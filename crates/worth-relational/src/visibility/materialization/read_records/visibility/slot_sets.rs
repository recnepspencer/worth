use super::super::read_work::record_memory;
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
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    count_scans: impl Fn(usize),
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<Option<DenseSlotBitSet>, crate::execution::RelationalExecutionDenialCause> {
    let Some(partition) = state.get_partition(partition_id) else {
        return Ok(None);
    };
    let arena = &partition.relation_arena;
    let mut visible_slots = DenseSlotBitSet::with_capacity(arena.slot_count());
    if version_id == current_version {
        let (slots, held) = record_memory::slots(&arena.live_bitset, memory)?;
        for slot in slots {
            charge(super::super::read_prices::EXAMINE_SLOT)
                .map_err(crate::execution::denial::kernel_stop)?;
            charge(super::super::read_prices::RESERVE_RECORD)
                .map_err(crate::execution::denial::kernel_stop)?;
            record_memory::set_slot(memory, &mut visible_slots, slot)?;
        }
        record_memory::release(memory, held)?;
    } else {
        count_scans(arena.slot_count());
        let (slots, held) = record_memory::occupied_slots(arena, memory)?;
        for slot in slots {
            charge(super::super::read_prices::EXAMINE_SLOT)
                .map_err(crate::execution::denial::kernel_stop)?;
            if visible_relation_metadata(partition, slot, version_id).is_some() {
                charge(super::super::read_prices::RESERVE_RECORD)
                    .map_err(crate::execution::denial::kernel_stop)?;
                record_memory::set_slot(memory, &mut visible_slots, slot)?;
            }
        }
        record_memory::release(memory, held)?;
    }
    if visible_slots.count_ones() == 0 {
        let bytes = visible_slots
            .owned_allocation_bytes()
            .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
        drop(visible_slots);
        record_memory::release(memory, bytes)?;
        Ok(None)
    } else {
        Ok(Some(visible_slots))
    }
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
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    count_scans: impl Fn(usize),
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<Option<DenseSlotBitSet>, crate::execution::RelationalExecutionDenialCause>
where
    K::Meta: HistoricalMetadata,
{
    let Some(partition) = state.get_partition(partition_id) else {
        return Ok(None);
    };
    let arena = K::arena(partition);
    let mut visible_slots = DenseSlotBitSet::with_capacity(arena.slot_count());
    if version_id == current_version {
        let (slots, held) = record_memory::slots(&arena.live_bitset, memory)?;
        for slot in slots {
            charge(super::super::read_prices::EXAMINE_SLOT)
                .map_err(crate::execution::denial::kernel_stop)?;
            charge(super::super::read_prices::RESERVE_RECORD)
                .map_err(crate::execution::denial::kernel_stop)?;
            record_memory::set_slot(memory, &mut visible_slots, slot)?;
        }
        record_memory::release(memory, held)?;
    } else {
        count_scans(arena.slot_count());
        let (slots, held) = record_memory::occupied_slots(arena, memory)?;
        for slot in slots {
            charge(super::super::read_prices::EXAMINE_SLOT)
                .map_err(crate::execution::denial::kernel_stop)?;
            if record_visible_in_arena_at_version(arena, slot, version_id) {
                charge(super::super::read_prices::RESERVE_RECORD)
                    .map_err(crate::execution::denial::kernel_stop)?;
                record_memory::set_slot(memory, &mut visible_slots, slot)?;
            }
        }
        record_memory::release(memory, held)?;
    }
    if visible_slots.count_ones() == 0 {
        let bytes = visible_slots
            .owned_allocation_bytes()
            .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
        drop(visible_slots);
        record_memory::release(memory, bytes)?;
        Ok(None)
    } else {
        Ok(Some(visible_slots))
    }
}
