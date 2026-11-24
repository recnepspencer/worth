//! Staged `WorkingState` inspection and lineage-slot installation helpers
//! shared between default commit finalization and Phase 2 posture validation.

use std::collections::BTreeSet;

use crate::identity::data::{EntityId, KindId, LineageId, PartitionId, RelationId, VersionId};
use crate::lineage::data::{LineageBasisView, LineageRecordTarget, PersistentNameHolder};
use crate::storage::overlay::CurrentPartitionAccess;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;

pub(super) fn created_entity_lineage_slot(
    staged: &WorkingState,
    entity_id: EntityId,
    version_id: VersionId,
) -> Option<Option<LineageId>> {
    let partition = staged.get_current_partition(entity_id.partition_id)?;
    let arena = partition.entity_arena();
    if arena.created_at_for_slot(entity_id.slot_index()) != Some(version_id) {
        return None;
    }
    Some(arena.get(&entity_id)?.extra().lineage_id)
}

pub(super) fn lineage_error(detail: String) -> TransactionCommitError {
    TransactionCommitError::publication(crate::publication::data::PublicationError::new(
        crate::publication::bundle::publication_status::PublicationStage::BundleAssembly,
        detail,
    ))
}

pub(super) fn entity_lineage(staged: &WorkingState, entity_id: EntityId) -> Option<LineageId> {
    staged
        .get_current_partition(entity_id.partition_id)
        .and_then(|partition| partition.entity_arena().get(&entity_id))
        .and_then(|slot_view| slot_view.extra().lineage_id)
}

pub(super) fn basis_lineage_id(
    staged: &WorkingState,
    basis: &dyn LineageBasisView,
    target: LineageRecordTarget,
) -> Option<LineageId> {
    match target {
        LineageRecordTarget::Entity(entity_id) => entity_lineage(staged, entity_id)
            .or_else(|| basis.live_lineage(&PersistentNameHolder::Entity(entity_id))),
        LineageRecordTarget::Relation(relation_id) => {
            basis.live_lineage(&PersistentNameHolder::Relation(relation_id))
        }
    }
}

pub(super) fn target_kind_id(staged: &WorkingState, target: LineageRecordTarget) -> Option<KindId> {
    match target {
        LineageRecordTarget::Entity(id) => staged
            .get_current_partition(id.partition_id)
            .and_then(|p| p.entity_arena().get(&id))
            .and_then(|slot| slot.kind_id()),
        LineageRecordTarget::Relation(id) => staged
            .get_current_partition(id.partition_id)
            .and_then(|p| p.relation_arena().get(&id))
            .and_then(|slot| slot.kind_id()),
    }
}

pub(super) fn is_created_in_commit(
    staged: &WorkingState,
    target: LineageRecordTarget,
    version_id: VersionId,
) -> bool {
    match target {
        LineageRecordTarget::Entity(id) => staged
            .get_current_partition(id.partition_id)
            .is_some_and(|p| {
                p.entity_arena().get(&id).is_some()
                    && p.entity_arena().created_at_for_slot(id.slot_index()) == Some(version_id)
            }),
        LineageRecordTarget::Relation(id) => relation_created_at(staged, id) == Some(version_id),
    }
}

/// Whether this commit staged `target`'s partition. A partition it never
/// staged is exactly as the commit basis left it, so its records are neither
/// created, retired nor mutated by this commit.
///
/// Complexity: O(1) expected partition probe.
pub(super) fn is_partition_staged(staged: &WorkingState, target: LineageRecordTarget) -> bool {
    let partition = match target {
        LineageRecordTarget::Entity(id) => id.partition_id,
        LineageRecordTarget::Relation(id) => id.partition_id,
    };
    staged.get_current_partition(partition).is_some()
}

pub(super) fn existed_at_basis(
    staged: &WorkingState,
    target: LineageRecordTarget,
    version_id: VersionId,
) -> bool {
    match target {
        LineageRecordTarget::Entity(id) => staged
            .get_current_partition(id.partition_id)
            .is_some_and(|p| {
                p.entity_arena().get(&id).is_some()
                    && p.entity_arena()
                        .created_at_for_slot(id.slot_index())
                        .is_some_and(|created| created < version_id)
            }),
        LineageRecordTarget::Relation(id) => {
            relation_created_at(staged, id).is_some_and(|created| created < version_id)
        }
    }
}

pub(super) fn is_retired_in_commit(
    staged: &WorkingState,
    target: LineageRecordTarget,
    version_id: VersionId,
) -> bool {
    match target {
        LineageRecordTarget::Entity(id) => staged
            .get_current_partition(id.partition_id)
            .and_then(|p| p.entity_arena().get(&id))
            .is_some_and(|slot| slot.retired_at() == Some(version_id)),
        LineageRecordTarget::Relation(id) => staged
            .get_current_partition(id.partition_id)
            .and_then(|p| p.relation_arena().get(&id))
            .is_some_and(|slot| slot.retired_at() == Some(version_id)),
    }
}

pub(super) fn is_live_after_commit(staged: &WorkingState, target: LineageRecordTarget) -> bool {
    match target {
        LineageRecordTarget::Entity(id) => staged
            .get_current_partition(id.partition_id)
            .and_then(|p| p.entity_arena().get(&id))
            .is_some_and(|slot| slot.retired_at().is_none()),
        LineageRecordTarget::Relation(id) => staged
            .get_current_partition(id.partition_id)
            .and_then(|p| p.relation_arena().get(&id))
            .is_some_and(|slot| slot.retired_at().is_none()),
    }
}

fn relation_created_at(staged: &WorkingState, id: RelationId) -> Option<VersionId> {
    let partition = staged.get_current_partition(id.partition_id)?;
    partition.relation_arena().get(&id)?;
    partition
        .relation_arena()
        .created_at_for_slot(id.slot_index())
}

fn is_matching_replace_candidate(
    staged: &WorkingState,
    candidate: EntityId,
    replacement_partition_id: PartitionId,
    replacement_kind_id: KindId,
    version_id: VersionId,
) -> bool {
    let Some(partition) = staged.get_current_partition(candidate.partition_id) else {
        return false;
    };
    let arena = partition.entity_arena();
    let created_now = arena.created_at_for_slot(candidate.slot_index()) == Some(version_id);
    let matching_partition = candidate.partition_id == replacement_partition_id;
    let matching_kind =
        arena.get(&candidate).and_then(|slot| slot.kind_id()) == Some(replacement_kind_id);
    created_now && matching_partition && matching_kind
}

pub(super) fn find_replace_target_entity(
    staged: &WorkingState,
    changed_records: &[RecordRef],
    source_entity_id: EntityId,
    replacement_partition_id: PartitionId,
    replacement_kind_id: KindId,
    version_id: VersionId,
    consumed_replace_targets: &mut BTreeSet<EntityId>,
) -> Option<EntityId> {
    let source_index = changed_records.iter().position(
        |record| matches!(record, RecordRef::Entity(candidate) if *candidate == source_entity_id),
    )?;
    for record in changed_records.iter().skip(source_index + 1) {
        let RecordRef::Entity(candidate) = record else {
            continue;
        };
        if consumed_replace_targets.contains(candidate) {
            continue;
        }
        if is_matching_replace_candidate(
            staged,
            *candidate,
            replacement_partition_id,
            replacement_kind_id,
            version_id,
        ) {
            consumed_replace_targets.insert(*candidate);
            return Some(*candidate);
        }
    }
    None
}

pub(super) fn find_claimed_replace_target_entity(
    staged: &WorkingState,
    changed_records: &[RecordRef],
    source_entity_id: EntityId,
    replacement_partition_id: PartitionId,
    replacement_kind_id: KindId,
    version_id: VersionId,
    claimed_created_entities: &BTreeSet<EntityId>,
) -> Option<EntityId> {
    if claimed_created_entities.is_empty() {
        return None;
    }
    let source_index = changed_records.iter().position(
        |record| matches!(record, RecordRef::Entity(candidate) if *candidate == source_entity_id),
    )?;
    for record in changed_records.iter().skip(source_index + 1) {
        let RecordRef::Entity(candidate) = record else {
            continue;
        };
        if claimed_created_entities.contains(candidate)
            && is_matching_replace_candidate(
                staged,
                *candidate,
                replacement_partition_id,
                replacement_kind_id,
                version_id,
            )
        {
            return Some(*candidate);
        }
    }
    None
}
