//! Align multi-round coupled record allocations to the final sorted commit plan.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::history::data::{CanonicalRecordAllocation, RecordAllocationOrigin};
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::mvcc::transaction::normalized_plan::{reserve_copy, slots};
use crate::runtime::{metered_binary_search, metered_false_mask, metered_heapsort};
use crate::symbols::data::ClientKey;
use crate::transactions::data::intents::mutation_intent::{EntityMutationIntent, MutationIntent};
use worth_relational_contracts::internal_port::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use worth_relational_contracts::internal_port::transactions::data::outcomes::created_relation_bindings::{
    BorrowedEntityRef, CommitCreatedRelationBindings,
};
use crate::transactions::data::primitives::{EntityReference, RecordRef};
use crate::transactions::data::intents::mutation_intent::CreateIntent;

pub(super) fn align_to_canonical_plan(
    execution: &ExecutionRequest<'_, '_>,
    commands: &crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan,
    mutation: &mut AppliedMutationCore,
) -> Result<(), TransactionCommitError> {
    let canonical_len = mutation.identities.records.canonical().len();
    if canonical_len == 0 {
        return Ok(());
    }
    let replacement_len = mutation
        .outcome
        .effect
        .publication
        .entity_replacements
        .len();
    // Exact byte-for-byte funding for every flat scratch Vec across both stages:
    // ordered, replacements, replacement_used, by_record, used, reordered.
    let scratch_bytes = slots::<RecordRef>(canonical_len).and_then(|ordered_bytes| {
        let by_record_bytes = slots::<(RecordRef, RecordAllocationOrigin)>(canonical_len)?;
        let used_bytes = slots::<bool>(canonical_len)?;
        let reorder_bytes = slots::<CanonicalRecordAllocation>(canonical_len)?;
        let repl_bytes = slots::<(EntityId, EntityId)>(replacement_len)?;
        let repl_used_bytes = slots::<bool>(replacement_len)?;
        ordered_bytes
            .checked_add(by_record_bytes)?
            .checked_add(used_bytes)?
            .checked_add(reorder_bytes)?
            .checked_add(repl_bytes)?
            .checked_add(repl_used_bytes)
    });
    let _scratch_memory = reserve_copy(execution, scratch_bytes)?;
    execution
        .run_native_with_checkpoint(|context| {
            let ordered = collect_ordered_records(
                context,
                commands,
                mutation,
                canonical_len,
                replacement_len,
            )?;
            mutation
                .identities
                .records
                .reorder_canonical_for_plan_metered(
                    &ordered,
                    |units| checkpoint(context, units).map_err(custody::execution_stop),
                    crate::transactions::data::outcomes::transaction_commit_error::origin_denial,
                )
        })
        .map_err(custody::execution_stop)?
        .0
}

fn collect_ordered_records(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    commands: &crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan,
    mutation: &AppliedMutationCore,
    canonical_len: usize,
    replacement_len: usize,
) -> Result<Vec<RecordRef>, TransactionCommitError> {
    let mut ordered: Vec<RecordRef> = Vec::new();
    ordered.try_reserve_exact(canonical_len).map_err(|_| {
        crate::transactions::data::outcomes::transaction_commit_error::origin_denial("coupled plan ordered scratch reservation failed")
    })?;
    let mut replacements: Vec<(EntityId, EntityId)> = Vec::new();
    replacements
        .try_reserve_exact(replacement_len)
        .map_err(|_| crate::transactions::data::outcomes::transaction_commit_error::origin_denial("coupled plan replacement scratch failed"))?;
    for &(old, new) in &mutation.outcome.effect.publication.entity_replacements {
        checkpoint(context, 1).map_err(custody::execution_stop)?;
        replacements.push((old, new));
    }
    let mut step = |units| checkpoint(context, units).map_err(custody::execution_stop);
    metered_heapsort(&mut replacements, |a, b| a.0.cmp(&b.0), &mut step)?;
    let mut replacement_used = metered_false_mask(
        replacement_len,
        &mut step,
        crate::transactions::data::outcomes::transaction_commit_error::origin_denial,
        "coupled plan replacement mask failed",
    )?;

    let entities = &mutation.outcome.created_entities;
    let relations = &mutation.outcome.created_relations;

    let push_record =
        |ordered: &mut Vec<RecordRef>, record: RecordRef| -> Result<(), TransactionCommitError> {
            if ordered.len() >= canonical_len {
                return Err(crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "coupled canonical plan produced more records than allocated",
                ));
            }
            ordered.push(record);
            Ok(())
        };

    for intent in &commands.merged_intents {
        checkpoint(context, 1).map_err(custody::execution_stop)?;
        match intent {
            MutationIntent::Create(CreateIntent::Entity(spec)) => {
                let id = resolve_entity_metered(
                    context,
                    entities,
                    spec.partition_id,
                    spec.kind_id,
                    &spec.client_key,
                    "coupled canonical plan entity missing allocated identity",
                )?;
                push_record(&mut ordered, RecordRef::Entity(id))?;
            }
            MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
                let id = resolve_entity_metered(
                    context,
                    entities,
                    spec.partition_id,
                    spec.kind_id,
                    &spec.client_key,
                    "coupled canonical plan aspect entity missing allocated identity",
                )?;
                push_record(&mut ordered, RecordRef::Entity(id))?;
            }
            MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                for key in &spec.client_keys {
                    let id = resolve_entity_metered(
                        context,
                        entities,
                        spec.partition_id,
                        spec.kind_id,
                        key,
                        "coupled canonical plan bulk entity missing allocated identity",
                    )?;
                    push_record(&mut ordered, RecordRef::Entity(id))?;
                }
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                let id = take_replacement_metered(
                    context,
                    &replacements,
                    &mut replacement_used,
                    spec.entity_id,
                )?;
                push_record(&mut ordered, RecordRef::Entity(id))?;
            }
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                let record = resolve_relation_metered(
                    context,
                    entities,
                    relations,
                    spec.partition_id,
                    spec.kind_id,
                    &spec.client_key,
                    &spec.source,
                    &spec.target,
                    "coupled canonical plan relation missing allocated identity",
                )?;
                push_record(&mut ordered, record)?;
            }
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                let record = resolve_relation_metered(
                    context,
                    entities,
                    relations,
                    spec.partition_id,
                    spec.kind_id,
                    &spec.client_key,
                    &spec.source,
                    &spec.target,
                    "coupled canonical plan aspect relation missing allocated identity",
                )?;
                push_record(&mut ordered, record)?;
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                for (key, (source, target)) in spec.client_keys.iter().zip(&spec.endpoints) {
                    let record = resolve_relation_metered(
                        context,
                        entities,
                        relations,
                        spec.partition_id,
                        spec.kind_id,
                        key,
                        source,
                        target,
                        "coupled canonical plan bulk relation missing allocated identity",
                    )?;
                    push_record(&mut ordered, record)?;
                }
            }
            MutationIntent::Entity(_)
            | MutationIntent::Relation(_)
            | MutationIntent::Materialization(_) => {}
        }
    }
    Ok(ordered)
}

fn resolve_entity_metered(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    entities: &CommitCreatedEntityBindings,
    partition_id: PartitionId,
    kind_id: KindId,
    client_key: &ClientKey,
    missing: &'static str,
) -> Result<EntityId, TransactionCommitError> {
    let work = entities
        .resolve_parts_work_bound(client_key)
        .ok_or_else(|| crate::transactions::data::outcomes::transaction_commit_error::origin_denial("coupled entity resolve work bound overflow"))?;
    checkpoint(context, work.max(1)).map_err(custody::execution_stop)?;
    entities
        .resolve_parts(partition_id, kind_id, client_key)
        .ok_or_else(|| crate::transactions::data::outcomes::transaction_commit_error::origin_denial(missing))
}

#[allow(clippy::too_many_arguments)]
fn resolve_relation_metered(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    entities: &CommitCreatedEntityBindings,
    relations: &CommitCreatedRelationBindings,
    partition_id: PartitionId,
    kind_id: KindId,
    client_key: &ClientKey,
    source: &EntityReference,
    target: &EntityReference,
    missing: &'static str,
) -> Result<RecordRef, TransactionCommitError> {
    let raw_source = BorrowedEntityRef::from_ref(source);
    let raw_target = BorrowedEntityRef::from_ref(target);
    let work = relations
        .resolve_parts_work_bound(client_key, raw_source, raw_target)
        .ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial("coupled relation resolve work bound overflow")
        })?;
    checkpoint(context, work.max(1)).map_err(custody::execution_stop)?;
    if let Some(id) =
        relations.resolve_parts(partition_id, kind_id, client_key, raw_source, raw_target)
    {
        return Ok(RecordRef::Relation(id));
    }
    let lowered_source = match raw_source {
        BorrowedEntityRef::Created(p, k, key) => {
            let w = entities.resolve_parts_work_bound(key).ok_or_else(|| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial("coupled source entity work bound overflow")
            })?;
            checkpoint(context, w.max(1)).map_err(custody::execution_stop)?;
            entities
                .resolve_parts(p, k, key)
                .map_or(raw_source, BorrowedEntityRef::Existing)
        }
        existing => existing,
    };
    let lowered_target = match raw_target {
        BorrowedEntityRef::Created(p, k, key) => {
            let w = entities.resolve_parts_work_bound(key).ok_or_else(|| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial("coupled target entity work bound overflow")
            })?;
            checkpoint(context, w.max(1)).map_err(custody::execution_stop)?;
            entities
                .resolve_parts(p, k, key)
                .map_or(raw_target, BorrowedEntityRef::Existing)
        }
        existing => existing,
    };
    let lowered_work = relations
        .resolve_parts_work_bound(client_key, lowered_source, lowered_target)
        .ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial("coupled lowered relation resolve work bound overflow")
        })?;
    checkpoint(context, lowered_work.max(1)).map_err(custody::execution_stop)?;
    relations
        .resolve_parts(
            partition_id,
            kind_id,
            client_key,
            lowered_source,
            lowered_target,
        )
        .map(RecordRef::Relation)
        .ok_or_else(|| crate::transactions::data::outcomes::transaction_commit_error::origin_denial(missing))
}

fn take_replacement_metered(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    replacements: &[(EntityId, EntityId)],
    used: &mut [bool],
    old_id: EntityId,
) -> Result<EntityId, TransactionCommitError> {
    let mut step = |units| checkpoint(context, units).map_err(custody::execution_stop);
    let Some(mut idx) =
        metered_binary_search(replacements, |entry| entry.0.cmp(&old_id), &mut step)?
    else {
        return Err(crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
            "coupled canonical plan replacement missing allocated identity",
        ));
    };
    while idx > 0 {
        step(1)?;
        if replacements[idx - 1].0 != old_id {
            break;
        }
        idx -= 1;
    }
    while idx < replacements.len() {
        step(1)?;
        if replacements[idx].0 != old_id {
            break;
        }
        if !used[idx] {
            used[idx] = true;
            return Ok(replacements[idx].1);
        }
        idx += 1;
    }
    Err(crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
        "coupled canonical plan replacement missing allocated identity",
    ))
}
