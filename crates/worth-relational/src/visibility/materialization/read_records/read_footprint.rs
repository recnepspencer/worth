use crate::storage::data::{EntityReadRecord, RelationReadRecord};

pub(crate) fn entity(record: &EntityReadRecord) -> u64 {
    (record.kind.kind_name.capacity() as u64)
        .saturating_add(record.kind.schema_id.0.capacity() as u64)
        .saturating_add(
            record
                .authoritative_aspect_state
                .as_ref()
                .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
        )
}

pub(crate) fn relation(record: &RelationReadRecord) -> u64 {
    (record.kind.kind_name.capacity() as u64)
        .saturating_add(record.kind.schema_id.0.capacity() as u64)
        .saturating_add(
            record
                .authoritative_aspect_state
                .as_ref()
                .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
        )
}

pub(crate) fn entities(records: &Vec<EntityReadRecord>) -> u64 {
    (records.capacity() as u64)
        .saturating_mul(std::mem::size_of::<EntityReadRecord>() as u64)
        .saturating_add(records.iter().map(entity).fold(0, u64::saturating_add))
}

pub(crate) fn relations(records: &Vec<RelationReadRecord>) -> u64 {
    (records.capacity() as u64)
        .saturating_mul(std::mem::size_of::<RelationReadRecord>() as u64)
        .saturating_add(records.iter().map(relation).fold(0, u64::saturating_add))
}
