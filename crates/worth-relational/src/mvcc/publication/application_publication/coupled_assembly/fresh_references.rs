//! Keep actual prefix allocations and their symbolic transaction references together.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::transactions::data::{CommitCreatedEntityBindings, EntityReference, MutationIntent};

/// A producer sees real allocated IDs in its candidate reads. The enclosing
/// transaction keeps their original Create references for the final full plan.
pub(super) fn retain(
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
