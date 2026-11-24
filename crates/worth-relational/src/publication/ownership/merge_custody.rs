//! Exact current ownership evidence used by the merge certification probe.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::EntityId;
use crate::transactions::data::intents::mutation_intent::{
    DeleteEntityIntent, EntityMutationIntent, MutationIntent,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::{RecordRef, WorkerIntentBatch};
use worth_execution::ExecutionRequest;

pub(crate) fn selected_member(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    owner: EntityId,
) -> Result<Option<EntityId>, TransactionCommitError> {
    let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
    let member = current
        .origins()
        .find(|origin| origin.meaning.source_scope == owner)
        .and_then(|origin| origin.outputs.first())
        .copied();
    Ok(member)
}

pub(crate) fn delete_member_batch(member: EntityId) -> WorkerIntentBatch {
    WorkerIntentBatch::new("external-owned-delete-after-merge").push(MutationIntent::Entity(
        EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: member }),
    ))
}
