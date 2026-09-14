use std::collections::{BTreeMap, BTreeSet};

use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::{EntityId, KindId, LineageId, PartitionId, VersionId};
use crate::lineage::authority::LineagePreparationAuthority;
use crate::lineage::data::{
    FinalizedLineageEventBatch, LineageDecisionKind, LineageDecisionLog, LineageEventKind,
    LineageFinalizationArtifact, LineageNode, PreparedLineageFinalization,
};
use crate::runtime::{PartitionAccess, WorkingState};
use crate::transactions::data::{EntityMutationIntent, MutationIntent, RecordRef};

struct LineageFinalizationPlan {
    drafts: Vec<LineageEventDraft>,
    required_lineage_ids: usize,
}

enum LineageEventDraft {
    Create {
        entity_id: EntityId,
        existing_lineage_id: Option<LineageId>,
    },
    Retire {
        lineage_id: LineageId,
    },
    Replace {
        source_lineage_id: LineageId,
        target_entity_id: EntityId,
        target_lineage_id: Option<LineageId>,
    },
}

struct AssignedLineageEvent {
    kind: LineageEventKind,
    decision_kind: LineageDecisionKind,
    sources: Vec<LineageId>,
    targets: Vec<LineageId>,
    node: Option<AssignedLineageNode>,
}

struct AssignedLineageNode {
    entity_id: EntityId,
    lineage_id: LineageId,
    install_in_staged_state: bool,
}

impl<'runtime> LineagePreparationAuthority<'runtime> {
    pub(crate) fn ensure_lineage_for_commit(
        &self,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
    ) -> Result<PreparedLineageFinalization, String> {
        let plan =
            LineageFinalizationPlan::from_staged(staged, commit, merged_plan, changed_records);
        let lineage_width = u64::try_from(plan.required_lineage_ids)
            .map_err(|_| "lineage id batch capacity exceeded u64".to_owned())?;
        let event_width = u64::try_from(plan.drafts.len())
            .map_err(|_| "lineage event batch capacity exceeded u64".to_owned())?;
        let (lineage_start, event_start) = self
            .runtime
            .lineage_identity
            .reserve(lineage_width, event_width)?;
        let lineage_end = lineage_start + lineage_width;
        let assigned = plan.assign_lineage_ids(staged, lineage_start..lineage_end)?;

        let mut events = Vec::with_capacity(assigned.len());
        let mut decisions = Vec::with_capacity(assigned.len());
        let mut new_nodes = Vec::new();
        for (offset, assigned) in assigned.into_iter().enumerate() {
            if let Some(node) = assigned.node {
                if node.install_in_staged_state {
                    install_lineage_id(staged, node.entity_id, node.lineage_id)?;
                }
                new_nodes.push(LineageNode {
                    lineage_id: node.lineage_id,
                    entity_id: node.entity_id,
                });
            }
            let offset = u64::try_from(offset)
                .map_err(|_| "lineage event batch capacity exceeded u64".to_owned())?;
            let event_id = event_start
                .checked_add(offset)
                .ok_or_else(|| "lineage event id allocator exhausted".to_owned())?;
            let event = self.prepare_authoritative_lineage_event(
                event_id,
                commit,
                assigned.kind,
                assigned.sources,
                assigned.targets,
            );
            decisions.push(self.accepted_decision_record(assigned.decision_kind, &event));
            events.push(event);
        }

        let artifact = LineageFinalizationArtifact::new(
            commit.branch_id.clone(),
            FinalizedLineageEventBatch::new(events),
            LineageDecisionLog::new(decisions),
        );
        self.runtime
            .performance_access()
            .count_lineage_finalization(
                artifact.event_batch().events().len(),
                artifact.decision_log().decisions().len(),
            );
        Ok(PreparedLineageFinalization::new(artifact, new_nodes))
    }
}

impl LineageFinalizationPlan {
    fn from_staged(
        staged: &WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
    ) -> Self {
        let mut drafts = Vec::new();
        let mut required_lineage_ids = 0;
        for record in changed_records {
            let RecordRef::Entity(entity_id) = record else {
                continue;
            };
            let Some(existing_lineage_id) =
                created_entity_lineage_slot(staged, *entity_id, commit.version_id)
            else {
                continue;
            };
            required_lineage_ids += usize::from(existing_lineage_id.is_none());
            drafts.push(LineageEventDraft::Create {
                entity_id: *entity_id,
                existing_lineage_id,
            });
        }

        let mut consumed_replace_targets = BTreeSet::new();
        for intent in merged_plan {
            match intent {
                MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
                    if let Some(lineage_id) = entity_lineage(staged, spec.entity_id) {
                        drafts.push(LineageEventDraft::Retire { lineage_id });
                    }
                }
                MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                    let Some(source_lineage_id) = entity_lineage(staged, spec.entity_id) else {
                        continue;
                    };
                    let Some(target_entity_id) = find_replace_target_entity(
                        staged,
                        changed_records,
                        spec.entity_id,
                        spec.replacement.partition_id,
                        spec.replacement.kind_id,
                        commit.version_id,
                        &mut consumed_replace_targets,
                    ) else {
                        continue;
                    };
                    drafts.push(LineageEventDraft::Replace {
                        source_lineage_id,
                        target_entity_id,
                        target_lineage_id: entity_lineage(staged, target_entity_id),
                    });
                }
                MutationIntent::Create(_)
                | MutationIntent::Relation(_)
                | MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
                | MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_))
                | MutationIntent::Materialization(_) => {}
            }
        }
        Self {
            drafts,
            required_lineage_ids,
        }
    }

    fn assign_lineage_ids(
        self,
        staged: &WorkingState,
        mut reserved: std::ops::Range<u64>,
    ) -> Result<Vec<AssignedLineageEvent>, String> {
        let mut created_lineages = BTreeMap::new();
        let mut assigned = Vec::with_capacity(self.drafts.len());
        for draft in self.drafts {
            let event = match draft {
                LineageEventDraft::Create {
                    entity_id,
                    existing_lineage_id,
                } => {
                    let lineage_id = match existing_lineage_id {
                        Some(lineage_id) => lineage_id,
                        None => LineageId(reserved.next().ok_or_else(|| {
                            "lineage id reservation did not cover its plan".to_owned()
                        })?),
                    };
                    created_lineages.insert(entity_id, lineage_id);
                    AssignedLineageEvent {
                        kind: LineageEventKind::Create,
                        decision_kind: LineageDecisionKind::CreateAccepted,
                        sources: Vec::new(),
                        targets: vec![lineage_id],
                        node: Some(AssignedLineageNode {
                            entity_id,
                            lineage_id,
                            install_in_staged_state: existing_lineage_id.is_none(),
                        }),
                    }
                }
                LineageEventDraft::Retire { lineage_id } => AssignedLineageEvent {
                    kind: LineageEventKind::Retire,
                    decision_kind: LineageDecisionKind::RetireAccepted,
                    sources: vec![lineage_id],
                    targets: Vec::new(),
                    node: None,
                },
                LineageEventDraft::Replace {
                    source_lineage_id,
                    target_entity_id,
                    target_lineage_id,
                } => {
                    let target_lineage_id = target_lineage_id
                        .or_else(|| created_lineages.get(&target_entity_id).copied())
                        .or_else(|| entity_lineage(staged, target_entity_id))
                        .ok_or_else(|| {
                            "lineage replacement target lost its planned lineage".to_owned()
                        })?;
                    AssignedLineageEvent {
                        kind: LineageEventKind::Replace,
                        decision_kind: LineageDecisionKind::ReplaceAccepted,
                        sources: vec![source_lineage_id],
                        targets: vec![target_lineage_id],
                        node: None,
                    }
                }
            };
            assigned.push(event);
        }
        if reserved.next().is_some() {
            return Err("lineage id reservation exceeded its plan".to_owned());
        }
        Ok(assigned)
    }
}

fn created_entity_lineage_slot(
    staged: &WorkingState,
    entity_id: EntityId,
    version_id: VersionId,
) -> Option<Option<LineageId>> {
    let partition = staged.get_partition(entity_id.partition_id)?;
    let physical = partition
        .entity_arena
        .physical_index(entity_id.local_slot.0 as usize)?;
    if partition.entity_arena.created_at.get(physical).copied() != Some(version_id) {
        return None;
    }
    Some(partition.entity_arena.extra.get(physical)?.lineage_id)
}

fn install_lineage_id(
    staged: &mut WorkingState,
    entity_id: EntityId,
    lineage_id: LineageId,
) -> Result<(), String> {
    let partition = staged.get_partition_mut(entity_id.partition_id);
    let slot = entity_id.local_slot.0 as usize;
    let physical = partition
        .entity_arena
        .physical_index(slot)
        .ok_or_else(|| "planned lineage entity slot is unavailable".to_owned())?;
    let extra = partition
        .entity_arena
        .extra
        .get_mut(physical)
        .ok_or_else(|| "planned lineage entity metadata is unavailable".to_owned())?;
    extra.lineage_id = Some(lineage_id);
    let metadata = partition
        .entity_arena
        .metadata_history_at_mut(slot)
        .and_then(|history| history.last_mut())
        .ok_or_else(|| "planned lineage metadata history is unavailable".to_owned())?;
    metadata.lineage_id = Some(lineage_id);
    Ok(())
}

fn entity_lineage(staged: &WorkingState, entity_id: EntityId) -> Option<LineageId> {
    staged
        .get_partition(entity_id.partition_id)
        .and_then(|partition| partition.entity_arena.get(&entity_id))
        .and_then(|slot_view| slot_view.extra().lineage_id)
}

fn find_replace_target_entity(
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
        let partition = staged.get_partition(candidate.partition_id)?;
        let physical = partition
            .entity_arena
            .physical_index(candidate.local_slot.0 as usize)?;
        let created_now = partition.entity_arena.created_at.get(physical) == Some(&version_id);
        let matching_partition = candidate.partition_id == replacement_partition_id;
        let matching_kind = partition
            .entity_arena
            .kind_ids
            .get(physical)
            .copied()
            .flatten()
            .is_some_and(|kind_id| kind_id == replacement_kind_id);
        if created_now && matching_partition && matching_kind {
            consumed_replace_targets.insert(*candidate);
            return Some(*candidate);
        }
    }
    None
}
