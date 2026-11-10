//! Entity retirement publishes its deletion patch and cascades to attached relations.
use super::super::intents::entity_authoritative_deletion_patch::plan_entity_authoritative_deletion_patch;
use super::super::outcomes::MutationAspectPatch;
use super::*;
use crate::storage::substrate::StorageAllocation;
struct EntityDeletionInput {
    kind_id: KindId,
    original: StorageAllocation<EntityExtra>,
}
fn input(state: &WorkingState, entity: EntityId) -> Result<EntityDeletionInput, CommitConflict> {
    let view = state
        .get_current_partition(entity.partition_id)
        .and_then(|partition| partition.entity_arena().get(&entity))
        .ok_or_else(|| {
            mutation_state_inconsistency(
                "entity delete requires an existing slot after stale-target validation",
                entity_cascade_delete_missing(entity, EntityCascadeDeleteMissingState::Slot),
            )
        })?;
    let kind_id = view.kind_id().ok_or_else(|| {
        mutation_state_inconsistency(
            "entity delete requires a retained kind id after stale-target validation",
            entity_cascade_delete_missing(entity, EntityCascadeDeleteMissingState::KindId),
        )
    })?;
    Ok(EntityDeletionInput {
        kind_id,
        original: view.retained_extra(),
    })
}
pub(in crate::authority::mutation) fn delete_entity_with_cascade(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    version_id: crate::identity::data::VersionId,
    entity_id: EntityId,
    schema_registry: &RelationalSchemaRegistry,
    default_cascade_delete_policy: CascadeDeletePolicy,
    outcome: &mut MutationOutcome,
) -> Result<(), crate::transactions::data::TransactionCommitError> {
    let input = input(state, entity_id)?;
    let declaration = schema_registry
        .entity_registration(input.kind_id)
        .map_err(|error| {
            CommitConflict::new(ConflictClass::KindSchemaMismatch {
                detail: format!("entity delete requires registered aspect contracts: {error:?}"),
            })
        })?;
    let patch = plan_entity_authoritative_deletion_patch(
        input.original.authoritative_aspect_state(),
        &declaration.aspect_contract_declarations,
    )
    .map_err(|denial| {
        CommitConflict::new(ConflictClass::EntityAuthoritativeAspectStateDenied {
            kind_id: input.kind_id,
            denial,
        })
    })?
    .map(MutationAspectPatch::from_ordinary);
    retire(
        execution,
        state,
        version_id,
        entity_id,
        schema_registry,
        default_cascade_delete_policy,
        outcome,
        input,
        patch,
    )
}
fn retire(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    version_id: crate::identity::data::VersionId,
    entity_id: EntityId,
    schema_registry: &RelationalSchemaRegistry,
    default_cascade_delete_policy: CascadeDeletePolicy,
    outcome: &mut MutationOutcome,
    input: EntityDeletionInput,
    authoritative_patch: Option<MutationAspectPatch>,
) -> Result<(), crate::transactions::data::TransactionCommitError> {
    let slot = entity_id.slot_index();
    state
        .mark_entity_slot_touched_for_request(entity_id.partition_id, slot, execution)
        .map_err(execution_denial)?;
    let partition = state
        .get_partition_mut_for_request(entity_id.partition_id, execution)
        .map_err(execution_denial)?;
    let EntityDeletionInput { kind_id, original } = input;
    let original = Some(original);
    partition
        .entity_arena
        .retire_for_request(execution, slot, version_id)
        .map_err(|cause| {
            crate::transactions::data::TransactionCommitError::execution(cause.into())
        })?
        .ok_or_else(|| {
            mutation_state_inconsistency(
                "entity retirement lost its materialized slot",
                entity_cascade_delete_missing(entity_id, EntityCascadeDeleteMissingState::Slot),
            )
        })?;
    outcome.record_change(
        execution,
        RecordMutation::EntityDeleted {
            entity_id,
            kind_id,
            authoritative_patch,
            old_authoritative_aspect_state: original,
        },
    )?;

    let mut attached = worth_execution::ExecutionOrderedMap::default();
    if let Some(adjacency) = partition.adjacency.get(slot) {
        for relation in adjacency.current_ids() {
            attached
                .insert_cloned_native_for_request(relation, &(), *execution)
                .map_err(map_denial)?;
        }
    }
    if let Some(reverse_adjacency) = partition.reverse_adjacency.get(slot) {
        for relation in reverse_adjacency.current_ids() {
            attached
                .insert_cloned_native_for_request(relation, &(), *execution)
                .map_err(map_denial)?;
        }
    }
    for &relation_id in attached.keys() {
        let cascade_policy = state
            .get_current_partition(relation_id.partition_id)
            .and_then(|partition| {
                partition
                    .relation_arena()
                    .get(&relation_id)
                    .and_then(|slot| slot.kind_id())
            })
            .and_then(|kind_id| {
                schema_registry
                    .relation_registration(kind_id)
                    .ok()
                    .map(|registration| registration.cascade_delete_policy)
            })
            .unwrap_or(default_cascade_delete_policy);
        match cascade_policy {
            CascadeDeletePolicy::CascadeDeleteRelations => {
                delete_relation(execution, state, version_id, relation_id, outcome)?;
            }
            CascadeDeletePolicy::RetainDanglingForAudit => {
                retain_relation_dangling_for_audit(
                    execution,
                    state,
                    version_id,
                    relation_id,
                    outcome,
                )?;
            }
        }
    }
    Ok(())
}
