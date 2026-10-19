//! Materialize the selected record only after its caller granted the original quote.
use super::super::super::materialization;
use super::ProjectionRecordSource;
use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionAccess;

pub(super) fn entity(
    source: &(impl ProjectionRecordSource + ?Sized),
    id: EntityId,
) -> Option<EntityReadRecord> {
    let root = source.selected_root()?;
    let partition = root.get_partition(id.partition_id)?;
    let registry = root.schema_authority().registry();
    let record = if source.is_exact_basis() {
        materialization::materialize_current_authoritative_entity_record(
            registry,
            partition,
            id.partition_id,
            id.slot_index(),
        )
    } else {
        materialization::materialize_authoritative_entity_record_at_version(
            registry,
            partition,
            id.partition_id,
            id.slot_index(),
            source.version_id(),
        )
    }?;
    (id.generation.is_zero() || record.entity_id.generation == id.generation).then_some(record)
}

pub(super) fn relation(
    source: &(impl ProjectionRecordSource + ?Sized),
    id: RelationId,
) -> Option<RelationReadRecord> {
    let root = source.selected_root()?;
    let partition = root.get_partition(id.partition_id)?;
    let registry = root.schema_authority().registry();
    let record = if source.is_exact_basis() {
        materialization::materialize_current_authoritative_relation_record(
            registry,
            partition,
            id.partition_id,
            id.slot_index(),
        )
    } else {
        materialization::materialize_authoritative_relation_record_at_version(
            registry,
            partition,
            id.partition_id,
            id.slot_index(),
            source.version_id(),
        )
    }?;
    (id.generation.is_zero() || record.relation_id.generation == id.generation).then_some(record)
}
