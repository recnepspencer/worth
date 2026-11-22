//! Many-to-many entity and relation lineage finalization over validated Phase 2
//! postures.

use std::collections::BTreeMap;

use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::lineage::authority::LineagePreparationAuthority;
use crate::lineage::data::{
    FinalizedLineageEventBatch, LineageCommitPostureInput, LineageDecisionKind, LineageDecisionLog,
    LineageDecisionRecord, LineageEventKind, LineageEventRecord, LineageFinalizationArtifact,
    LineageNode, LineageRecordTarget, LineageSubject, PersistentNameBirthRecord,
    PersistentNameIdentity, PreparedLineageFinalization,
};
use crate::runtime::{LineageIdentityAllocator, WorkingState};
use crate::transactions::data::{MutationIntent, RecordRef, TransactionCommitError};

use super::posture_drafts::{build_posture_drafts, PostureEventDraft};
use super::posture_validation::{validate_commit_postures, ValidatedMergeTarget};
#[cfg(test)]
use super::staged_effects::install_lineage_id_in_working_state;
use super::staged_effects::{entity_lineage, install_lineage_id, lineage_error};

struct AssignedPostureEvent {
    kind: LineageEventKind,
    decision_kind: LineageDecisionKind,
    subject: LineageSubject,
    sources: Vec<LineageId>,
    targets: Vec<LineageId>,
    entity_node: Option<(EntityId, LineageId, bool)>,
    relation_lineage: Option<(RelationId, LineageId)>,
    name_birth: Option<PersistentNameBirthRecord>,
    kept_name: Option<PersistentNameIdentity>,
}

impl<'runtime> LineagePreparationAuthority<'runtime> {
    /// Validates every declared entity and relation posture against staged
    /// effects, mints persistent names, and then atomically reserves lineage
    /// IDs and emits canonical many-to-many `LineageEventRecord`s.
    pub(crate) fn ensure_lineage_for_commit_with_postures(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
        input: &LineageCommitPostureInput,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let prepared = finalize_postures_with_installer(
            &self.runtime.lineage_identity,
            staged,
            commit,
            merged_plan,
            changed_records,
            input,
            |staged, entity_id, lineage_id| {
                install_lineage_id(execution, staged, entity_id, lineage_id)
            },
        )?;
        self.runtime
            .performance_access()
            .count_lineage_finalization(
                prepared.artifact().event_batch().events().len(),
                prepared.artifact().decision_log().decisions().len(),
            );
        Ok(prepared)
    }
}

#[cfg(test)]
pub(crate) fn finalize_postures_in_working_state(
    allocator: &LineageIdentityAllocator,
    staged: &mut WorkingState,
    commit: &RelationalCommitReceipt,
    merged_plan: &[MutationIntent],
    changed_records: &[RecordRef],
    input: &LineageCommitPostureInput,
) -> Result<PreparedLineageFinalization, TransactionCommitError> {
    finalize_postures_with_installer(
        allocator,
        staged,
        commit,
        merged_plan,
        changed_records,
        input,
        install_lineage_id_in_working_state,
    )
}

fn finalize_postures_with_installer<F>(
    allocator: &LineageIdentityAllocator,
    staged: &mut WorkingState,
    commit: &RelationalCommitReceipt,
    merged_plan: &[MutationIntent],
    changed_records: &[RecordRef],
    input: &LineageCommitPostureInput,
    mut install_entity_lineage: F,
) -> Result<PreparedLineageFinalization, TransactionCommitError>
where
    F: FnMut(&mut WorkingState, EntityId, LineageId) -> Result<(), TransactionCommitError>,
{
    let mut validated = validate_commit_postures(staged, commit.version_id, changed_records, input)
        .map_err(TransactionCommitError::lineage_name_denial)?;

    let (drafts, required_lineage_ids) = build_posture_drafts(
        staged,
        commit,
        merged_plan,
        changed_records,
        input,
        &validated,
    );

    let lineage_width = u64::try_from(required_lineage_ids)
        .map_err(|_| lineage_error("lineage id batch capacity exceeded u64".to_owned()))?;
    let event_width = u64::try_from(drafts.len())
        .map_err(|_| lineage_error("lineage event batch capacity exceeded u64".to_owned()))?;
    let (lineage_start, event_start) = allocator
        .reserve(lineage_width, event_width)
        .map_err(lineage_error)?;
    let lineage_end = lineage_start + lineage_width;

    let assigned = assign_posture_drafts(
        staged,
        drafts,
        lineage_start..lineage_end,
        &mut validated.minted_births,
    )
    .map_err(lineage_error)?;
    debug_assert!(
        validated.minted_births.is_empty(),
        "all verified persistent name births must attach to emitted Create events"
    );

    let mut events = Vec::with_capacity(assigned.len());
    let mut decisions = Vec::with_capacity(assigned.len());
    let mut new_nodes = Vec::new();
    let mut new_relation_lineages = BTreeMap::new();

    for (offset, item) in assigned.into_iter().enumerate() {
        if let Some((entity_id, lineage_id, install)) = item.entity_node {
            if install {
                install_entity_lineage(staged, entity_id, lineage_id)?;
            }
            new_nodes.push(LineageNode {
                lineage_id,
                entity_id,
            });
        }
        if let Some((relation_id, lineage_id)) = item.relation_lineage {
            new_relation_lineages.insert(relation_id, lineage_id);
        }
        let offset = u64::try_from(offset)
            .map_err(|_| lineage_error("lineage event batch capacity exceeded u64".to_owned()))?;
        let event_id = event_start
            .checked_add(offset)
            .ok_or_else(|| lineage_error("lineage event id allocator exhausted".to_owned()))?;
        let event = LineageEventRecord {
            event_id,
            commit: commit.clone(),
            branch_id: commit.branch_id.clone(),
            kind: item.kind,
            sources: item.sources,
            targets: item.targets,
            subject: item.subject,
            name_birth: item.name_birth,
            kept_name: item.kept_name,
        };
        decisions.push(LineageDecisionRecord {
            branch_id: event.branch_id().clone(),
            kind: item.decision_kind,
            event_id: Some(event.event_id()),
            sources: event.sources().to_vec(),
            targets: event.targets().to_vec(),
        });
        events.push(event);
    }

    let artifact = LineageFinalizationArtifact::new(
        commit.branch_id.clone(),
        FinalizedLineageEventBatch::new(events),
        LineageDecisionLog::new(decisions),
    );
    Ok(PreparedLineageFinalization::with_relation_lineages(
        artifact,
        new_nodes,
        new_relation_lineages,
    ))
}

fn assign_posture_drafts(
    staged: &WorkingState,
    drafts: Vec<PostureEventDraft>,
    mut reserved: std::ops::Range<u64>,
    minted_births: &mut BTreeMap<LineageRecordTarget, PersistentNameBirthRecord>,
) -> Result<Vec<AssignedPostureEvent>, String> {
    let mut created_lineages: BTreeMap<LineageRecordTarget, LineageId> = BTreeMap::new();
    let mut assigned = Vec::with_capacity(drafts.len());

    for draft in drafts {
        let item = match draft {
            PostureEventDraft::Create {
                target,
                existing_lineage_id,
                kept_name,
            } => {
                let lineage_id = match existing_lineage_id {
                    Some(id) => id,
                    None => LineageId(reserved.next().ok_or_else(|| {
                        "lineage id reservation did not cover posture plan".to_owned()
                    })?),
                };
                created_lineages.insert(target, lineage_id);
                let (entity_node, relation_lineage) = match target {
                    LineageRecordTarget::Entity(entity_id) => (
                        Some((entity_id, lineage_id, existing_lineage_id.is_none())),
                        None,
                    ),
                    LineageRecordTarget::Relation(relation_id) => {
                        (None, Some((relation_id, lineage_id)))
                    }
                };
                AssignedPostureEvent {
                    kind: LineageEventKind::Create,
                    decision_kind: LineageDecisionKind::CreateAccepted,
                    subject: target.subject(),
                    sources: Vec::new(),
                    targets: vec![lineage_id],
                    entity_node,
                    relation_lineage,
                    name_birth: minted_births.remove(&target),
                    kept_name,
                }
            }
            PostureEventDraft::Retire {
                subject,
                lineage_id,
            } => AssignedPostureEvent {
                kind: LineageEventKind::Retire,
                decision_kind: LineageDecisionKind::RetireAccepted,
                subject,
                sources: vec![lineage_id],
                targets: Vec::new(),
                entity_node: None,
                relation_lineage: None,
                name_birth: None,
                kept_name: None,
            },
            PostureEventDraft::Replace {
                source_lineage_id,
                target_entity_id,
                target_lineage_id,
            } => {
                let target_lineage_id = target_lineage_id
                    .or_else(|| {
                        created_lineages
                            .get(&LineageRecordTarget::Entity(target_entity_id))
                            .copied()
                    })
                    .or_else(|| entity_lineage(staged, target_entity_id))
                    .ok_or_else(|| {
                        "lineage replacement target lost its planned lineage".to_owned()
                    })?;
                AssignedPostureEvent {
                    kind: LineageEventKind::Replace,
                    decision_kind: LineageDecisionKind::ReplaceAccepted,
                    subject: LineageSubject::Entity,
                    sources: vec![source_lineage_id],
                    targets: vec![target_lineage_id],
                    entity_node: None,
                    relation_lineage: None,
                    name_birth: None,
                    kept_name: None,
                }
            }
            PostureEventDraft::Split {
                subject,
                parent_lineage,
                children,
            } => {
                let mut target_lineages = Vec::with_capacity(children.len());
                for child in children {
                    let child_lineage = created_lineages
                        .get(&child)
                        .copied()
                        .ok_or_else(|| "split child lost its planned lineage".to_owned())?;
                    target_lineages.push(child_lineage);
                }
                target_lineages.sort_unstable();
                AssignedPostureEvent {
                    kind: LineageEventKind::Split,
                    decision_kind: LineageDecisionKind::SplitAccepted,
                    subject,
                    sources: vec![parent_lineage],
                    targets: target_lineages,
                    entity_node: None,
                    relation_lineage: None,
                    name_birth: None,
                    kept_name: None,
                }
            }
            PostureEventDraft::Merge {
                subject,
                source_lineages,
                target,
            } => {
                let (target_lineage, kept_name) = match target {
                    ValidatedMergeTarget::InPlaceKeeper {
                        lineage_id,
                        kept_name,
                    } => (lineage_id, Some(kept_name)),
                    ValidatedMergeTarget::RecreatedKeeper { target, kept_name } => {
                        let id = created_lineages.get(&target).copied().ok_or_else(|| {
                            "merge keeper target lost its planned lineage".to_owned()
                        })?;
                        (id, Some(kept_name))
                    }
                    ValidatedMergeTarget::MintedNew { target } => {
                        let id = created_lineages.get(&target).copied().ok_or_else(|| {
                            "merge minted target lost its planned lineage".to_owned()
                        })?;
                        (id, None)
                    }
                };
                AssignedPostureEvent {
                    kind: LineageEventKind::Merge,
                    decision_kind: LineageDecisionKind::MergeAccepted,
                    subject,
                    sources: source_lineages,
                    targets: vec![target_lineage],
                    entity_node: None,
                    relation_lineage: None,
                    name_birth: None,
                    kept_name,
                }
            }
        };
        assigned.push(item);
    }

    if reserved.next().is_some() {
        return Err("lineage id reservation exceeded posture plan".to_owned());
    }
    Ok(assigned)
}
