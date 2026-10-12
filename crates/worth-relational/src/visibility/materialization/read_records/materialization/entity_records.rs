use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::data::{EntityReadRecord, RecordLifecycleState};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::HistoricalMetadata;

use super::super::visibility::{
    entity_visible_in_partition_at_version, historical_created_at, historical_lifecycle,
    historical_retired_at, visible_metadata,
};
pub(crate) fn materialize_current_authoritative_entity_record(
    registry: &RelationalSchemaRegistry,
    partition: &PartitionState,
    partition_id: crate::identity::data::PartitionId,
    slot: usize,
) -> Option<EntityReadRecord> {
    #[cfg(test)]
    super::row_reads_for_test::record();
    let entity_slot = partition.entity_arena.get_slot(slot)?;
    if entity_slot.lifecycle() != RecordLifecycleState::Live {
        return None;
    }
    let kind_id = entity_slot.kind_id()?;
    let kind = registry.resolve_entity(kind_id).ok()?;
    Some(EntityReadRecord {
        entity_id: crate::identity::data::EntityId::new(
            partition_id,
            slot as u64,
            entity_slot.generation(),
        ),
        lineage_id: entity_slot.extra().lineage_id,
        kind,
        lifecycle: entity_slot.lifecycle(),
        created_at_version: partition
            .entity_arena
            .created_at_for_slot(slot)
            .expect("visible entity slot has creation metadata"),
        retired_at_version: entity_slot.retired_at(),
        authoritative_aspect_state: entity_slot.extra().authoritative_aspect_state.clone(),
    })
}

pub(crate) fn materialize_authoritative_entity_record_at_version(
    registry: &RelationalSchemaRegistry,
    partition: &PartitionState,
    partition_id: crate::identity::data::PartitionId,
    slot: usize,
    version_id: crate::identity::data::VersionId,
) -> Option<EntityReadRecord> {
    #[cfg(test)]
    super::row_reads_for_test::record();
    if !entity_visible_in_partition_at_version(partition, slot, version_id) {
        return None;
    }
    let history = partition.entity_arena.metadata_history_at(slot)?;
    let metadata = visible_metadata(history, version_id)?;
    let current = partition.entity_arena.get_slot(slot)?;
    let kind = registry.resolve_entity(metadata.kind_id).ok()?;
    let retired_at = historical_retired_at(metadata.retired_at(), version_id);
    Some(EntityReadRecord {
        entity_id: crate::identity::data::EntityId::new(
            partition_id,
            slot as u64,
            metadata.generation,
        ),
        lineage_id: metadata.lineage_id,
        kind,
        lifecycle: historical_lifecycle(retired_at, version_id),
        created_at_version: historical_created_at(
            history,
            metadata,
            current.generation(),
            partition.entity_arena.created_at_for_slot(slot)?,
        ),
        retired_at_version: retired_at,
        authoritative_aspect_state: metadata.authoritative_aspect_state.clone(),
    })
}
