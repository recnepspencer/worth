//! Keep actual prefix allocations and their symbolic transaction references together.
use crate::mvcc::publication::application_publication::custody;
use contracts::execution::request_stage::checkpoint;
use contracts::symbols::data::client_key::{ClientKey, ClientKeyOwnerPort as _};
use contracts::symbols::data::string_interner::StringInterner;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::EntityReference;
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_relational_contracts::internal_port as contracts;
use contracts::lineage::data::CommitNamingPlan;

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
                    retain_reference(context, bindings, reference)
                })?;
            }
            Ok(())
        })
        .map_err(custody::execution_stop)?
        .0
}

/// The same retention for a part's naming plan, whose every key then takes
/// its text: commit finalization resolves a plan's targets by their key text
/// (the commands are normalized to symbols, the plan never is), so a retained
/// prefix endpoint must not keep the interned key its binding carries.
pub fn retain_naming(
    execution: &ExecutionRequest<'_, '_>,
    bindings: &CommitCreatedEntityBindings,
    symbols: &StringInterner,
    plan: &mut CommitNamingPlan,
) -> Result<(), TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            super::part_naming::visit_endpoints_mut(plan, |reference| {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                retain_reference(context, bindings, reference)
            })?;
            super::part_naming::visit_client_keys_mut(plan, |key| {
                if key.as_raw_str().is_some() {
                    return Ok(());
                }
                let text = key.resolve_with_interner(symbols).ok_or_else(|| {
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "coupled naming key has no actual Native symbol meaning",
                    )
                })?;
                checkpoint(context, text.len() as u64 + 1).map_err(custody::execution_stop)?;
                *key = ClientKey::raw(text);
                Ok::<_, TransactionCommitError>(())
            })
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
