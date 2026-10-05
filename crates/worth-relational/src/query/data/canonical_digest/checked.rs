use sha2::{Digest, Sha256};

use crate::query::data::QueryOrderingContract;
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

use super::primitive_terms::{encode_string, encode_usize};
use super::read_record_terms::{encode_entity_read_record, encode_relation_read_record};
use super::scope_terms::encode_query_ordering_contract;

/// Streams the same canonical bytes as the serial digest while admitting each
/// record's encoding scratch and charging its work before encoding it.
pub(crate) fn query_result_reduction_digest_checked<E>(
    ordering: QueryOrderingContract,
    entities: &[EntityReadRecord],
    relations: &[RelationReadRecord],
    mut check: impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<String, E> {
    let mut hasher = Sha256::new();
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "query.reduction-result.v1");
    encode_query_ordering_contract(&mut bytes, ordering);
    encode_usize(&mut bytes, entities.len());
    hasher.update(&bytes);
    for record in entities {
        check(
            record_work_units(entity_record_bytes(record)),
            record_temporary_bound(entity_record_bytes(record)),
        )?;
        bytes.clear();
        encode_entity_read_record(&mut bytes, record);
        check(0, bytes.capacity() as u64)?;
        hasher.update(&bytes);
    }
    bytes.clear();
    encode_usize(&mut bytes, relations.len());
    hasher.update(&bytes);
    for record in relations {
        check(
            record_work_units(relation_record_bytes(record)),
            record_temporary_bound(relation_record_bytes(record)),
        )?;
        bytes.clear();
        encode_relation_read_record(&mut bytes, record);
        check(0, bytes.capacity() as u64)?;
        hasher.update(&bytes);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(crate) fn entity_record_digest_checked<E>(
    record: &EntityReadRecord,
    mut check: impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<String, E> {
    let bound = record_temporary_bound(entity_record_bytes(record));
    check(record_work_units(bound), bound)?;
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "query.entity-record.v1");
    encode_entity_read_record(&mut bytes, record);
    check(0, bytes.capacity() as u64)?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

pub(crate) fn relation_record_digest_checked<E>(
    record: &RelationReadRecord,
    mut check: impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<String, E> {
    let bound = record_temporary_bound(relation_record_bytes(record));
    check(record_work_units(bound), bound)?;
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "query.relation-record.v1");
    encode_relation_read_record(&mut bytes, record);
    check(0, bytes.capacity() as u64)?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

pub(crate) fn entity_record_bytes(record: &EntityReadRecord) -> u64 {
    (std::mem::size_of::<EntityReadRecord>() as u64)
        .saturating_add(record.kind.kind_name.capacity() as u64)
        .saturating_add(record.kind.schema_id.0.capacity() as u64)
        .saturating_add(
            record
                .authoritative_aspect_state
                .as_ref()
                .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
        )
}

pub(crate) fn relation_record_bytes(record: &RelationReadRecord) -> u64 {
    (std::mem::size_of::<RelationReadRecord>() as u64)
        .saturating_add(record.kind.kind_name.capacity() as u64)
        .saturating_add(record.kind.schema_id.0.capacity() as u64)
        .saturating_add(
            record
                .authoritative_aspect_state
                .as_ref()
                .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
        )
}

fn record_temporary_bound(record_bytes: u64) -> u64 {
    record_bytes.saturating_mul(4).saturating_add(1024)
}

fn record_work_units(record_bytes: u64) -> u64 {
    (record_bytes.saturating_add(4095) / 4096).max(1)
}
