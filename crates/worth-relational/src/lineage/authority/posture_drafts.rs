//! Canonical event draft planning for validated Phase 2 entity and relation
//! lineage postures alongside unclaimed default mutations.

use std::collections::{BTreeMap, BTreeSet};

use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::{
    LineageCommitPostureInput, LineageRecordTarget, LineageSubject, PersistentNameHolder,
    PersistentNameIdentity,
};
use crate::runtime::WorkingState;
use crate::transactions::data::{
    EntityMutationIntent, MutationIntent, RecordRef, RelationMutationIntent,
};

use super::posture_validation::{ValidatedMergeTarget, ValidatedPosture, ValidatedPostureBatch};
use super::staged_effects::{
    created_entity_lineage_slot, entity_lineage, find_replace_target_entity,
};

pub(super) enum PostureEventDraft {
    Create {
        target: LineageRecordTarget,
        existing_lineage_id: Option<LineageId>,
        kept_name: Option<PersistentNameIdentity>,
    },
    Retire {
        subject: LineageSubject,
        lineage_id: LineageId,
    },
    Replace {
        source_lineage_id: LineageId,
        target_entity_id: EntityId,
        target_lineage_id: Option<LineageId>,
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
) -> (Vec<PostureEventDraft>, usize) {
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
    for posture in &validated.postures {
        match posture {
            ValidatedPosture::Create { target } => {
                create_targets.insert(*target, None);
            }
            ValidatedPosture::Split { children, .. } => {
                for &child in children {
                    create_targets.insert(child, None);
                }
            }
            ValidatedPosture::Merge { target, .. } => match target {
                ValidatedMergeTarget::MintedNew { target } => {
                    create_targets.insert(*target, None);
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

    // 2. Emit validated posture transitions (Retire, Split, Merge).
    for posture in &validated.postures {
        match posture {
            ValidatedPosture::Retire {
                subject,
                source_lineage,
            } => drafts.push(PostureEventDraft::Retire {
                subject: *subject,
                lineage_id: *source_lineage,
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
            ValidatedPosture::Create { .. } | ValidatedPosture::Preserve => {}
        }
    }

    // 3. Emit default entity Delete/Replace and tracked relation Delete intents
    //    not already consumed by an explicit Phase 2 posture.
    //    Pre-seed `consumed_replace_targets` with posture-claimed created entities
    //    so an unpostured `Replace` never hijacks a posture-claimed target.
    let mut consumed_replace_targets: BTreeSet<EntityId> = validated
        .claimed_created_targets
        .iter()
        .filter_map(|target| match target {
            LineageRecordTarget::Entity(id) => Some(*id),
            LineageRecordTarget::Relation(_) => None,
        })
        .collect();
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
                    continue;
                };
                drafts.push(PostureEventDraft::Replace {
                    source_lineage_id,
                    target_entity_id,
                    target_lineage_id: entity_lineage(staged, target_entity_id),
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

    (drafts, required_lineage_ids)
}
