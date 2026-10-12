mod entity_records;
mod relation_records;

pub(crate) use entity_records::{
    materialize_authoritative_entity_record_at_version,
    materialize_current_authoritative_entity_record,
};
pub(crate) use relation_records::{
    materialize_authoritative_relation_record_at_version,
    materialize_current_authoritative_relation_record,
};

#[cfg(test)]
pub(crate) mod row_reads_for_test;
