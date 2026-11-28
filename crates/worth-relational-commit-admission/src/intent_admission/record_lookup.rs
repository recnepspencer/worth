use crate::contracts::identity::data::record_ids::{EntityId, RecordId, RelationId};
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
        .and_then(|partition| partition.arena::<K>().get(&record_id))
        .is_some_and(|slot| slot.is_live())
}
