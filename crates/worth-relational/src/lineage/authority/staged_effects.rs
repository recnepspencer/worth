//! Staged `WorkingState` inspection and lineage-slot installation helpers
//! shared between default commit finalization and Phase 2 posture validation.

use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{EntityId, KindId, LineageId, PartitionId, RelationId, VersionId};
use crate::lineage::data::{LineageRecordTarget, PersistentNameHolder};
use crate::runtime::WorkingState;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{RecordRef, TransactionCommitError};

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
        crate::publication::bundle::PublicationStage::BundleAssembly,
        detail,
    ))
}

pub(super) fn install_lineage_id(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: &mut WorkingState,
    entity_id: EntityId,
    lineage_id: LineageId,
) -> Result<(), TransactionCommitError> {
    staged
        .get_partition_mut(entity_id.partition_id)
        .entity_arena
        .install_lineage_id(execution, entity_id.slot_index(), lineage_id)
        .map_err(|denial| match denial {
            crate::storage::substrate::ExtraLineageDenial::Request(cause) => {
                TransactionCommitError::execution(cause.into())
            }
            crate::storage::substrate::ExtraLineageDenial::Missing(detail) => {
                lineage_error(detail.to_owned())
            }
        })
}

#[cfg(test)]
pub(super) fn install_lineage_id_in_working_state(
    staged: &mut WorkingState,
    entity_id: EntityId,
    lineage_id: LineageId,
) -> Result<(), TransactionCommitError> {
    let arena = &mut staged
        .get_partition_mut(entity_id.partition_id)
        .entity_arena;
    let physical = arena
        .physical_index(entity_id.slot_index())
        .ok_or_else(|| lineage_error("planned lineage entity slot is unavailable".to_owned()))?;
    let extra = arena.extra.get_mut(physical).ok_or_else(|| {
        lineage_error("planned lineage entity metadata is unavailable".to_owned())
    })?;
    extra.lineage_id = Some(lineage_id);
    Ok(())
}

pub(super) fn entity_lineage(staged: &WorkingState, entity_id: EntityId) -> Option<LineageId> {
    staged
        .get_current_partition(entity_id.partition_id)
        .and_then(|partition| partition.entity_arena().get(&entity_id))
        .and_then(|slot_view| slot_view.extra().lineage_id)
}

pub(super) fn basis_lineage_id(
    staged: &WorkingState,
    live_lineages: &BTreeMap<PersistentNameHolder, LineageId>,
    target: LineageRecordTarget,
) -> Option<LineageId> {
    match target {
        LineageRecordTarget::Entity(entity_id) => entity_lineage(staged, entity_id).or_else(|| {
            live_lineages
                .get(&PersistentNameHolder::Entity(entity_id))
                .copied()
        }),
        LineageRecordTarget::Relation(relation_id) => live_lineages
            .get(&PersistentNameHolder::Relation(relation_id))
            .copied(),
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
        let partition = staged.get_current_partition(candidate.partition_id)?;
        let arena = partition.entity_arena();
        let created_now = arena.created_at_for_slot(candidate.slot_index()) == Some(version_id);
        let matching_partition = candidate.partition_id == replacement_partition_id;
        let matching_kind =
            arena.get(candidate).and_then(|slot| slot.kind_id()) == Some(replacement_kind_id);
        if created_now && matching_partition && matching_kind {
            consumed_replace_targets.insert(*candidate);
            return Some(*candidate);
        }
    }
    None
}
