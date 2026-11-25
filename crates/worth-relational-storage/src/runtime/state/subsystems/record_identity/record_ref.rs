use crate::identity::data::PartitionId;
use crate::transactions::data::primitives::RecordRef;

pub(super) fn record_partition(record: &RecordRef) -> PartitionId {
    match record {
        RecordRef::Entity(id) => id.partition_id,
        RecordRef::Relation(id) => id.partition_id,
    }
}

pub(super) fn record_slot(record: &RecordRef) -> usize {
    match record {
        RecordRef::Entity(id) => id.slot_index(),
        RecordRef::Relation(id) => id.slot_index(),
    }
}

pub(super) fn record_generation(record: &RecordRef) -> u32 {
    match record {
        RecordRef::Entity(id) => id.generation_value(),
        RecordRef::Relation(id) => id.generation_value(),
    }
}
