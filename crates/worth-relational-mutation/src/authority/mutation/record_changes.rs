use crate::contracts::config::data::policies::CascadeDeletePolicy;
use crate::contracts::identity::data::primitives::{KindId, PartitionId};
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::schema::data::RelationalSchemaRegistry;
use crate::contracts::transactions::data::outcomes::commit_conflict::{CommitConflict, CommitConflictOwnerPort as _};
use crate::contracts::transactions::data::outcomes::conflict_class::mutation_state_inconsistency::{EntityCascadeDeleteMissingState, MutationStateInconsistencyEvidence};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use worth_relational_storage::internal_port::storage::substrate::record_arena::kinds::{EntityRecordKind, RecordKind, RelationRecordKind};
use worth_relational_storage::internal_port::storage::substrate::record_arena::values::extra::{EntityExtra, RelationExtra};
use worth_relational_storage::internal_port::storage::substrate::record_arena::values::RelationEndpoints;

use super::outcomes::{MutationOutcome, RecordMutation};
use super::{AdjacencyDelta, AdjacencyDeltaKind};

pub mod relation_lifecycle;

pub(super) use relation_lifecycle::{delete_relation, retain_relation_dangling_for_audit};

pub(super) fn allocate_entity_with_extra(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    allocations: &mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_batch::MutationRecordAllocations,
    version_id: crate::contracts::identity::data::primitives::VersionId,
    partition_id: PartitionId,
    kind_id: KindId,
    extra: EntityExtra,
) -> Result<
    EntityId,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
    let (slot, generation) = allocate_record::<EntityRecordKind>(
        execution,
        state,
        allocations,
        crate::contracts::history::data::record_allocation_class::RecordAllocationClass::Entity,
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
        .record(crate::contracts::transactions::data::primitives::RecordRef::Entity(entity_id))
        .map_err(record_allocation_conflict)?;
    Ok(entity_id)
}

pub(super) fn allocate_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    allocations: &mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_batch::MutationRecordAllocations,
    version_id: crate::contracts::identity::data::primitives::VersionId,
    partition_id: PartitionId,
    kind_id: KindId,
    source: EntityId,
    target: EntityId,
    mut extra: RelationExtra,
) -> Result<
    RelationId,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
    extra.endpoints = Some(RelationEndpoints { source, target });
    let (slot, generation) = allocate_record::<RelationRecordKind>(
        execution,
        state,
        allocations,
        crate::contracts::history::data::record_allocation_class::RecordAllocationClass::Relation,
        partition_id,
        kind_id,
        version_id,
        extra,
    )?;
    let relation_id = RelationId::new(partition_id, slot as u64, generation);
    allocations
        .record(crate::contracts::transactions::data::primitives::RecordRef::Relation(relation_id))
        .map_err(record_allocation_conflict)?;
    Ok(relation_id)
}

pub mod entity_deletion;
pub(super) use entity_deletion::delete_entity_with_cascade;

pub use adjacency::apply_adjacency_deltas_for_request;
pub mod adjacency;
fn ensure_partition_state<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &'a mut WorkingState,
    partition_id: PartitionId,
) -> Result<
    &'a mut PartitionState,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
    state
        .get_partition_mut_for_request(partition_id, execution)
        .map_err(execution_denial)
}
fn execution_denial(
    cause: crate::contracts::execution::denial_cause::RelationalExecutionDenialCause,
) -> crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError
{
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(
        cause.into(),
    )
}
fn map_denial(
    cause: worth_execution::ExecutionOrderedMapDenial,
) -> crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError
{
    use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError as Error;
    match cause {
        worth_execution::ExecutionOrderedMapDenial::Request(cause) => {
            Error::execution(cause.into())
        }
        worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => Error::execution(
            crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow.into(),
        ),
        worth_execution::ExecutionOrderedMapDenial::Allocation(denial) => {
            crate::contracts::execution::denial::allocation_commit_error(Error::conflict(CommitConflict::new(
                ConflictClass::ExecutionAllocationDenied { denial },
            )))
        }
    }
}

fn allocate_record<K: RecordKind>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    allocations: &mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_batch::MutationRecordAllocations,
    class: crate::contracts::history::data::record_allocation_class::RecordAllocationClass,
    partition_id: PartitionId,
    kind_id: KindId,
    version_id: crate::contracts::identity::data::primitives::VersionId,
    extra: K::Extra,
) -> Result<
    (usize, u32),
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
    let partition = ensure_partition_state(execution, state, partition_id)?;
    let arena = K::arena_mut(partition);
    let reserved = allocations
        .reserve(class, partition_id)
        .map_err(record_allocation_conflict)?;
    arena
        .write_reserved_slot_for_request(
            execution,
            worth_relational_storage::internal_port::storage::substrate::record_arena::arena::SlotInit {
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
                worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::MetadataWriteDenial::Request(cause) => {
                    return crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(
                        cause.into(),
                    )
                }
                worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::MetadataWriteDenial::Invalid(detail) => detail,
            };
            record_allocation_conflict(
                crate::contracts::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial::ArenaWriteDenied {
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
    denial: crate::contracts::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial,
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
