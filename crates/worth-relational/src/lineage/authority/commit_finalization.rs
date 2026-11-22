#[cfg(test)]
use std::collections::{BTreeMap, BTreeSet};

use crate::history::data::RelationalCommitReceipt;
#[cfg(test)]
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::authority::LineagePreparationAuthority;
use crate::lineage::data::PreparedLineageFinalization;
#[cfg(test)]
use crate::lineage::data::{
    FinalizedLineageEventBatch, InstalledPersistentNameSchemeRegistry, LineageDecisionKind,
    LineageDecisionLog, LineageEventKind, LineageFinalizationArtifact, LineageNode, LineageSubject,
    PersistentNameBirthRecord, PersistentNameBirthRequest, PersistentNameHolder,
    PersistentNameIdentity,
};
use crate::runtime::WorkingState;
#[cfg(test)]
use crate::transactions::data::EntityMutationIntent;
use crate::transactions::data::{MutationIntent, RecordRef, TransactionCommitError};

#[cfg(test)]
use super::staged_effects::{
    created_entity_lineage_slot, entity_lineage, find_replace_target_entity, install_lineage_id,
    lineage_error,
};

#[cfg(test)]
struct LineageFinalizationPlan {
    drafts: Vec<LineageEventDraft>,
    required_lineage_ids: usize,
}

#[cfg(test)]
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

#[cfg(test)]
struct AssignedLineageEvent {
    kind: LineageEventKind,
    decision_kind: LineageDecisionKind,
    sources: Vec<LineageId>,
    targets: Vec<LineageId>,
    node: Option<AssignedLineageNode>,
    name_birth: Option<PersistentNameBirthRecord>,
}

#[cfg(test)]
struct AssignedLineageNode {
    entity_id: EntityId,
    lineage_id: LineageId,
    install_in_staged_state: bool,
}

impl<'runtime> LineagePreparationAuthority<'runtime> {
    pub(crate) fn ensure_lineage_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        self.ensure_lineage_for_commit_with_postures(
            execution,
            staged,
            commit,
            merged_plan,
            changed_records,
            &crate::lineage::data::LineageCommitPostureInput::default(),
        )
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ensure_lineage_for_commit_with_births(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
        scheme_registry: &InstalledPersistentNameSchemeRegistry,
        live_holders: &BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
        retired_in_commit: &BTreeSet<PersistentNameIdentity>,
        named_births: &BTreeMap<EntityId, PersistentNameBirthRequest>,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        // Verify every birth targets a created entity before minting or reserving IDs.
        let plan =
            LineageFinalizationPlan::from_staged(staged, commit, merged_plan, changed_records);
        let created_entities: BTreeSet<EntityId> = plan
            .drafts
            .iter()
            .filter_map(|draft| match draft {
                LineageEventDraft::Create { entity_id, .. } => Some(*entity_id),
                _ => None,
            })
            .collect();
        let mut minted_births = super::name_minting::mint_entity_births(
            scheme_registry,
            live_holders,
            retired_in_commit,
            &created_entities,
            named_births,
        )
        .map_err(TransactionCommitError::lineage_name_denial)?;
        let lineage_width = u64::try_from(plan.required_lineage_ids)
            .map_err(|_| lineage_error("lineage id batch capacity exceeded u64".to_owned()))?;
        let event_width = u64::try_from(plan.drafts.len())
            .map_err(|_| lineage_error("lineage event batch capacity exceeded u64".to_owned()))?;
        let (lineage_start, event_start) = self
            .runtime
            .lineage_identity
            .reserve(lineage_width, event_width)
            .map_err(lineage_error)?;
        let lineage_end = lineage_start + lineage_width;
        let assigned = plan
            .assign_lineage_ids(staged, lineage_start..lineage_end, &mut minted_births)
            .map_err(lineage_error)?;
        debug_assert!(
            minted_births.is_empty(),
            "all verified entity births must be attached to Create lineage events"
        );

        let mut events = Vec::with_capacity(assigned.len());
        let mut decisions = Vec::with_capacity(assigned.len());
        let mut new_nodes = Vec::new();
        for (offset, assigned) in assigned.into_iter().enumerate() {
            if let Some(node) = assigned.node {
                if node.install_in_staged_state {
                    install_lineage_id(execution, staged, node.entity_id, node.lineage_id)?;
                }
                new_nodes.push(LineageNode {
                    lineage_id: node.lineage_id,
                    entity_id: node.entity_id,
                });
            }
            let offset = u64::try_from(offset).map_err(|_| {
                lineage_error("lineage event batch capacity exceeded u64".to_owned())
            })?;
            let event_id = event_start
                .checked_add(offset)
                .ok_or_else(|| lineage_error("lineage event id allocator exhausted".to_owned()))?;
            let event = self.prepare_authoritative_lineage_event(
                event_id,
                commit,
                assigned.kind,
                LineageSubject::Entity,
                assigned.sources,
                assigned.targets,
                assigned.name_birth,
                None,
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

#[cfg(test)]
impl LineageFinalizationPlan {
    fn from_staged(
        staged: &WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
    ) -> Self {
        let mut drafts = Vec::new();
        let mut required_lineage_ids = 0;
        let mut created_entities = Vec::new();
        for record in changed_records {
            let RecordRef::Entity(entity_id) = record else {
                continue;
            };
            let Some(existing_lineage_id) =
                created_entity_lineage_slot(staged, *entity_id, commit.version_id)
            else {
                continue;
            };
            created_entities.push((*entity_id, existing_lineage_id));
        }
        // Create targets bind positionally to canonical entity patch records.
        // Establish that order before assigning identities and digesting events;
        // replacement target discovery still needs the original draft order.
        created_entities.sort_unstable_by_key(|(entity_id, _)| *entity_id);
        for (entity_id, existing_lineage_id) in created_entities {
            required_lineage_ids += usize::from(existing_lineage_id.is_none());
            drafts.push(LineageEventDraft::Create {
                entity_id,
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
                | MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
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
        minted_births: &mut BTreeMap<EntityId, PersistentNameBirthRecord>,
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
                        name_birth: minted_births.remove(&entity_id),
                    }
                }
                LineageEventDraft::Retire { lineage_id } => AssignedLineageEvent {
                    kind: LineageEventKind::Retire,
                    decision_kind: LineageDecisionKind::RetireAccepted,
                    sources: vec![lineage_id],
                    targets: Vec::new(),
                    node: None,
                    name_birth: None,
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
                        name_birth: None,
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
