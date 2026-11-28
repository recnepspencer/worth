use crate::contracts::identity::data::record_ids::RelationId;
use crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::contracts::transactions::data::primitives::ExistingRecordTarget;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

use super::super::record_lookup::require_live_target;

pub(super) fn validate_existing_relation_target(
    state: &impl CurrentPartitionAccess,
    relation_id: RelationId,
) -> Result<(), CommitConflict> {
    require_live_target(
        state,
        ExistingRecordTarget::Relation(relation_id),
        "relation validation",
    )
}
