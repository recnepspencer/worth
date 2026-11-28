mod record_history;
pub mod entity_records;
pub mod relation_records;

pub use crate::storage::partition::current_records::materialize_current_authoritative_relation_record;
pub use entity_records::materialize_authoritative_entity_record_at_version;
pub use relation_records::materialize_authoritative_relation_record_at_version;

#[doc(hidden)]
pub mod row_reads_for_test;

#[cfg(not(test))]
pub use crate::storage::partition::current_records::materialize_current_authoritative_entity_record;

#[cfg(test)]
pub fn materialize_current_authoritative_entity_record(
    registry: &crate::schema::data::RelationalSchemaRegistry,
    partition: &crate::storage::overlay::PartitionState,
    partition_id: crate::identity::data::PartitionId,
    slot: usize,
) -> Option<crate::storage::data::EntityReadRecord> {
    row_reads_for_test::record();
    crate::storage::partition::current_records::materialize_current_authoritative_entity_record(
        registry,
        partition,
        partition_id,
        slot,
    )
}
