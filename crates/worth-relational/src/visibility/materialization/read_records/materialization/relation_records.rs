use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::data::RelationReadRecord;
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::HistoricalMetadata;

use super::super::visibility::{
    historical_created_at, historical_relation_lifecycle, historical_retired_at,
    relation_visible_in_partition_at_version, visible_relation_metadata,
};
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
        authoritative_aspect_state: metadata.authoritative_aspect_state().cloned(),
    })
}
