//! Compose submitted disjoint aspect grammars; final C admission remains authoritative.
use super::patch_target::patch_entity;
use crate::mvcc::publication::application_publication::custody;
use contracts::execution::request_stage::checkpoint;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntentOwnerPort as _;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::ExistingRecordTarget;
use contracts::transactions::data::primitives::WorkerIntentBatch;
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AspectKey, FieldKey, PortableAspectPatchOperation as Operation, PortableRecordAspectPatch,
};
use worth_relational_contracts::internal_port as contracts;
type Entity = contracts::identity::data::record_ids::EntityId;
#[derive(Default)]
struct Patches {
    operations: BTreeMap<AspectKey, Operation>,
    fields: BTreeSet<(AspectKey, FieldKey)>,
}

/// Called only for repeated records with a portable participant. Pure field
/// groups keep their original command representation and allocation path.
pub(super) fn collect(
    execution: &ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batches: &[WorkerIntentBatch],
    selected: impl Fn(Entity) -> bool,
    state: &impl worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess,
    plans: &contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
) -> Result<BTreeMap<Entity, PortableRecordAspectPatch>, TransactionCommitError> {
    let mut groups = BTreeMap::<Entity, Patches>::new();
    for batch in batches {
        for intent in &batch.intents {
            checkpoint(context, 1).map_err(custody::execution_stop)?;
            let Some(entity) = patch_entity(intent).filter(|entity| selected(*entity)) else {
                continue;
            };
            checkpoint(
                context,
                intent.owned_allocation_capacity_bytes().saturating_add(1),
            )
            .map_err(custody::execution_stop)?;
            let lowered;
            let operations = match intent {
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(fields)) => {
                    lowered = worth_relational_mutation::internal_port::authority::mutation::intents::field_authoring_candidate::lower_entity(
                        execution, fields, state, plans,
                    )?;
                    lowered.operations()
                }
                MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(patch)) => {
                    patch.aspect_patch.operations()
                }
                _ => unreachable!("selected aspect mutation"),
            };
            let group = groups.entry(entity).or_default();
            for operation in operations {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                let key = operation.basis().key();
                if let Operation::PatchFields {
                    selected_fields, ..
                } = operation
                {
                    for field in selected_fields {
                        checkpoint(
                            context,
                            (key.as_str().len() + field.as_str().len() + 1) as u64,
                        )
                        .map_err(custody::execution_stop)?;
                        if !group.fields.insert((key.clone(), field.clone())) {
                            return Err(conflict(entity));
                        }
                    }
                }
                match group.operations.get_mut(key) {
                    None => {
                        group.operations.insert(key.clone(), operation.clone());
                    }
                    Some(prior) => merge(entity, prior, operation)?,
                }
            }
        }
    }
    Ok(groups
        .into_iter()
        .map(|(entity, group)| {
            (
                entity,
                PortableRecordAspectPatch::new(group.operations.into_values()),
            )
        })
        .collect())
}
fn merge(
    entity: Entity,
    prior: &mut Operation,
    next: &Operation,
) -> Result<(), TransactionCommitError> {
    match (prior, next) {
        (
            Operation::PatchFields {
                basis: a,
                selected_fields: selected,
                field_sets: sets,
                field_clears: clears,
            },
            Operation::PatchFields {
                basis: b,
                selected_fields,
                field_sets,
                field_clears,
            },
        ) if a == b => {
            selected.extend(selected_fields.iter().cloned());
            sets.extend(field_sets.iter().cloned());
            clears.extend(field_clears.iter().cloned());
            Ok(())
        }
        _ => Err(conflict(entity)),
    }
}
fn conflict(entity: Entity) -> TransactionCommitError {
    TransactionCommitError::conflict(CommitConflict::new(ConflictClass::ConflictingIntent {
        target: ExistingRecordTarget::Entity(entity),
    }))
}
