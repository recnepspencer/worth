//! Decide whether a discarded continued round changed canonical record meaning.
use super::super::super::custody;
use super::super::refusals::reject;
use crate::authority::mutation::MutationApplyOutcome;
use crate::execution::request_stage::checkpoint;
use crate::identity::data::EntityId;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::publication::patch::data::preserves_application;
use crate::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::transactions::data::intents::mutation_intent::{
    EntityMutationIntent, MutationIntent, RelationMutationIntent,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::WorkerIntentBatch;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;

/// Only these commands can leave a record structurally as it was. Creation,
/// deletion, replacement and endpoint changes are effects by their application.
pub(super) fn patches_in_place(intent: &MutationIntent) -> bool {
    matches!(
        intent,
        MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(_) | EntityMutationIntent::ApplyAspectPatch(_)
        ) | MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_))
    )
}

pub(super) fn commands_are_in_place(
    execution: &ExecutionRequest<'_, '_>,
    batches: &[WorkerIntentBatch],
) -> Result<bool, TransactionCommitError> {
    let allocation = execution.allocation_policy();
    execution
        .run_native_with_checkpoint(|context| {
            let mut raw_keys = std::collections::BTreeSet::new();
            for intent in batches.iter().flat_map(|batch| &batch.intents) {
                checkpoint(
                    context,
                    intent.owned_allocation_capacity_bytes().saturating_add(1),
                )
                .map_err(custody::execution_stop)?;
                intent
                    .collect_raw_client_keys(&mut raw_keys, allocation)
                    .map_err(|cause| {
                        TransactionCommitError::conflict(
                            crate::mvcc::transaction::staging::RelationalTransactionStagingDenial::from(
                                cause,
                            )
                            .into_conflict(),
                        )
                    })?;
                if !patches_in_place(intent) || !raw_keys.is_empty() {
                    return Ok(false);
                }
            }
            Ok::<_, TransactionCommitError>(true)
        })
        .map_err(custody::execution_stop)?
        .0
}

pub(super) fn preserves_applied_records(
    execution: &ExecutionRequest<'_, '_>,
    outcome: &MutationApplyOutcome,
    reserved_symbols: usize,
    observed_symbols: usize,
) -> Result<bool, TransactionCommitError> {
    if !outcome.effect.adjacency.deltas.is_empty()
        || outcome
            .created_entities
            .allocated_identities()
            .next()
            .is_some()
        || outcome.created_relations.iter().next().is_some()
        || observed_symbols != reserved_symbols
    {
        return Ok(false);
    }
    let publication = &outcome.effect.publication;
    execution
        .run_native_with_checkpoint(|context| {
            for fragment in &publication.patch_fragments {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                if !preserves_application(
                    fragment.structural_change,
                    fragment.contains_opaque_aspect,
                    &fragment.semantic_changes,
                    &fragment.published_patch,
                    fragment.relation_birth.as_ref(),
                ) {
                    return Ok(false);
                }
            }
            for record in &publication.changed_records {
                checkpoint(context, publication.patch_fragments.len() as u64 + 1)
                    .map_err(custody::execution_stop)?;
                if !publication
                    .patch_fragments
                    .iter()
                    .any(|fragment| &fragment.target == record)
                {
                    return Ok(false);
                }
            }
            Ok::<_, TransactionCommitError>(true)
        })
        .map_err(custody::execution_stop)?
        .0
}

pub(super) fn probe_commands(
    batches: &[WorkerIntentBatch],
    probe: u8,
    source: EntityId,
) -> Result<Vec<WorkerIntentBatch>, TransactionCommitError> {
    let mut round = batches.to_vec();
    if probe != 7 {
        for batch in &mut round {
            batch.intents.retain(patches_in_place);
        }
        if round.iter().all(|batch| batch.intents.is_empty()) {
            return Err(reject(Cause::CorruptPublication, source));
        }
    }
    if probe == 9 {
        for intent in round.iter_mut().flat_map(|batch| &mut batch.intents) {
            if let MutationIntent::Entity(EntityMutationIntent::UpdateFields(update)) = intent {
                update.fields = AspectFieldPatch::new(
                    update
                        .fields
                        .iter()
                        .map(|(locator, value)| {
                            let moved = match value {
                                AspectValue::UInt64(value) => {
                                    AspectValue::UInt64(value.wrapping_add(1))
                                }
                                other => other.clone(),
                            };
                            (locator.clone(), moved)
                        })
                        .collect(),
                );
            }
        }
    }
    Ok(round)
}
