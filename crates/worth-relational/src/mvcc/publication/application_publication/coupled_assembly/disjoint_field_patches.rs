//! Compose disjoint field patches of Native-applied participants, preserving all reads.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::transactions::data::{
    AspectFieldPatch, CommitConflict, ConflictClass, EntityMutationIntent, ExistingRecordTarget,
    MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{AspectFieldLocator, AspectKey, CanonicalFieldPath};
type Entity = crate::identity::data::EntityId;
struct Patches {
    first: (usize, usize),
    fields: BTreeSet<(AspectKey, CanonicalFieldPath)>,
    repeated: bool,
    portable: bool,
}

/// Each writer has already passed admission and actual application, including
/// the field owner's single-field-path validation. A canonical field patch can
/// represent their disjoint writes, never two values at one admitted locator.
/// Portable patches use the same installed field lowering and disjoint-aspect
/// composition. Structural changes retain repeated-record C refusal.
pub(super) fn compose(
    execution: &ExecutionRequest<'_, '_>,
    transaction: &mut BranchBoundRelationalTransaction,
    effect: &mut crate::authority::mutation::MutationEffect,
    state: &impl crate::storage::overlay::CurrentPartitionAccess,
    plans: &crate::schema::data::AspectContractPlanCatalog,
) -> Result<(), TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            let _groups_memory = reserve(execution, quote_groups(context, transaction.batches())?)?;
            let Some(groups) = disjoint_groups(context, transaction.batches())? else {
                return Ok(());
            };
            let _copies_memory = reserve(execution, quote_copies(context, transaction.batches())?)?;
            let composed = compose_batches(context, transaction.batches(), &groups, state, plans)?;
            let mut targets = Vec::new();
            targets.try_reserve_exact(groups.len()).map_err(|_| {
                custody::execution_stop(
                    crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                )
            })?;
            targets.extend(
                groups
                    .iter()
                    .filter(|(_, group)| group.repeated)
                    .map(|(entity, _)| *entity),
            );
            crate::authority::mutation::compose_disjoint_publication(
                context,
                execution,
                &mut effect.publication,
                &targets,
            )?;
            transaction
                .restage_coupled_field_batches(composed, execution.allocation_policy())
                .map_err(|stop| TransactionCommitError::conflict(stop.into_conflict()))
        })
        .map_err(custody::execution_stop)?
        .0
}
fn field_patch(intent: &MutationIntent) -> Option<&UpdateEntityFieldsIntent> {
    match intent {
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(patch)) => Some(patch),
        _ => None,
    }
}
fn reserve(
    execution: &ExecutionRequest<'_, '_>,
    bytes: u64,
) -> Result<worth_execution::ExecutionMemoryReservation, TransactionCommitError> {
    worth_execution::ExecutionMemoryReservation::reserve_in_scope(
        crate::authority::commit::pipeline::request_lease(execution),
        bytes,
    )
    .map_err(|cause| custody::execution_stop(crate::execution::denial::lease_denial(cause)))
}
fn quote_groups(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batches: &[WorkerIntentBatch],
) -> Result<u64, TransactionCommitError> {
    let mut bytes = 0u64;
    for batch in batches {
        for intent in &batch.intents {
            checkpoint(context, 1).map_err(custody::execution_stop)?;
            if patch_entity(intent).is_some() {
                bytes = add(
                    bytes,
                    intent
                        .owned_allocation_capacity_bytes()
                        .checked_mul(8)
                        .ok_or_else(overflow)?,
                )?;
                bytes = add(
                    bytes,
                    (std::mem::size_of::<(Entity, Patches)>() + 8 * std::mem::size_of::<usize>())
                        as u64,
                )?;
                for key in field_patch(intent)
                    .into_iter()
                    .flat_map(|p| p.fields.locators())
                {
                    checkpoint(context, key.owned_allocation_capacity_bytes() as u64 + 1)
                        .map_err(custody::execution_stop)?;
                    bytes = add(bytes, key.owned_allocation_capacity_bytes() as u64)?;
                    bytes = add(
                        bytes,
                        (std::mem::size_of::<AspectFieldLocator>()
                            + 8 * std::mem::size_of::<usize>()) as u64,
                    )?;
                }
            }
        }
    }
    Ok(bytes)
}
fn quote_copies(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batches: &[WorkerIntentBatch],
) -> Result<u64, TransactionCommitError> {
    let mut bytes = 0u64;
    let mut count = 0u64;
    for batch in batches {
        bytes = add(bytes, batch.name.len() as u64)?;
        bytes = add(
            bytes,
            batch.partition_key.as_ref().map_or(0, |s| s.len()) as u64,
        )?;
        for intent in &batch.intents {
            checkpoint(context, 1).map_err(custody::execution_stop)?;
            bytes = add(bytes, intent.owned_allocation_capacity_bytes())?;
            count = add(count, 1)?;
        }
    }
    // Merged fields and replacement input coexist until successful restaging.
    // Surviving payload backing then replaces the original input backing.
    let directory = (std::mem::size_of::<MutationIntent>()
        + std::mem::size_of::<Entity>()
        + 8 * std::mem::size_of::<usize>()) as u64;
    let headers = (batches.len() as u64)
        .checked_mul(std::mem::size_of::<WorkerIntentBatch>() as u64)
        .ok_or_else(overflow)?;
    bytes
        .checked_mul(3)
        .and_then(|b| count.checked_mul(directory).and_then(|d| b.checked_add(d)))
        .and_then(|b| b.checked_add(headers))
        .ok_or_else(overflow)
}
fn disjoint_groups(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batches: &[WorkerIntentBatch],
) -> Result<Option<BTreeMap<Entity, Patches>>, TransactionCommitError> {
    let mut groups = BTreeMap::<Entity, Patches>::new();
    let mut repeated = false;
    for (batch_index, batch) in batches.iter().enumerate() {
        for (index, intent) in batch.intents.iter().enumerate() {
            let Some(entity) = patch_entity(intent) else {
                continue;
            };
            let group = groups.entry(entity).or_insert_with(|| Patches {
                first: (batch_index, index),
                fields: BTreeSet::new(),
                repeated: false,
                portable: false,
            });
            let later = group.first != (batch_index, index);
            for key in field_patch(intent)
                .into_iter()
                .flat_map(|p| p.fields.locators())
            {
                checkpoint(context, key.owned_allocation_capacity_bytes() as u64 + 1)
                    .map_err(custody::execution_stop)?;
                if later
                    && group
                        .fields
                        .contains(&(key.aspect().aspect_key().clone(), key.field_path().clone()))
                {
                    return Err(TransactionCommitError::conflict(CommitConflict::new(
                        ConflictClass::ConflictingIntent {
                            target: ExistingRecordTarget::Entity(entity),
                        },
                    )));
                }
            }
            group.fields.extend(
                field_patch(intent)
                    .into_iter()
                    .flat_map(|p| p.fields.locators())
                    .map(|key| (key.aspect().aspect_key().clone(), key.field_path().clone())),
            );
            group.portable |= field_patch(intent).is_none();
            group.repeated |= later;
            repeated |= later;
        }
    }
    Ok(repeated.then_some(groups))
}
fn compose_batches(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batches: &[WorkerIntentBatch],
    groups: &BTreeMap<Entity, Patches>,
    state: &impl crate::storage::overlay::CurrentPartitionAccess,
    plans: &crate::schema::data::AspectContractPlanCatalog,
) -> Result<Vec<WorkerIntentBatch>, TransactionCommitError> {
    let mut fields = BTreeMap::<Entity, BTreeMap<_, _>>::new();
    let mut portable = super::disjoint_aspect_patches::collect(
        context,
        batches,
        |entity| groups[&entity].repeated && groups[&entity].portable,
        state,
        plans,
    )?;
    for batch in batches {
        for intent in &batch.intents {
            if let Some(patch) = field_patch(intent)
                && groups[&patch.entity_id].repeated
                && !groups[&patch.entity_id].portable
            {
                checkpoint(
                    context,
                    intent.owned_allocation_capacity_bytes().saturating_add(1),
                )
                .map_err(custody::execution_stop)?;
                fields
                    .entry(patch.entity_id)
                    .or_default()
                    .extend(patch.fields.iter().map(|(l, v)| (l.clone(), v.clone())));
            }
        }
    }
    let mut composed = Vec::with_capacity(batches.len());
    for (batch_index, batch) in batches.iter().enumerate() {
        let mut intents = Vec::with_capacity(batch.intents.len());
        for (index, intent) in batch.intents.iter().enumerate() {
            checkpoint(
                context,
                intent.owned_allocation_capacity_bytes().saturating_add(1),
            )
            .map_err(custody::execution_stop)?;
            if let Some(entity) = patch_entity(intent)
                && groups[&entity].repeated
                && groups[&entity].portable
            {
                if groups[&entity].first == (batch_index, index) {
                    intents.push(MutationIntent::Entity(
                        EntityMutationIntent::ApplyAspectPatch(
                            crate::transactions::data::ApplyEntityAspectPatchIntent {
                                entity_id: entity,
                                aspect_patch: portable
                                    .remove(&entity)
                                    .expect("first admitted aspect patch"),
                            },
                        ),
                    ));
                }
            } else if let Some(patch) = field_patch(intent)
                && groups[&patch.entity_id].repeated
            {
                if groups[&patch.entity_id].first == (batch_index, index) {
                    intents.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent {
                            entity_id: patch.entity_id,
                            fields: AspectFieldPatch::new(
                                fields
                                    .remove(&patch.entity_id)
                                    .expect("first admitted field patch"),
                            ),
                        },
                    )));
                }
            } else {
                intents.push(intent.clone());
            }
        }
        composed.push(WorkerIntentBatch {
            name: batch.name.clone(),
            partition_key: batch.partition_key.clone(),
            worker_local_only: batch.worker_local_only,
            intents,
        });
    }
    Ok(composed)
}
fn add(left: u64, right: u64) -> Result<u64, TransactionCommitError> {
    left.checked_add(right).ok_or_else(overflow)
}
fn overflow() -> TransactionCommitError {
    custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}

pub(super) fn patch_entity(intent: &MutationIntent) -> Option<Entity> {
    match intent {
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(p)) => Some(p.entity_id),
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(p)) => Some(p.entity_id),
        _ => None,
    }
}
