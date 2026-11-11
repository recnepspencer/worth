use crate::config::data::CascadeDeletePolicy;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::WorkingState;
use crate::storage::overlay::{CurrentPartitionAccess, PartitionState};
use crate::storage::substrate::{
    EntityExtra, EntityRecordKind, RecordKind, RelationEndpoints, RelationExtra, RelationRecordKind,
};
use crate::transactions::data::{
    CommitConflict, ConflictClass, EntityCascadeDeleteMissingState,
    MutationStateInconsistencyEvidence,
};

use super::outcomes::{MutationOutcome, RecordMutation};
use super::{AdjacencyDelta, AdjacencyDeltaKind};

mod relation_lifecycle;

pub(super) use relation_lifecycle::{delete_relation, retain_relation_dangling_for_audit};

pub(super) fn allocate_entity_with_extra(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    allocations: &mut dyn crate::runtime::MutationRecordAllocations,
    version_id: crate::identity::data::VersionId,
    partition_id: PartitionId,
    kind_id: KindId,
    extra: EntityExtra,
) -> Result<EntityId, crate::transactions::data::TransactionCommitError> {
    let (slot, generation) = allocate_record::<EntityRecordKind>(
        execution,
        state,
        allocations,
        crate::history::data::RecordAllocationClass::Entity,
        partition_id,
        kind_id,
        version_id,
        extra,
    )?;
    let partition = ensure_partition_state(execution, state, partition_id)?;
    let adjacency_policy = partition.adjacency_policy.clone();
    partition
        .adjacency
        .clear_slot_for_request(slot, &adjacency_policy, execution)
        .map_err(execution_denial)?;
    partition
        .reverse_adjacency
        .clear_slot_for_request(slot, &adjacency_policy, execution)
        .map_err(execution_denial)?;
    // Empty membership is still stored adjacency truth. Record initialization
    // and slot reuse must publish both changed adjacency axes in the journal.
    state
        .mark_adjacency_slot_touched_for_request(partition_id, slot, execution)
        .map_err(execution_denial)?;
    state
        .mark_reverse_adjacency_slot_touched_for_request(partition_id, slot, execution)
        .map_err(execution_denial)?;
    let entity_id = EntityId::new(partition_id, slot as u64, generation);
    allocations
        .record(crate::transactions::data::RecordRef::Entity(entity_id))
        .map_err(record_allocation_conflict)?;
    Ok(entity_id)
}

pub(super) fn allocate_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    allocations: &mut dyn crate::runtime::MutationRecordAllocations,
    version_id: crate::identity::data::VersionId,
    partition_id: PartitionId,
    kind_id: KindId,
    source: EntityId,
    target: EntityId,
    mut extra: RelationExtra,
) -> Result<RelationId, crate::transactions::data::TransactionCommitError> {
    extra.endpoints = Some(RelationEndpoints { source, target });
    let (slot, generation) = allocate_record::<RelationRecordKind>(
        execution,
        state,
        allocations,
        crate::history::data::RecordAllocationClass::Relation,
        partition_id,
        kind_id,
        version_id,
        extra,
    )?;
    let relation_id = RelationId::new(partition_id, slot as u64, generation);
    allocations
        .record(crate::transactions::data::RecordRef::Relation(relation_id))
        .map_err(record_allocation_conflict)?;
    Ok(relation_id)
}

mod entity_deletion;
pub(super) use entity_deletion::delete_entity_with_cascade;

pub(crate) use adjacency::apply_adjacency_deltas_for_request;
mod adjacency;
fn ensure_partition_state<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &'a mut WorkingState,
    partition_id: PartitionId,
) -> Result<&'a mut PartitionState, crate::transactions::data::TransactionCommitError> {
    state
        .get_partition_mut_for_request(partition_id, execution)
        .map_err(execution_denial)
}
fn execution_denial(
    cause: crate::execution::RelationalExecutionDenialCause,
) -> crate::transactions::data::TransactionCommitError {
    crate::transactions::data::TransactionCommitError::execution(cause.into())
}
fn map_denial(
    cause: worth_execution::ExecutionOrderedMapDenial,
) -> crate::transactions::data::TransactionCommitError {
    use crate::transactions::data::TransactionCommitError as Error;
    match cause {
        worth_execution::ExecutionOrderedMapDenial::Request(cause) => {
            Error::execution(cause.into())
        }
        worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => Error::execution(
            crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow.into(),
        ),
        worth_execution::ExecutionOrderedMapDenial::Allocation(denial) => {
            crate::execution::allocation_commit_error(Error::conflict(CommitConflict::new(
                ConflictClass::ExecutionAllocationDenied { denial },
            )))
        }
    }
}

fn allocate_record<K: RecordKind>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    allocations: &mut dyn crate::runtime::MutationRecordAllocations,
    class: crate::history::data::RecordAllocationClass,
    partition_id: PartitionId,
    kind_id: KindId,
    version_id: crate::identity::data::VersionId,
    extra: K::Extra,
) -> Result<(usize, u32), crate::transactions::data::TransactionCommitError> {
    let partition = ensure_partition_state(execution, state, partition_id)?;
    let arena = K::arena_mut(partition);
    let reserved = allocations
        .reserve(class, partition_id)
        .map_err(record_allocation_conflict)?;
    arena
        .write_reserved_slot_for_request(
            execution,
            crate::storage::substrate::SlotInit {
                partition_id,
                kind_id,
                version_id,
                extra,
            },
            reserved.slot,
            reserved.generation,
        )
        .map_err(|denial| {
            let detail = match denial {
                crate::storage::substrate::MetadataWriteDenial::Request(cause) => {
                    return crate::transactions::data::TransactionCommitError::execution(
                        cause.into(),
                    )
                }
                crate::storage::substrate::MetadataWriteDenial::Invalid(detail) => detail,
            };
            record_allocation_conflict(
                crate::transactions::data::RecordAllocationDenial::ArenaWriteDenied {
                    class,
                    partition_id,
                    slot: reserved.slot,
                    detail: detail.to_owned(),
                },
            )
            .into()
        })?;
    Ok((reserved.slot, reserved.generation))
}

fn record_allocation_conflict(
    denial: crate::transactions::data::RecordAllocationDenial,
) -> CommitConflict {
    CommitConflict::new(ConflictClass::RecordAllocationDenied { denial })
}

fn mutation_state_inconsistency(
    detail: impl Into<String>,
    evidence: MutationStateInconsistencyEvidence,
) -> CommitConflict {
    CommitConflict::new(ConflictClass::MutationStateInconsistency {
        detail: detail.into(),
        evidence,
    })
}

fn entity_cascade_delete_missing(
    entity_id: EntityId,
    missing: EntityCascadeDeleteMissingState,
) -> MutationStateInconsistencyEvidence {
    MutationStateInconsistencyEvidence::EntityCascadeDelete { entity_id, missing }
}
