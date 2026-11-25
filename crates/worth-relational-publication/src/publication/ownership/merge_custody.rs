//! Exact current ownership evidence used by the merge certification probe.
use contracts::identity::data::record_ids::EntityId;
use contracts::transactions::data::intents::mutation_intent::DeleteEntityIntent;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use contracts::transactions::data::primitives::WorkerIntentBatch;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_contracts::internal_port as contracts;

pub fn selected_member(
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

pub fn delete_member_batch(member: EntityId) -> WorkerIntentBatch {
    WorkerIntentBatch::new("external-owned-delete-after-merge").push(MutationIntent::Entity(
        EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: member }),
    ))
}
