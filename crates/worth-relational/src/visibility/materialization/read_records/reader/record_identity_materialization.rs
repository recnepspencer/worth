//! Materialize one identity from explicitly carried storage and schema.
use super::*;

pub(crate) fn authoritative_entity_record_for_id_from_exact_state(
    state: &(impl PartitionAccess + ?Sized),
    registry: &crate::schema::data::RelationalSchemaRegistry,
    entity_id: crate::identity::data::EntityId,
) -> Option<EntityReadRecord> {
    let slot = entity_id.slot_index();
    materialize_current_authoritative_entity_record(
        registry,
        state.get_partition(entity_id.partition_id)?,
        entity_id.partition_id,
        slot,
    )
    .filter(|record| {
        entity_id.generation.is_zero() || record.entity_id.generation == entity_id.generation
    })
}

pub(crate) fn authoritative_relation_record_for_id_from_exact_state(
    state: &(impl PartitionAccess + ?Sized),
    registry: &crate::schema::data::RelationalSchemaRegistry,
    relation_id: crate::identity::data::RelationId,
) -> Option<RelationReadRecord> {
    let slot = relation_id.slot_index();
    materialize_current_authoritative_relation_record(
        registry,
        state.get_partition(relation_id.partition_id)?,
        relation_id.partition_id,
        slot,
    )
    .filter(|record| {
        relation_id.generation.is_zero() || record.relation_id.generation == relation_id.generation
    })
}

pub(crate) fn authoritative_entity_record_for_id_at_version_with_registry(
    state: &(impl PartitionAccess + ?Sized),
    registry: &crate::schema::data::RelationalSchemaRegistry,
    entity_id: crate::identity::data::EntityId,
    version_id: crate::identity::data::VersionId,
    current_version: crate::identity::data::VersionId,
) -> Option<EntityReadRecord> {
    let partition = state.get_partition(entity_id.partition_id)?;
    let slot = entity_id.slot_index();
    if version_id == current_version {
        materialize_current_authoritative_entity_record(
            registry,
            partition,
            entity_id.partition_id,
            slot,
        )
        .filter(|record| {
            entity_id.generation.is_zero() || record.entity_id.generation == entity_id.generation
        })
    } else {
        materialize_authoritative_entity_record_at_version(
            registry,
            partition,
            entity_id.partition_id,
            slot,
            version_id,
        )
        .filter(|record| {
            entity_id.generation.is_zero() || record.entity_id.generation == entity_id.generation
        })
    }
}

pub(crate) fn authoritative_relation_record_for_id_at_version_with_registry(
    state: &(impl PartitionAccess + ?Sized),
    registry: &crate::schema::data::RelationalSchemaRegistry,
    relation_id: crate::identity::data::RelationId,
    version_id: crate::identity::data::VersionId,
    current_version: crate::identity::data::VersionId,
) -> Option<RelationReadRecord> {
    let partition = state.get_partition(relation_id.partition_id)?;
    let slot = relation_id.slot_index();
    if version_id == current_version {
        materialize_current_authoritative_relation_record(
            registry,
            partition,
            relation_id.partition_id,
            slot,
        )
        .filter(|record| {
            relation_id.generation.is_zero()
                || record.relation_id.generation == relation_id.generation
        })
    } else {
        materialize_authoritative_relation_record_at_version(
            registry,
            partition,
            relation_id.partition_id,
            slot,
            version_id,
        )
        .filter(|record| {
            relation_id.generation.is_zero()
                || record.relation_id.generation == relation_id.generation
        })
    }
}
