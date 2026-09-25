use std::collections::BTreeSet;

use crate::config::data::CascadeDeletePolicy;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::WorkingState;
use crate::storage::overlay::{PartitionAccess, PartitionState};
use crate::storage::substrate::{
    EntityExtra, EntityRecordKind, RecordKind, RelationEndpoints, RelationExtra, RelationRecordKind,
};
use crate::transactions::data::{
    CommitConflict, ConflictClass, EntityCascadeDeleteMissingState,
    MutationStateInconsistencyEvidence,
};

use super::intents::entity_authoritative_deletion_patch::plan_entity_authoritative_deletion_patch;
use super::outcomes::{MutationOutcome, RecordMutation};
use super::{AdjacencyDelta, AdjacencyDeltaKind};

mod relation_lifecycle;

pub(super) use relation_lifecycle::{delete_relation, retain_relation_dangling_for_audit};

pub(super) fn allocate_entity_with_extra(
    state: &mut WorkingState,
    allocations: &mut crate::runtime::PendingRecordAllocations,
    version_id: crate::identity::data::VersionId,
    partition_id: PartitionId,
    kind_id: KindId,
    extra: EntityExtra,
) -> Result<EntityId, CommitConflict> {
    let (slot, generation) = allocate_record::<EntityRecordKind>(
        state,
        allocations,
        crate::history::data::RecordAllocationClass::Entity,
        partition_id,
        kind_id,
        version_id,
        extra,
    )?;
    let partition = ensure_partition_state(state, partition_id);
    let adjacency_policy = partition.adjacency_policy.clone();
    partition.adjacency.clear_slot(slot, &adjacency_policy);
    partition
        .reverse_adjacency
        .clear_slot(slot, &adjacency_policy);
    // Empty membership is still stored adjacency truth. Record initialization
    // and slot reuse must publish both changed adjacency axes in the journal.
    state.mark_adjacency_slot_touched(partition_id, slot);
    state.mark_reverse_adjacency_slot_touched(partition_id, slot);
    let entity_id = EntityId::new(partition_id, slot as u64, generation);
    allocations.record(crate::transactions::data::RecordRef::Entity(entity_id));
    Ok(entity_id)
}

pub(super) fn allocate_relation(
    state: &mut WorkingState,
    allocations: &mut crate::runtime::PendingRecordAllocations,
    version_id: crate::identity::data::VersionId,
    partition_id: PartitionId,
    kind_id: KindId,
    source: EntityId,
    target: EntityId,
    authoritative_aspect_state: Option<worth_foundational::facade::AuthoritativeRecordAspectState>,
) -> Result<RelationId, CommitConflict> {
    let extra = RelationExtra {
        endpoints: Some(RelationEndpoints { source, target }),
        authoritative_aspect_state,
    };
    let (slot, generation) = allocate_record::<RelationRecordKind>(
        state,
        allocations,
        crate::history::data::RecordAllocationClass::Relation,
        partition_id,
        kind_id,
        version_id,
        extra,
    )?;
    let relation_id = RelationId::new(partition_id, slot as u64, generation);
    allocations.record(crate::transactions::data::RecordRef::Relation(relation_id));
    Ok(relation_id)
}

pub(super) fn delete_entity_with_cascade(
    state: &mut WorkingState,
    version_id: crate::identity::data::VersionId,
    entity_id: EntityId,
    schema_registry: &RelationalSchemaRegistry,
    default_cascade_delete_policy: CascadeDeletePolicy,
    outcome: &mut MutationOutcome,
) -> Result<(), CommitConflict> {
    let slot = entity_id.slot_index();
    state.mark_entity_slot_touched(entity_id.partition_id, slot);
    let partition = state.get_partition_mut(entity_id.partition_id);
    let slot_view = partition.entity_arena.get_slot(slot).ok_or_else(|| {
        mutation_state_inconsistency(
            "entity delete requires an existing slot after stale-target validation",
            entity_cascade_delete_missing(entity_id, EntityCascadeDeleteMissingState::Slot),
        )
    })?;
    let kind_id = slot_view.kind_id().ok_or_else(|| {
        mutation_state_inconsistency(
            "entity delete requires a retained kind id after stale-target validation",
            entity_cascade_delete_missing(entity_id, EntityCascadeDeleteMissingState::KindId),
        )
    })?;
    let authoritative_patch = plan_entity_authoritative_deletion_patch(
        slot_view.extra().authoritative_aspect_state.as_ref(),
        &schema_registry
            .entity_registration(kind_id)
            .map_err(|error| {
                CommitConflict::new(ConflictClass::KindSchemaMismatch {
                    detail: format!(
                        "entity delete requires registered aspect contracts: {error:?}"
                    ),
                })
            })?
            .aspect_contract_declarations,
    )
    .map_err(|denial| {
        CommitConflict::new(ConflictClass::EntityAuthoritativeAspectStateDenied { kind_id, denial })
    })?;
    partition.entity_arena.retire(slot, version_id);
    outcome.record_change(RecordMutation::EntityDeleted {
        entity_id,
        kind_id,
        authoritative_patch,
    });

    let mut attached = BTreeSet::new();
    if let Some(adjacency) = partition.adjacency.get(slot) {
        adjacency.extend_into(&mut attached);
    }
    if let Some(reverse_adjacency) = partition.reverse_adjacency.get(slot) {
        reverse_adjacency.extend_into(&mut attached);
    }
    for relation_id in attached {
        let cascade_policy = state
            .get_partition(relation_id.partition_id)
            .and_then(|partition| {
                partition
                    .relation_arena
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
                delete_relation(state, version_id, relation_id, outcome);
            }
            CascadeDeletePolicy::RetainDanglingForAudit => {
                retain_relation_dangling_for_audit(state, version_id, relation_id, outcome);
            }
        }
    }
    Ok(())
}

pub(crate) fn apply_adjacency_deltas(
    state: &mut WorkingState,
    deltas: &[AdjacencyDelta],
    version_id: crate::identity::data::VersionId,
) {
    for delta in deltas {
        let (source, target) = match delta.kind {
            AdjacencyDeltaKind::Created { source, target }
            | AdjacencyDeltaKind::Deleted { source, target } => (source, target),
        };
        state.mark_adjacency_slot_touched(source.partition_id, source.slot_index());
        state.mark_reverse_adjacency_slot_touched(target.partition_id, target.slot_index());

        let source_partition = ensure_partition_state(state, source.partition_id);
        ensure_entity_adjacency_capacity(source_partition, source.slot_index());
        match delta.kind {
            AdjacencyDeltaKind::Created { .. } => {
                source_partition
                    .adjacency
                    .ensure(source.slot_index(), &source_partition.adjacency_policy)
                    .insert_at(delta.kind_id, delta.relation_id, version_id);
            }
            AdjacencyDeltaKind::Deleted { .. } => {
                if let Some(relations) = source_partition.adjacency.get_mut(source.slot_index()) {
                    relations.remove_at(delta.kind_id, &delta.relation_id, version_id);
                }
            }
        }

        let target_partition = ensure_partition_state(state, target.partition_id);
        ensure_entity_adjacency_capacity(target_partition, target.slot_index());
        match delta.kind {
            AdjacencyDeltaKind::Created { .. } => {
                target_partition
                    .reverse_adjacency
                    .ensure(target.slot_index(), &target_partition.adjacency_policy)
                    .insert_at(delta.kind_id, delta.relation_id, version_id);
            }
            AdjacencyDeltaKind::Deleted { .. } => {
                if let Some(relations) = target_partition
                    .reverse_adjacency
                    .get_mut(target.slot_index())
                {
                    relations.remove_at(delta.kind_id, &delta.relation_id, version_id);
                }
            }
        }
    }
}

fn ensure_partition_state(
    state: &mut WorkingState,
    partition_id: PartitionId,
) -> &mut PartitionState {
    state.get_partition_mut(partition_id)
}

fn allocate_record<K: RecordKind>(
    state: &mut WorkingState,
    allocations: &mut crate::runtime::PendingRecordAllocations,
    class: crate::history::data::RecordAllocationClass,
    partition_id: PartitionId,
    kind_id: KindId,
    version_id: crate::identity::data::VersionId,
    extra: K::Extra,
) -> Result<(usize, u32), CommitConflict> {
    let partition = ensure_partition_state(state, partition_id);
    let arena = K::arena_mut(partition);
    let reserved = allocations
        .reserve(class, partition_id)
        .map_err(record_allocation_conflict)?;
    arena
        .write_reserved_slot(
            crate::storage::substrate::SlotInit {
                partition_id,
                kind_id,
                version_id,
                extra,
            },
            reserved.slot,
            reserved.generation,
        )
        .map_err(|detail| {
            record_allocation_conflict(
                crate::transactions::data::RecordAllocationDenial::ArenaWriteDenied {
                    class,
                    partition_id,
                    slot: reserved.slot,
                    detail: detail.to_owned(),
                },
            )
        })?;
    Ok((reserved.slot, reserved.generation))
}

fn record_allocation_conflict(
    denial: crate::transactions::data::RecordAllocationDenial,
) -> CommitConflict {
    CommitConflict::new(ConflictClass::RecordAllocationDenied { denial })
}

fn ensure_entity_adjacency_capacity(partition: &mut PartitionState, slot: usize) {
    let policy = partition.adjacency_policy.clone();
    partition.adjacency.ensure(slot, &policy);
    partition.reverse_adjacency.ensure(slot, &policy);
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
