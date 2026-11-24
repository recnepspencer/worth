//! Keep actual prefix allocations and their symbolic transaction references together.
use crate::execution::request_stage::checkpoint;
use crate::mvcc::publication::application_publication::custody;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::EntityReference;
use worth_execution::{ExecutionRequest, MapKernelContext};

/// A producer sees real allocated IDs in its candidate reads. The enclosing
/// transaction keeps their original Create references for the final full plan,
/// and so does the part's naming plan for each created relation's endpoints.
pub(super) fn retain(
    execution: &ExecutionRequest<'_, '_>,
    bindings: &CommitCreatedEntityBindings,
    intents: &mut [MutationIntent],
    naming: Option<&mut crate::lineage::data::CommitNamingPlan>,
) -> Result<(), TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            for intent in intents {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                intent.visit_entity_references_mut(|reference| {
                    retain_reference(context, bindings, reference)
                })?;
            }
            if let Some(plan) = naming {
                super::part_naming::visit_endpoints_mut(plan, |reference| {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    retain_reference(context, bindings, reference)
                })?;
            }
            Ok(())
        })
        .map_err(custody::execution_stop)?
        .0
}

fn retain_reference(
    context: &mut MapKernelContext<'_, '_>,
    bindings: &CommitCreatedEntityBindings,
    reference: &mut EntityReference,
) -> Result<(), TransactionCommitError> {
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
    Ok(())
}
