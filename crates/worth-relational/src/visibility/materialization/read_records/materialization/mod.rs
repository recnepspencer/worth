mod entity_records;
mod relation_records;

pub(crate) use crate::storage::partition::current_records::{
    materialize_current_authoritative_entity_record,
    materialize_current_authoritative_relation_record,
};
pub(crate) use entity_records::materialize_authoritative_entity_record_at_version;
pub(crate) use relation_records::materialize_authoritative_relation_record_at_version;

#[cfg(test)]
pub(crate) use crate::storage::partition::current_records::row_reads_for_test;
