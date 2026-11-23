//! Versioned record observations over an already selected partition.
use super::{
    current_entity_partition_source_for_slot, current_relation_partition_source_for_slot,
    CurrentPartitionAccess,
};
use crate::identity::data::{KindId, RecordId, VersionId};
use crate::storage::residency::RelationalRecordHeadArenaView;
use crate::storage::substrate::{HistoricalMetadata, RecordKind};
use crate::transactions::data::primitives::RecordRef;
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(crate) fn visible_record_metadata<'state, K: RecordKind>(
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

pub(crate) fn metadata_is_visible(metadata: &impl HistoricalMetadata, version: VersionId) -> bool {
    metadata.effective_at() <= version
        && metadata
            .retired_at()
            .is_none_or(|retired| version < retired)
}

pub(crate) struct PhysicalRecordObservation<'state> {
    pub(crate) kind: KindId,
    pub(crate) aspects: Option<&'state AuthoritativeRecordAspectState>,
}

pub(crate) fn observe_record<'state>(
    state: &'state dyn CurrentPartitionAccess,
    record: &RecordRef,
    version: VersionId,
) -> Option<PhysicalRecordObservation<'state>> {
    match *record {
        RecordRef::Entity(id) => {
            let (_, partition) =
                current_entity_partition_source_for_slot(state, id.partition_id, id.slot_index())?;
            let metadata = visible_record_metadata(partition.entity_arena(), id, version)?;
            Some(PhysicalRecordObservation {
                kind: metadata.kind_id,
                aspects: metadata.authoritative_aspect_state(),
            })
        }
        RecordRef::Relation(id) => {
            let (_, partition) = current_relation_partition_source_for_slot(
                state,
                id.partition_id,
                id.slot_index(),
            )?;
            let metadata = visible_record_metadata(partition.relation_arena(), id, version)?;
            Some(PhysicalRecordObservation {
                kind: metadata.kind_id,
                aspects: metadata.authoritative_aspect_state(),
            })
        }
    }
}
