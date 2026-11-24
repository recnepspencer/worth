//! Keep actual prefix allocations and their symbolic transaction references together.
use crate::mvcc::publication::application_publication::custody;
use contracts::execution::request_stage::checkpoint;
use contracts::symbols::data::client_key::ClientKeyOwnerPort as _;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::EntityReference;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;

/// A producer sees real allocated IDs in its candidate reads. The enclosing
/// transaction keeps their original Create references for the final full plan.
pub fn retain(
    execution: &ExecutionRequest<'_, '_>,
    bindings: &CommitCreatedEntityBindings,
    intents: &mut [MutationIntent],
) -> Result<(), TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            for intent in intents {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                intent.visit_entity_references_mut(|reference| {
                    if let EntityReference::Existing(id) = reference
                        && let Some(created) = bindings.creation_reference(*id, || {
                            checkpoint(context, 1).map_err(custody::execution_stop)
                        })?
                    {
                        checkpoint(
                            context,
                            created
                                .client_key
                                .owned_allocation_capacity_bytes()
                                .saturating_add(1),
                        )
                        .map_err(custody::execution_stop)?;
                        *reference = EntityReference::Created(created.clone());
                    }
                    Ok::<_, TransactionCommitError>(())
                })?;
            }
            Ok(())
        })
        .map_err(custody::execution_stop)?
        .0
}
