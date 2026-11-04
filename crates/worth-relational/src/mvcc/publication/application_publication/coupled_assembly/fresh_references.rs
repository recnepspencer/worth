//! Keep actual prefix allocations and their symbolic transaction references together.
use super::*;
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
                context.checkpoint(1).map_err(custody::execution_stop)?;
                intent.visit_entity_references_mut(|reference| {
                    if let EntityReference::Existing(id) = reference
                        && let Some(created) = bindings.creation_reference(*id, || {
                            context.checkpoint(1).map_err(custody::execution_stop)
                        })?
                    {
                        context
                            .checkpoint(
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

/// The sequential suffix merges against the actual prefix root, where those
/// same Create references have already acquired their real IDs.
pub(super) fn resolve(
    execution: &ExecutionRequest<'_, '_>,
    bindings: &CommitCreatedEntityBindings,
    intents: &mut [MutationIntent],
) -> Result<(), TransactionCommitError> {
    execution.run_native_with_checkpoint(|context| {
        for intent in intents {
            context.checkpoint(1).map_err(custody::execution_stop)?;
            intent.visit_entity_references_mut(|reference| {
                if let EntityReference::Created(created) = reference {
                    context.checkpoint(bindings.resolve_work_bound(created).ok_or_else(|| custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?).map_err(custody::execution_stop)?;
                    if let Some(id) = bindings.resolve(created) { *reference = EntityReference::Existing(id); }
                }
                Ok::<_, TransactionCommitError>(())
            })?;
        }
        Ok(())
    }).map_err(custody::execution_stop)?.0
}
