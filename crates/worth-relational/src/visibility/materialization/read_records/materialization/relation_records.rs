use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::data::RelationReadRecord;
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::HistoricalMetadata;

use super::super::visibility::{
    historical_created_at, historical_relation_lifecycle, historical_retired_at,
    lifecycle_storage_visible, relation_visible_in_partition_at_version, visible_relation_metadata,
};
pub(crate) fn materialize_current_authoritative_relation_record(
    registry: &RelationalSchemaRegistry,
    partition: &PartitionState,
    partition_id: crate::identity::data::PartitionId,
    slot: usize,
) -> Option<RelationReadRecord> {
    let relation_slot = partition.relation_arena.get_slot(slot)?;
    let lifecycle = relation_slot.lifecycle();
    if !lifecycle_storage_visible(lifecycle) {
        return None;
    }
    let kind_id = relation_slot.kind_id()?;
    let kind = registry.resolve_relation(kind_id).ok()?;
    let endpoints = relation_slot.extra().endpoints.as_ref()?;
    Some(RelationReadRecord {
        relation_id: crate::identity::data::RelationId::new(
            partition_id,
            slot as u64,
            relation_slot.generation(),
        ),
        kind,
        lifecycle,
        created_at_version: partition
            .relation_arena
            .created_at_for_slot(slot)
            .expect(/* Proof: The owner selects this slot from the same retained arena after existence and visibility admission; arena materialization installs its kind, extra and creation columns together. */ "visible relation slot has creation metadata"),
        retired_at_version: relation_slot.retired_at(),
        source: endpoints.source,
        target: endpoints.target,
        authoritative_aspect_state: relation_slot.extra().authoritative_aspect_state.clone(),
    })
}

pub(crate) fn materialize_authoritative_relation_record_at_version(
    registry: &RelationalSchemaRegistry,
    partition: &PartitionState,
    partition_id: crate::identity::data::PartitionId,
    slot: usize,
    version_id: crate::identity::data::VersionId,
) -> Option<RelationReadRecord> {
    if !relation_visible_in_partition_at_version(partition, slot, version_id) {
        return None;
    }
    let history = partition.relation_arena.metadata_history_at(slot)?;
    let metadata = visible_relation_metadata(partition, slot, version_id)?;
    let current = partition.relation_arena.get_slot(slot)?;
    let kind = registry.resolve_relation(metadata.kind_id).ok()?;
    let retired_at = historical_retired_at(metadata.retired_at(), version_id);
    Some(RelationReadRecord {
        relation_id: crate::identity::data::RelationId::new(
            partition_id,
            slot as u64,
            metadata.generation,
        ),
        kind,
        lifecycle: historical_relation_lifecycle(partition, slot, metadata, version_id),
        created_at_version: historical_created_at(
            history,
            metadata,
            current.generation(),
            partition.relation_arena.created_at_for_slot(slot)?,
        ),
        retired_at_version: retired_at,
        source: metadata.endpoints.source,
        target: metadata.endpoints.target,
        authoritative_aspect_state: metadata.authoritative_aspect_state.clone(),
    })
}
