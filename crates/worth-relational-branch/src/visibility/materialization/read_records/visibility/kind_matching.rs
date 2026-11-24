use crate::identity::data::{KindId, VersionId};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{RecordArena, RecordKind};

use super::{visible_metadata, visible_relation_metadata};

pub(in super::super) fn slot_kind_matches_current<K: RecordKind>(
    arena: &RecordArena<K>,
    slot: usize,
    kind_id: KindId,
) -> bool {
    arena
        .get_slot(slot)
        .and_then(|slot_view| slot_view.kind_id())
        == Some(kind_id)
}

pub fn entity_slot_matches_kind_at_version(
    partition: &PartitionState,
    slot: usize,
    kind_id: KindId,
    version_id: VersionId,
    current_version: VersionId,
) -> bool {
    if version_id == current_version {
        return slot_kind_matches_current(&partition.entity_arena, slot, kind_id);
    }
    partition
        .entity_arena
        .metadata_history_at(slot)
        .and_then(|history| visible_metadata(history, version_id))
        .is_some_and(|metadata| metadata.kind_id == kind_id)
}

pub fn relation_slot_matches_kind_at_version(
    partition: &PartitionState,
    slot: usize,
    kind_id: KindId,
    version_id: VersionId,
    current_version: VersionId,
) -> bool {
    if version_id == current_version {
        return slot_kind_matches_current(&partition.relation_arena, slot, kind_id);
    }
    visible_relation_metadata(partition, slot, version_id)
        .is_some_and(|metadata| metadata.kind_id == kind_id)
}
