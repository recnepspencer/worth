//! Decide whether a discarded continued round changed canonical record meaning.
use super::super::super::custody;
use super::super::refusals::reject;
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::record_ids::EntityId;
use contracts::mvcc::transaction::staging::RelationalTransactionStagingDenialOwnerPort as _;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::publication::patch::data::authoritative_patch_envelope::preserves_application;
use contracts::transactions::data::aspect_field_patch::AspectFieldPatch;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntentOwnerPort as _;
use contracts::transactions::data::intents::mutation_intent::RelationMutationIntent;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::WorkerIntentBatch;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::authority::mutation::execution::MutationApplyOutcome;

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

pub fn commands_are_in_place(
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
                            contracts::mvcc::transaction::staging::RelationalTransactionStagingDenial::from(
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

pub fn preserves_applied_records(
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
                    &fragment.authoritative_patch,
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

pub fn probe_commands(
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
