use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::data::RelationReadRecord;
use crate::storage::overlay::PartitionState;
use super::record_history::HistoricalSlot;

use super::super::visibility::{
    historical_relation_lifecycle,
    relation_visible_in_partition_at_version, visible_relation_metadata,
};
pub fn materialize_authoritative_relation_record_at_version(
    registry: &RelationalSchemaRegistry,
    partition: &PartitionState,
    partition_id: crate::identity::data::PartitionId,
    slot: usize,
    version_id: crate::identity::data::VersionId,
) -> Option<RelationReadRecord> {
    if !relation_visible_in_partition_at_version(partition, slot, version_id) {
        return None;
    }
    let metadata = visible_relation_metadata(partition, slot, version_id)?;
    let kind = registry.resolve_relation(metadata.kind_id).ok()?;
    let lifetime = HistoricalSlot::read(&partition.relation_arena, slot, metadata, version_id)?;
    Some(RelationReadRecord {
        relation_id: crate::identity::data::RelationId::new(
            partition_id,
            slot as u64,
            metadata.generation,
        ),
        kind,
        lifecycle: historical_relation_lifecycle(partition, slot, metadata, version_id),
        created_at_version: lifetime.created,
        retired_at_version: lifetime.retired,
        source: metadata.endpoints.source,
        target: metadata.endpoints.target,
        authoritative_aspect_state: metadata.authoritative_aspect_state().cloned(),
    })
}
