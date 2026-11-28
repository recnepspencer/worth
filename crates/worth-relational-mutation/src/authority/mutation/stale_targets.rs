use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::contracts::transactions::data::primitives::ExistingRecordTarget;
use crate::intent_admission::record_lookup::require_live_target;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

pub(super) fn ensure_entity_target_is_current(
    state: &impl CurrentPartitionAccess,
    entity_id: EntityId,
) -> Result<(), CommitConflict> {
    require_live_target(
        state,
        ExistingRecordTarget::Entity(entity_id),
        "entity authoritative apply",
    )
}

pub(super) fn ensure_relation_target_is_current(
    state: &impl CurrentPartitionAccess,
    relation_id: RelationId,
) -> Result<(), CommitConflict> {
    require_live_target(
        state,
        ExistingRecordTarget::Relation(relation_id),
        "relation authoritative apply",
    )
}
