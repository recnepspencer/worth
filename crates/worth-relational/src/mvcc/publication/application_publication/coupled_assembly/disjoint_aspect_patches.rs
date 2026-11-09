//! Compose submitted disjoint aspect grammars; final C admission remains authoritative.
use super::disjoint_field_patches::patch_entity;
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::transactions::data::{
    CommitConflict, ConflictClass, EntityMutationIntent, ExistingRecordTarget, MutationIntent,
    WorkerIntentBatch,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{
    AspectKey, FieldKey, PortableAspectPatchOperation as Operation, PortableRecordAspectPatch,
};
type Entity = crate::identity::data::EntityId;
#[derive(Default)]
struct Patches {
    operations: BTreeMap<AspectKey, Operation>,
    fields: BTreeSet<(AspectKey, FieldKey)>,
}

/// Called only for repeated records with a portable participant. Pure field
/// groups keep their original command representation and allocation path.
pub(super) fn collect(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batches: &[WorkerIntentBatch],
    selected: impl Fn(Entity) -> bool,
    state: &impl crate::storage::overlay::CurrentPartitionAccess,
    plans: &crate::schema::data::AspectContractPlanCatalog,
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
            let patch = match intent {
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(fields)) => {
                    crate::authority::mutation::intents::field_authoring_candidate::lower_entity(
                        fields, state, plans,
                    )
                    .map_err(TransactionCommitError::conflict)?
                }
                MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(patch)) => {
                    patch.aspect_patch.clone()
                }
                _ => unreachable!("selected aspect mutation"),
            };
            let group = groups.entry(entity).or_default();
            for operation in patch.operations() {
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
