use crate::identity::data::RelationId;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::transactions::data::primitives::ExistingRecordTarget;

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
