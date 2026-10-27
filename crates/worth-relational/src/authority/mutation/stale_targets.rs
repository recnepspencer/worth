use crate::identity::data::RecordId;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::storage::substrate::{
    generation_of, partition_of, slot_of, EntityRecordKind, RecordKind, RelationRecordKind,
};
use crate::transactions::data::{CommitConflict, ConflictClass, ExistingRecordTarget};

pub(super) fn ensure_entity_target_is_current(
    staged: &impl CurrentPartitionAccess,
    entity_id: crate::identity::data::EntityId,
) -> Result<(), CommitConflict> {
    ensure_target_is_current::<EntityRecordKind>(staged, entity_id, "entity")
}

pub(super) fn ensure_relation_target_is_current(
    staged: &impl CurrentPartitionAccess,
    relation_id: crate::identity::data::RelationId,
) -> Result<(), CommitConflict> {
    ensure_target_is_current::<RelationRecordKind>(staged, relation_id, "relation")
}

fn ensure_target_is_current<K: RecordKind>(
    staged: &impl CurrentPartitionAccess,
    record_id: RecordId<K::Domain>,
    record_kind: &str,
) -> Result<(), CommitConflict> {
    let slot = slot_of::<K>(&record_id);
    let partition_id = partition_of::<K>(&record_id);
    let generation = generation_of::<K>(&record_id);
    let Some(partition) = staged.get_current_partition(partition_id) else {
        return stale_handle_conflict(
            record_kind,
            if record_kind == "entity" {
                ExistingRecordTarget::Entity(crate::identity::data::EntityId::new(
                    partition_id,
                    slot as u64,
                    generation,
                ))
            } else {
                ExistingRecordTarget::Relation(crate::identity::data::RelationId::new(
                    partition_id,
                    slot as u64,
                    generation,
                ))
            },
        );
    };
    let arena = partition.arena::<K>();
    if !arena.contains_live_id(&record_id) {
        return stale_handle_conflict(
            record_kind,
            if record_kind == "entity" {
                ExistingRecordTarget::Entity(crate::identity::data::EntityId::new(
                    partition_id,
                    slot as u64,
                    generation,
                ))
            } else {
                ExistingRecordTarget::Relation(crate::identity::data::RelationId::new(
                    partition_id,
                    slot as u64,
                    generation,
                ))
            },
        );
    }
    Ok(())
}

fn stale_handle_conflict(
    record_kind: &str,
    target: ExistingRecordTarget,
) -> Result<(), CommitConflict> {
    Err(CommitConflict::new(ConflictClass::StaleTarget {
        target,
        context: format!("{record_kind} authoritative apply"),
    }))
}
