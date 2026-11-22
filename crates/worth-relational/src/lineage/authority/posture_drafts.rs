//! Canonical event draft planning for validated Phase 2 entity and relation
//! lineage postures alongside unclaimed default mutations.

use std::collections::{BTreeMap, BTreeSet};

use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::{
    LineageCommitPostureInput, LineageNameDenial, LineagePostureKind, LineageRecordTarget,
    LineageSubject, PersistentNameHolder, PersistentNameIdentity,
};
use crate::runtime::WorkingState;
use crate::transactions::data::{
    EntityMutationIntent, MutationIntent, RecordRef, RelationMutationIntent,
};

use super::posture_validation::{ValidatedMergeTarget, ValidatedPosture, ValidatedPostureBatch};
use super::staged_effects::{
    created_entity_lineage_slot, entity_lineage, find_claimed_replace_target_entity,
    find_replace_target_entity,
};

pub(super) enum PostureEventDraft {
    Create {
        target: LineageRecordTarget,
        existing_lineage_id: Option<LineageId>,
        kept_name: Option<PersistentNameIdentity>,
    },
    Restore {
        target: LineageRecordTarget,
        source_lineage: LineageId,
        name: PersistentNameIdentity,
        existing_lineage_id: Option<LineageId>,
    },
    Retire {
        subject: LineageSubject,
        lineage_id: LineageId,
    },
    Replace {
        subject: LineageSubject,
        source_lineage_id: LineageId,
        target: LineageRecordTarget,
        target_lineage_id: Option<LineageId>,
        kept_name: Option<PersistentNameIdentity>,
    },
    Split {
        subject: LineageSubject,
        parent_lineage: LineageId,
        children: Vec<LineageRecordTarget>,
    },
    Merge {
        subject: LineageSubject,
        source_lineages: Vec<LineageId>,
        target: ValidatedMergeTarget,
    },
}

pub(super) fn build_posture_drafts(
    staged: &WorkingState,
    commit: &RelationalCommitReceipt,
    merged_plan: &[MutationIntent],
    changed_records: &[RecordRef],
    input: &LineageCommitPostureInput,
    validated: &ValidatedPostureBatch,
) -> Result<(Vec<PostureEventDraft>, usize), LineageNameDenial> {
    let mut drafts = Vec::new();
    let mut required_lineage_ids = 0usize;

    // 1. Collect all created targets needing Create events:
    //    - Unclaimed created entities (backward-compatible default behavior)
    //    - Claimed entity & relation creations (Create, Split children, Merge targets)
    let mut create_targets: BTreeMap<LineageRecordTarget, Option<PersistentNameIdentity>> =
        BTreeMap::new();
    for record in changed_records {
        let RecordRef::Entity(entity_id) = record else {
            continue;
        };
        let target = LineageRecordTarget::Entity(*entity_id);
        if validated.claimed_created_targets.contains(&target) {
            continue;
        }
        if created_entity_lineage_slot(staged, *entity_id, commit.version_id).is_some() {
            create_targets.insert(target, None);
        }
    }
    let mut restore_targets: BTreeMap<LineageRecordTarget, (LineageId, PersistentNameIdentity)> =
        BTreeMap::new();
    for posture in &validated.postures {
        match posture {
            ValidatedPosture::Create { target } => {
                create_targets.insert(*target, None);
            }
            ValidatedPosture::Regenerate { target, name, .. } => {
                create_targets.insert(*target, Some(*name));
            }
            ValidatedPosture::Restore {
                target,
                source_lineage,
                name,
            } => {
                restore_targets.insert(*target, (*source_lineage, *name));
            }
            ValidatedPosture::Split { children, .. } => {
                for &child in children {
                    if !validated.restored_created_targets.contains(&child) {
                        create_targets.insert(child, None);
                    }
                }
            }
            ValidatedPosture::Merge { target, .. } => match target {
                ValidatedMergeTarget::MintedNew { target } => {
                    if !validated.restored_created_targets.contains(target) {
                        create_targets.insert(*target, None);
                    }
                }
                ValidatedMergeTarget::RecreatedKeeper { target, kept_name } => {
                    create_targets.insert(*target, Some(*kept_name));
                }
                ValidatedMergeTarget::InPlaceKeeper { .. } => {}
            },
            ValidatedPosture::Preserve | ValidatedPosture::Retire { .. } => {}
        }
    }

    for (target, kept_name) in create_targets {
        let existing_lineage_id = match target {
            LineageRecordTarget::Entity(entity_id) => {
                created_entity_lineage_slot(staged, entity_id, commit.version_id).flatten()
            }
            LineageRecordTarget::Relation(_) => None,
        };
        required_lineage_ids += usize::from(existing_lineage_id.is_none());
        drafts.push(PostureEventDraft::Create {
            target,
            existing_lineage_id,
            kept_name,
        });
    }
    for (target, (source_lineage, name)) in restore_targets {
        let existing_lineage_id = match target {
            LineageRecordTarget::Entity(entity_id) => {
                created_entity_lineage_slot(staged, entity_id, commit.version_id).flatten()
            }
            LineageRecordTarget::Relation(_) => None,
        };
        required_lineage_ids += usize::from(existing_lineage_id.is_none());
        drafts.push(PostureEventDraft::Restore {
            target,
            source_lineage,
            name,
            existing_lineage_id,
        });
    }

    // 2. Emit validated posture transitions (Retire, Regenerate->Replace, Split, Merge).
    for posture in &validated.postures {
        match posture {
            ValidatedPosture::Retire {
                subject,
                source_lineage,
                ..
            } => drafts.push(PostureEventDraft::Retire {
                subject: *subject,
                lineage_id: *source_lineage,
            }),
            ValidatedPosture::Regenerate {
                subject,
                source_lineage,
                target,
                name,
            } => drafts.push(PostureEventDraft::Replace {
                subject: *subject,
                source_lineage_id: *source_lineage,
                target: *target,
                target_lineage_id: None,
                kept_name: Some(*name),
            }),
            ValidatedPosture::Split {
                subject,
                parent_lineage,
                children,
            } => drafts.push(PostureEventDraft::Split {
                subject: *subject,
                parent_lineage: *parent_lineage,
                children: children.clone(),
            }),
            ValidatedPosture::Merge {
                subject,
                source_lineages,
                target,
            } => drafts.push(PostureEventDraft::Merge {
                subject: *subject,
                source_lineages: source_lineages.clone(),
                target: match target {
                    ValidatedMergeTarget::InPlaceKeeper {
                        lineage_id,
                        kept_name,
                    } => ValidatedMergeTarget::InPlaceKeeper {
                        lineage_id: *lineage_id,
                        kept_name: *kept_name,
                    },
                    ValidatedMergeTarget::RecreatedKeeper { target, kept_name } => {
                        ValidatedMergeTarget::RecreatedKeeper {
                            target: *target,
                            kept_name: *kept_name,
                        }
                    }
                    ValidatedMergeTarget::MintedNew { target } => {
                        ValidatedMergeTarget::MintedNew { target: *target }
                    }
                },
            }),
            ValidatedPosture::Create { .. }
            | ValidatedPosture::Restore { .. }
            | ValidatedPosture::Preserve => {}
        }
    }

    // 3. Emit default entity Delete/Replace and tracked relation Delete intents
    //    not already consumed by an explicit Phase 2 posture.
    //    Pre-seed `consumed_replace_targets` with posture-claimed created entities
    //    so an unpostured `Replace` never hijacks a posture-claimed target, and
    //    fail closed if posture claims leave an unpostured `Replace` with no
    //    unclaimed successor.
    let claimed_created_entities: BTreeSet<EntityId> = validated
        .claimed_created_targets
        .iter()
        .filter_map(|target| match target {
            LineageRecordTarget::Entity(id) => Some(*id),
            LineageRecordTarget::Relation(_) => None,
        })
        .collect();
    let mut consumed_replace_targets = claimed_created_entities.clone();
    for intent in merged_plan {
        match intent {
            MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
                if validated
                    .claimed_basis_targets
                    .contains(&LineageRecordTarget::Entity(spec.entity_id))
                {
                    continue;
                }
                if let Some(lineage_id) = entity_lineage(staged, spec.entity_id) {
                    drafts.push(PostureEventDraft::Retire {
                        subject: LineageSubject::Entity,
                        lineage_id,
                    });
                }
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                if validated
                    .claimed_basis_targets
                    .contains(&LineageRecordTarget::Entity(spec.entity_id))
                {
                    continue;
                }
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
                    if let Some(claimed_target) = find_claimed_replace_target_entity(
                        staged,
                        changed_records,
                        spec.entity_id,
                        spec.replacement.partition_id,
                        spec.replacement.kind_id,
                        commit.version_id,
                        &claimed_created_entities,
                    ) {
                        return Err(LineageNameDenial::LineagePostureMismatch {
                            posture: claiming_posture_kind(validated, claimed_target),
                            subject: LineageSubject::Entity,
                            holder: PersistentNameHolder::Entity(spec.entity_id),
                            detail: format!(
                                "unpostured Replace source {:?} has no unclaimed replacement target because candidate {:?} is claimed by an explicit posture",
                                spec.entity_id, claimed_target
                            ),
                        });
                    }
                    continue;
                };
                drafts.push(PostureEventDraft::Replace {
                    subject: LineageSubject::Entity,
                    source_lineage_id,
                    target: LineageRecordTarget::Entity(target_entity_id),
                    target_lineage_id: entity_lineage(staged, target_entity_id),
                    kept_name: None,
                });
            }
            MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
                let target = LineageRecordTarget::Relation(spec.relation_id);
                if validated.claimed_basis_targets.contains(&target) {
                    continue;
                }
                if let Some(&lineage_id) = input
                    .live_lineages
                    .get(&PersistentNameHolder::Relation(spec.relation_id))
                {
                    drafts.push(PostureEventDraft::Retire {
                        subject: LineageSubject::Relation,
                        lineage_id,
                    });
                }
            }
            MutationIntent::Create(_)
            | MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(_)
                | RelationMutationIntent::ApplyAspectPatch(_)
                | RelationMutationIntent::Revalidate(_),
            )
            | MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(_)
                | EntityMutationIntent::ApplyAspectPatch(_)
                | EntityMutationIntent::Revalidate(_),
            )
            | MutationIntent::Materialization(_) => {}
        }
    }

    Ok((drafts, required_lineage_ids))
}

fn claiming_posture_kind(
    validated: &ValidatedPostureBatch,
    entity_id: EntityId,
) -> LineagePostureKind {
    let target = LineageRecordTarget::Entity(entity_id);
    for posture in &validated.postures {
        match posture {
            ValidatedPosture::Create { target: claimed }
            | ValidatedPosture::Regenerate {
                target: claimed, ..
            } if *claimed == target => {
                return LineagePostureKind::Create;
            }
            ValidatedPosture::Restore {
                target: claimed, ..
            } if *claimed == target => {
                return LineagePostureKind::Restore;
            }
            ValidatedPosture::Split { children, .. } if children.contains(&target) => {
                return LineagePostureKind::Split;
            }
            ValidatedPosture::Merge {
                target:
                    ValidatedMergeTarget::MintedNew { target: claimed }
                    | ValidatedMergeTarget::RecreatedKeeper {
                        target: claimed, ..
                    },
                ..
            } if *claimed == target => {
                return LineagePostureKind::Merge;
            }
            ValidatedPosture::Create { .. }
            | ValidatedPosture::Regenerate { .. }
            | ValidatedPosture::Restore { .. }
            | ValidatedPosture::Preserve
            | ValidatedPosture::Retire { .. }
            | ValidatedPosture::Split { .. }
            | ValidatedPosture::Merge { .. } => {}
        }
    }
    LineagePostureKind::Create
}
