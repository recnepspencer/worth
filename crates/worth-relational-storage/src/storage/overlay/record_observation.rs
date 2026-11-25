//! Versioned record observations over an already selected partition.
use super::{
    current_entity_partition_source_for_slot, current_relation_partition_source_for_slot,
    CurrentPartitionAccess,
};
use crate::identity::data::{EntityId, KindId, RecordId, RelationId, VersionId};
use crate::storage::residency::RelationalRecordHeadArenaView;
use crate::storage::substrate::{
    HistoricalMetadata, RecordKind, VersionedEntityMetadata, VersionedRelationMetadata,
};
use crate::transactions::data::primitives::RecordRef;
use worth_foundational::facade::AuthoritativeRecordAspectState;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

pub fn visible_record_metadata<'state, K: RecordKind>(
    arena: RelationalRecordHeadArenaView<'state, K>,
    id: RecordId<K::Domain>,
    version: VersionId,
) -> Option<&'state K::Meta> {
    if arena.get(&id)?.generation() != id.generation_value() {
        return None;
    }
    arena
        .current_metadata_at(id.slot_index())
        .filter(|metadata| metadata_is_visible(*metadata, version))
}

pub fn metadata_is_visible(metadata: &impl HistoricalMetadata, version: VersionId) -> bool {
    metadata.effective_at() <= version
        && metadata
            .retired_at()
            .is_none_or(|retired| version < retired)
}

pub fn observe_entity_metadata(
    state: &dyn CurrentPartitionAccess,
    id: EntityId,
    version: VersionId,
) -> Option<&VersionedEntityMetadata> {
    let (_, partition) =
        current_entity_partition_source_for_slot(state, id.partition_id, id.slot_index())?;
    visible_record_metadata(partition.entity_arena(), id, version)
}

pub fn observe_relation_metadata(
    state: &dyn CurrentPartitionAccess,
    id: RelationId,
    version: VersionId,
) -> Option<&VersionedRelationMetadata> {
    let (_, partition) =
        current_relation_partition_source_for_slot(state, id.partition_id, id.slot_index())?;
    visible_record_metadata(partition.relation_arena(), id, version)
}

pub struct PhysicalRecordObservation<'state> {
    pub kind: KindId,
    pub aspects: Option<&'state AuthoritativeRecordAspectState>,
}

pub fn observe_record<'state>(
    state: &'state dyn CurrentPartitionAccess,
    record: &RecordRef,
    version: VersionId,
) -> Option<PhysicalRecordObservation<'state>> {
    match *record {
        RecordRef::Entity(id) => {
            let metadata = observe_entity_metadata(state, id, version)?;
            Some(PhysicalRecordObservation {
                kind: metadata.kind_id,
                aspects: metadata.authoritative_aspect_state(),
            })
        }
        RecordRef::Relation(id) => {
            let metadata = observe_relation_metadata(state, id, version)?;
            Some(PhysicalRecordObservation {
                kind: metadata.kind_id,
                aspects: metadata.authoritative_aspect_state(),
            })
        }
    }
}

/// Metadata classification uses the actual schema and retained record generation,
/// including records just deleted by this unpublished candidate.
pub fn is_metadata_record(
    registry: &crate::schema::data::RelationalSchemaRegistry,
    state: &dyn CurrentPartitionAccess,
    record: &RecordRef,
) -> bool {
    let kind = match record {
        RecordRef::Entity(id) => state
            .get_current_partition(id.partition_id)
            .and_then(|partition| partition.entity_arena().get(id))
            .and_then(|row| row.kind_id()),
        RecordRef::Relation(id) => state
            .get_current_partition(id.partition_id)
            .and_then(|partition| partition.relation_arena().get(id))
            .and_then(|row| row.kind_id()),
    };
    kind.is_some_and(|kind| (*registry.owner_native_private_kinds()).contains(&kind))
}
