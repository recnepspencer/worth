//! Multi-operation commit staging helpers for the Phase 2 many-to-many lineage
//! conservation property test.

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;

use crate::facade::identity::{EntityId, KindId, LineageId, PartitionId, RelationId, VersionId};
use crate::facade::lineage::{
    LineagePostureClaim, LineageRecordTarget, LineageSplitChild, PersistentNameIdentity,
};
use crate::facade::transactions::{
    AspectFieldPatch, DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent, EntitySpec,
    MutationIntent, RecordRef, RelationMutationIntent, ReplaceEntityIntent,
};
use crate::storage::overlay::WorkingState;
use crate::symbols::data::ClientKey;

use super::posture_fixtures::{
    face_birth, push_entity, push_relation, retire_entity, retire_relation, seam_birth,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum MergeMode {
    InPlaceKeepParent,
    RecreatedKeepParent,
    MintNew,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum StepOp {
    CreateEntity,
    CreateRelation,
    PreserveEntity(usize),
    PreserveRelation(usize),
    SplitEntity(usize, u8),
    SplitRelation(usize, u8),
    MergeEntities { a: usize, b: usize, mode: MergeMode },
    MergeRelations { a: usize, b: usize, mode: MergeMode },
    RetireEntity(usize),
    RetireRelation(usize),
    DefaultDeleteEntity(usize),
    DefaultDeleteRelation(usize),
    DefaultReplaceEntity(usize),
}

fn arb_merge_mode() -> impl Strategy<Value = MergeMode> {
    prop_oneof![
        Just(MergeMode::InPlaceKeepParent),
        Just(MergeMode::RecreatedKeepParent),
        Just(MergeMode::MintNew),
    ]
}

pub(super) fn arb_step() -> impl Strategy<Value = StepOp> {
    prop_oneof![
        Just(StepOp::CreateEntity),
        Just(StepOp::CreateRelation),
        (0usize..8).prop_map(StepOp::PreserveEntity),
        (0usize..8).prop_map(StepOp::PreserveRelation),
        (0usize..8, 2u8..=3).prop_map(|(i, n)| StepOp::SplitEntity(i, n)),
        (0usize..8, 2u8..=3).prop_map(|(i, n)| StepOp::SplitRelation(i, n)),
        (0usize..8, 0usize..8, arb_merge_mode()).prop_map(|(a, b, mode)| StepOp::MergeEntities {
            a,
            b,
            mode
        }),
        (0usize..8, 0usize..8, arb_merge_mode()).prop_map(|(a, b, mode)| StepOp::MergeRelations {
            a,
            b,
            mode
        }),
        (0usize..8).prop_map(StepOp::RetireEntity),
        (0usize..8).prop_map(StepOp::RetireRelation),
        (0usize..8).prop_map(StepOp::DefaultDeleteEntity),
        (0usize..8).prop_map(StepOp::DefaultDeleteRelation),
        (0usize..8).prop_map(StepOp::DefaultReplaceEntity),
    ]
}

pub(super) struct CommitBatchAccumulator {
    pub(super) changed: Vec<RecordRef>,
    pub(super) merged_plan: Vec<MutationIntent>,
    pub(super) postures: Vec<LineagePostureClaim>,
    pub(super) expected_retired: BTreeSet<LineageId>,
    pub(super) expected_recreated_keepers: BTreeMap<LineageRecordTarget, PersistentNameIdentity>,
    pub(super) expected_unnamed_entities: BTreeSet<EntityId>,
    pub(super) kept_entities: Vec<(EntityId, LineageId, PersistentNameIdentity)>,
    pub(super) kept_relations: Vec<(RelationId, LineageId, PersistentNameIdentity)>,
}

impl CommitBatchAccumulator {
    pub(super) fn new() -> Self {
        Self {
            changed: Vec::new(),
            merged_plan: Vec::new(),
            postures: Vec::new(),
            expected_retired: BTreeSet::new(),
            expected_recreated_keepers: BTreeMap::new(),
            expected_unnamed_entities: BTreeSet::new(),
            kept_entities: Vec::new(),
            kept_relations: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_op(
        &mut self,
        staged: &mut WorkingState,
        avail_entities: &mut Vec<(EntityId, LineageId, PersistentNameIdentity)>,
        avail_relations: &mut Vec<(RelationId, LineageId, PersistentNameIdentity)>,
        anchors: (EntityId, EntityId),
        ver: VersionId,
        commit_no: u64,
        op_idx: usize,
        op: StepOp,
    ) {
        let occ = format!("occ.c{commit_no}.op{op_idx}");
        match op {
            StepOp::CreateEntity => {
                let e = push_entity(staged, ver);
                self.changed.push(RecordRef::Entity(e));
                self.postures.push(LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(e),
                    birth: face_birth(
                        &occ,
                        "authored",
                        vec![],
                        &format!("e-{commit_no}-{op_idx}"),
                        e,
                    ),
                });
            }
            StepOp::CreateRelation => {
                let r = push_relation(staged, ver, anchors.0, anchors.1);
                self.changed.push(RecordRef::Relation(r));
                self.postures.push(LineagePostureClaim::Create {
                    target: LineageRecordTarget::Relation(r),
                    birth: seam_birth(&occ, "seam", vec![], &format!("r-{commit_no}-{op_idx}"), r),
                });
            }
            StepOp::PreserveEntity(pick) if !avail_entities.is_empty() => {
                let item = avail_entities.remove(pick % avail_entities.len());
                self.postures.push(LineagePostureClaim::Preserve {
                    target: LineageRecordTarget::Entity(item.0),
                    name: item.2,
                });
                self.kept_entities.push(item);
            }
            StepOp::PreserveRelation(pick) if !avail_relations.is_empty() => {
                let item = avail_relations.remove(pick % avail_relations.len());
                self.postures.push(LineagePostureClaim::Preserve {
                    target: LineageRecordTarget::Relation(item.0),
                    name: item.2,
                });
                self.kept_relations.push(item);
            }
            StepOp::SplitEntity(pick, pieces) if !avail_entities.is_empty() => {
                let (parent_e, parent_lin, parent_name) =
                    avail_entities.remove(pick % avail_entities.len());
                retire_entity(staged, parent_e, ver);
                self.changed.push(RecordRef::Entity(parent_e));
                // Include matching Delete intent to continuously prove claimed_basis_targets suppression.
                self.merged_plan
                    .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                        DeleteEntityIntent {
                            entity_id: parent_e,
                        },
                    )));
                self.expected_retired.insert(parent_lin);
                let children = (0..pieces)
                    .map(|piece| {
                        let c = push_entity(staged, ver);
                        self.changed.push(RecordRef::Entity(c));
                        LineageSplitChild {
                            target: LineageRecordTarget::Entity(c),
                            birth: face_birth(
                                &occ,
                                "cut_piece",
                                vec![parent_name],
                                &format!("ep-{commit_no}-{op_idx}-{piece}"),
                                c,
                            ),
                        }
                    })
                    .collect();
                self.postures.push(LineagePostureClaim::Split {
                    parent_target: LineageRecordTarget::Entity(parent_e),
                    parent_name,
                    children,
                });
            }
            StepOp::SplitRelation(pick, pieces) if !avail_relations.is_empty() => {
                let (parent_r, parent_lin, parent_name) =
                    avail_relations.remove(pick % avail_relations.len());
                retire_relation(staged, parent_r, ver);
                self.changed.push(RecordRef::Relation(parent_r));
                self.merged_plan
                    .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                        DeleteRelationIntent {
                            relation_id: parent_r,
                        },
                    )));
                self.expected_retired.insert(parent_lin);
                let children = (0..pieces)
                    .map(|piece| {
                        let c = push_relation(staged, ver, anchors.0, anchors.1);
                        self.changed.push(RecordRef::Relation(c));
                        LineageSplitChild {
                            target: LineageRecordTarget::Relation(c),
                            birth: seam_birth(
                                &occ,
                                "seam_piece",
                                vec![parent_name],
                                &format!("rp-{commit_no}-{op_idx}-{piece}"),
                                c,
                            ),
                        }
                    })
                    .collect();
                self.postures.push(LineagePostureClaim::Split {
                    parent_target: LineageRecordTarget::Relation(parent_r),
                    parent_name,
                    children,
                });
            }
            StepOp::MergeEntities { a, b, mode } if avail_entities.len() >= 2 => {
                self.stage_entity_merge(
                    staged,
                    avail_entities,
                    ver,
                    &occ,
                    commit_no,
                    op_idx,
                    a,
                    b,
                    mode,
                );
            }
            StepOp::MergeRelations { a, b, mode } if avail_relations.len() >= 2 => {
                self.stage_relation_merge(
                    staged,
                    avail_relations,
                    anchors,
                    ver,
                    &occ,
                    commit_no,
                    op_idx,
                    a,
                    b,
                    mode,
                );
            }
            StepOp::RetireEntity(pick) if avail_entities.len() > 2 => {
                let (e, lin, name) = avail_entities.remove(pick % avail_entities.len());
                retire_entity(staged, e, ver);
                self.changed.push(RecordRef::Entity(e));
                self.merged_plan
                    .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                        DeleteEntityIntent { entity_id: e },
                    )));
                self.expected_retired.insert(lin);
                self.postures.push(LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Entity(e),
                    name,
                });
            }
            StepOp::RetireRelation(pick) if !avail_relations.is_empty() => {
                let (r, lin, name) = avail_relations.remove(pick % avail_relations.len());
                retire_relation(staged, r, ver);
                self.changed.push(RecordRef::Relation(r));
                self.merged_plan
                    .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                        DeleteRelationIntent { relation_id: r },
                    )));
                self.expected_retired.insert(lin);
                self.postures.push(LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Relation(r),
                    name,
                });
            }
            StepOp::DefaultDeleteEntity(pick) if avail_entities.len() > 2 => {
                let (e, lin, _) = avail_entities.remove(pick % avail_entities.len());
                retire_entity(staged, e, ver);
                self.changed.push(RecordRef::Entity(e));
                self.merged_plan
                    .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                        DeleteEntityIntent { entity_id: e },
                    )));
                self.expected_retired.insert(lin);
            }
            StepOp::DefaultDeleteRelation(pick) if !avail_relations.is_empty() => {
                let (r, lin, _) = avail_relations.remove(pick % avail_relations.len());
                retire_relation(staged, r, ver);
                self.changed.push(RecordRef::Relation(r));
                self.merged_plan
                    .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                        DeleteRelationIntent { relation_id: r },
                    )));
                self.expected_retired.insert(lin);
            }
            StepOp::DefaultReplaceEntity(pick) if avail_entities.len() > 2 => {
                let (e_old, lin_old, _) = avail_entities.remove(pick % avail_entities.len());
                retire_entity(staged, e_old, ver);
                let e_new = push_entity(staged, ver);
                self.changed.push(RecordRef::Entity(e_old));
                self.changed.push(RecordRef::Entity(e_new));
                self.merged_plan
                    .push(MutationIntent::Entity(EntityMutationIntent::Replace(
                        ReplaceEntityIntent {
                            entity_id: e_old,
                            replacement: EntitySpec {
                                partition_id: PartitionId::main(),
                                kind_id: KindId(1),
                                client_key: ClientKey::raw(format!("rep-{commit_no}-{op_idx}")),
                                fields: AspectFieldPatch::default(),
                            },
                        },
                    )));
                self.expected_retired.insert(lin_old);
                self.expected_unnamed_entities.insert(e_new);
            }
            _ => {}
        }
    }
}
