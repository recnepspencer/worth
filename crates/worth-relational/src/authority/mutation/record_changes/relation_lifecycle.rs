use crate::identity::data::RelationId;
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionState;
use crate::storage::overlay::{CurrentPartitionAccess, WorkingState};

use crate::authority::mutation::outcomes::{MutationOutcome, RecordMutation};

pub(in crate::authority::mutation) fn retain_relation_dangling_for_audit(
    state: &mut WorkingState,
    version_id: crate::identity::data::VersionId,
    relation_id: RelationId,
    outcome: &mut MutationOutcome,
) {
    let slot = relation_id.slot_index();
    if !relation_is_visible_for_audit_retention(state, relation_id) {
        return;
    }

    state.mark_relation_slot_touched(relation_id.partition_id, slot);
    let partition = state.get_partition_mut(relation_id.partition_id);
    partition.relation_arena.retire(slot, version_id);
    partition
        .relation_arena
        .set_lifecycle_for_slot(slot, RecordLifecycleState::RetainedDanglingForAudit);
    partition.relation_arena.live_bitset.set(slot, true);
    partition.relation_arena.reclaimable_bitset.set(slot, false);

    let Some(snapshot) = relation_mutation_snapshot(state, relation_id) else {
        return;
    };
    outcome.record_change(RecordMutation::RelationRetainedForAudit {
        relation_id,
        kind_id: snapshot.kind_id,
        source: snapshot.source,
        target: snapshot.target,
        authoritative_aspect_state: snapshot.authoritative_aspect_state,
    });
}

pub(in crate::authority::mutation) fn delete_relation(
    state: &mut WorkingState,
    version_id: crate::identity::data::VersionId,
    relation_id: RelationId,
    outcome: &mut MutationOutcome,
) {
    let slot = relation_id.slot_index();
    if !relation_is_live(state, relation_id) {
        return;
    }

    state.mark_relation_slot_touched(relation_id.partition_id, slot);
    let snapshot = relation_mutation_snapshot(state, relation_id);
    let partition = state.get_partition_mut(relation_id.partition_id);
    partition.relation_arena.retire(slot, version_id);
    let lifecycle = deletion_retention_lifecycle(partition, slot);
    partition
        .relation_arena
        .set_lifecycle_for_slot(slot, lifecycle);

    let Some(snapshot) = snapshot else {
        return;
    };
    outcome.record_change(RecordMutation::RelationDeleted {
        relation_id,
        kind_id: snapshot.kind_id,
        source: snapshot.source,
        target: snapshot.target,
        authoritative_aspect_state: snapshot.authoritative_aspect_state,
    });
}

fn relation_is_visible_for_audit_retention(state: &WorkingState, relation_id: RelationId) -> bool {
    state
        .get_current_partition(relation_id.partition_id)
        .and_then(|partition| partition.relation_arena().get(&relation_id))
        .is_some_and(|relation_slot| {
            matches!(
                relation_slot.lifecycle(),
                RecordLifecycleState::Live | RecordLifecycleState::RetainedDanglingForAudit
            )
        })
}

fn relation_is_live(state: &WorkingState, relation_id: RelationId) -> bool {
    state
        .get_current_partition(relation_id.partition_id)
        .and_then(|partition| partition.relation_arena().get(&relation_id))
        .is_some_and(|relation_slot| relation_slot.lifecycle() == RecordLifecycleState::Live)
}

#[derive(Debug)]
struct RelationMutationSnapshot {
    kind_id: crate::identity::data::KindId,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
    authoritative_aspect_state: Option<worth_foundational::facade::AuthoritativeRecordAspectState>,
}

fn relation_mutation_snapshot(
    state: &WorkingState,
    relation_id: RelationId,
) -> Option<RelationMutationSnapshot> {
    let relation_slot = state
        .get_current_partition(relation_id.partition_id)?
        .relation_arena()
        .get_slot(relation_id.slot_index())?;
    let endpoints = relation_slot.extra().endpoints.clone()?;
    Some(RelationMutationSnapshot {
        kind_id: relation_slot.kind_id()?,
        source: endpoints.source,
        target: endpoints.target,
        authoritative_aspect_state: relation_slot.extra().authoritative_aspect_state.clone(),
    })
}

fn deletion_retention_lifecycle(partition: &PartitionState, slot: usize) -> RecordLifecycleState {
    if partition
        .relation_arena
        .snapshot_pin_count(slot)
        .unwrap_or(0)
        > 0
    {
        RecordLifecycleState::PinnedBySnapshot
    } else if partition.relation_arena.branch_pin_count(slot).unwrap_or(0) > 0 {
        RecordLifecycleState::PinnedByBranch
    } else if partition.relation_arena.replay_pin_count(slot).unwrap_or(0) > 0 {
        RecordLifecycleState::PinnedByReplayRetention
    } else {
        RecordLifecycleState::DeletedRetained
    }
}
