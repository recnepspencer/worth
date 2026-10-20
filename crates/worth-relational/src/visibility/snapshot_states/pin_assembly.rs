use std::collections::BTreeMap;

use crate::identity::data::PartitionId;
use crate::snapshots::data::SnapshotHandle;
use crate::storage::overlay::SnapshotPartitionPins;
use crate::storage::partition::DenseSlotBitSet;

use super::{SnapshotState, SnapshotStateBasis};

pub(super) struct SelectedRelationSlots {
    pub(super) partition_id: PartitionId,
    pub(super) visible: DenseSlotBitSet,
    pub(super) retained: DenseSlotBitSet,
}

pub(super) fn assemble_snapshot_state(
    handle: SnapshotHandle,
    basis: SnapshotStateBasis,
    entity_partitions: Vec<(PartitionId, DenseSlotBitSet)>,
    relation_partitions: Vec<SelectedRelationSlots>,
) -> SnapshotState {
    let mut pinned_partitions = BTreeMap::new();
    let mut pinned_entity_count = 0;
    for (partition_id, entity_slots) in entity_partitions {
        pinned_entity_count += entity_slots.count_ones();
        let pins = pinned_partitions
            .entry(partition_id)
            .or_insert_with(empty_partition_pins);
        pins.entity_slots = entity_slots;
    }
    let mut pinned_relation_count = 0;
    for selected in relation_partitions {
        pinned_relation_count += selected.visible.count_ones();
        let pins = pinned_partitions
            .entry(selected.partition_id)
            .or_insert_with(empty_partition_pins);
        pins.relation_slots = selected.visible;
        pins.retained_relation_slots = selected.retained;
    }
    SnapshotState {
        handle,
        basis,
        pinned_entity_count,
        pinned_relation_count,
        pinned_partitions,
    }
}

fn empty_partition_pins() -> SnapshotPartitionPins {
    SnapshotPartitionPins {
        entity_slots: DenseSlotBitSet::with_capacity(0),
        relation_slots: DenseSlotBitSet::with_capacity(0),
        retained_relation_slots: DenseSlotBitSet::with_capacity(0),
    }
}

pub(super) fn assemble_snapshot_state_with_memory(
    handle: SnapshotHandle,
    basis: SnapshotStateBasis,
    entity_partitions: Vec<(PartitionId, DenseSlotBitSet)>,
    relation_partitions: Vec<SelectedRelationSlots>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<SnapshotState, crate::execution::RelationalExecutionDenialCause> {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    use crate::visibility::materialization::read_records::read_work::record_memory;
    let mut entities = entity_partitions.iter().map(|(id, _)| *id).peekable();
    let mut relations = relation_partitions
        .iter()
        .map(|entry| entry.partition_id)
        .peekable();
    let mut entries = 0_u64;
    while entities.peek().is_some() || relations.peek().is_some() {
        match (entities.peek(), relations.peek()) {
            (Some(left), Some(right)) => match left.cmp(right) {
                std::cmp::Ordering::Less => {
                    entities.next();
                }
                std::cmp::Ordering::Greater => {
                    relations.next();
                }
                std::cmp::Ordering::Equal => {
                    entities.next();
                    relations.next();
                }
            },
            (Some(_), None) => {
                entities.next();
            }
            (None, Some(_)) => {
                relations.next();
            }
            _ => unreachable!(),
        }
        entries = entries.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
    }
    // This is the original owner's std B-tree bound: eleven pairs and twelve
    // edges plus header; twice the entry count covers minimally occupied nodes.
    let node = 11 * std::mem::size_of::<(PartitionId, SnapshotPartitionPins)>() as u64
        + 15 * std::mem::size_of::<usize>() as u64;
    let map_bytes = entries
        .checked_mul(2)
        .and_then(|count| count.checked_mul(node))
        .ok_or(Cause::ChargedBytesOverflow)?;
    record_memory::claim(memory, map_bytes)?;
    let vectors = record_memory::vector_bytes::<(PartitionId, DenseSlotBitSet)>(
        entity_partitions.capacity(),
    )?
    .checked_add(record_memory::vector_bytes::<SelectedRelationSlots>(
        relation_partitions.capacity(),
    )?)
    .ok_or(Cause::ChargedBytesOverflow)?;
    let state = assemble_snapshot_state(handle, basis, entity_partitions, relation_partitions);
    record_memory::release(memory, vectors)?;
    Ok(state)
}
