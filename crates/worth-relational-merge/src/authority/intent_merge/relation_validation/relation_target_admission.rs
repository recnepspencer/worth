use crate::contracts::identity::data::record_ids::RelationId;
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::ExistingRecordTarget;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

use super::super::record_lookup::relation_exists_in_state;

pub(super) fn validate_existing_relation_target(
    state: &impl CurrentPartitionAccess,
    relation_id: RelationId,
) -> Result<(), CommitConflict> {
    if relation_exists_in_state(state, relation_id) {
        return Ok(());
    }

    Err(CommitConflict::new(ConflictClass::StaleTarget {
        target: ExistingRecordTarget::Relation(relation_id),
        context: "relation validation".to_string(),
    }))
}
