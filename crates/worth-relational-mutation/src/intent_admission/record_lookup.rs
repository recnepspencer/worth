use crate::contracts::identity::data::record_ids::{EntityId, RecordId, RelationId};
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::ExistingRecordTarget;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::substrate::record_arena::kinds::{
    partition_of, EntityRecordKind, RecordKind, RelationRecordKind,
};

pub fn entity_exists_in_state(state: &impl CurrentPartitionAccess, entity_id: EntityId) -> bool {
    record_exists_in_state::<EntityRecordKind>(state, entity_id)
}

pub fn relation_exists_in_state(
    state: &impl CurrentPartitionAccess,
    relation_id: RelationId,
) -> bool {
    record_exists_in_state::<RelationRecordKind>(state, relation_id)
}

fn record_exists_in_state<K: RecordKind>(
    state: &impl CurrentPartitionAccess,
    record_id: RecordId<K::Domain>,
) -> bool {
    state
        .get_current_partition(partition_of::<K>(&record_id))
        .is_some_and(|partition| partition.arena::<K>().contains_live_id(&record_id))
}

pub(crate) fn require_live_target(
    state: &impl CurrentPartitionAccess,
    target: ExistingRecordTarget,
    context: &str,
) -> Result<(), CommitConflict> {
    let live = match target {
        ExistingRecordTarget::Entity(id) => entity_exists_in_state(state, id),
        ExistingRecordTarget::Relation(id) => relation_exists_in_state(state, id),
    };
    if live {
        return Ok(());
    }
    Err(CommitConflict::new(ConflictClass::StaleTarget {
        target,
        context: context.to_owned(),
    }))
}
