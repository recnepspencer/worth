//! Merge staging helpers (`InPlaceKeepParent`, `RecreatedKeepParent`, and
//! `MintNew`) for the multi-op lineage conservation property test.

use crate::facade::identity::{EntityId, LineageId, RelationId, VersionId};
use crate::facade::lineage::{
    LineageMergeKeeper, LineagePostureClaim, LineageRecordTarget, PersistentNameIdentity,
};
use crate::facade::transactions::{
    DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent, MutationIntent, RecordRef,
    RelationMutationIntent,
};
use crate::storage::overlay::WorkingState;

use super::posture_conservation_ops::{CommitBatchAccumulator, MergeMode};
use super::posture_fixtures::{
    face_birth, push_entity, push_relation, retire_entity, retire_relation, seam_birth,
};

impl CommitBatchAccumulator {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn stage_entity_merge(
        &mut self,
        staged: &mut WorkingState,
        avail: &mut Vec<(EntityId, LineageId, PersistentNameIdentity)>,
        ver: VersionId,
        occ: &str,
        commit_no: u64,
        op_idx: usize,
        a: usize,
        b: usize,
        mode: MergeMode,
    ) {
        let ((e1, lin1, name1), (e2, lin2, name2)) = pick_two_distinct(avail, a, b);
        retire_entity(staged, e2, ver);
        self.changed.push(RecordRef::Entity(e2));
        self.merged_plan
            .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id: e2 },
            )));
        self.expected_retired.insert(lin2);
        let (target, keeper) = match mode {
            MergeMode::InPlaceKeepParent => {
                self.kept_entities.push((e1, lin1, name1));
                (
                    LineageRecordTarget::Entity(e1),
                    LineageMergeKeeper::KeepParent(name1),
                )
            }
            MergeMode::RecreatedKeepParent => {
                retire_entity(staged, e1, ver);
                self.changed.push(RecordRef::Entity(e1));
                self.merged_plan
                    .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                        DeleteEntityIntent { entity_id: e1 },
                    )));
                self.expected_retired.insert(lin1);
                let rep = push_entity(staged, ver);
                self.changed.push(RecordRef::Entity(rep));
                self.expected_recreated_keepers
                    .insert(LineageRecordTarget::Entity(rep), name1);
                (
                    LineageRecordTarget::Entity(rep),
                    LineageMergeKeeper::KeepParent(name1),
                )
            }
            MergeMode::MintNew => {
                retire_entity(staged, e1, ver);
                self.changed.push(RecordRef::Entity(e1));
                self.expected_retired.insert(lin1);
                let m = push_entity(staged, ver);
                self.changed.push(RecordRef::Entity(m));
                (
                    LineageRecordTarget::Entity(m),
                    LineageMergeKeeper::MintNew {
                        birth: face_birth(
                            occ,
                            "merged_face",
                            vec![name1, name2],
                            &format!("em-{commit_no}-{op_idx}"),
                            m,
                        ),
                    },
                )
            }
        };
        self.postures.push(LineagePostureClaim::Merge {
            parents: vec![
                (LineageRecordTarget::Entity(e1), name1),
                (LineageRecordTarget::Entity(e2), name2),
            ],
            target,
            keeper,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn stage_relation_merge(
        &mut self,
        staged: &mut WorkingState,
        avail: &mut Vec<(RelationId, LineageId, PersistentNameIdentity)>,
        anchors: (EntityId, EntityId),
        ver: VersionId,
        occ: &str,
        commit_no: u64,
        op_idx: usize,
        a: usize,
        b: usize,
        mode: MergeMode,
    ) {
        let ((r1, lin1, name1), (r2, lin2, name2)) = pick_two_distinct(avail, a, b);
        retire_relation(staged, r2, ver);
        self.changed.push(RecordRef::Relation(r2));
        self.merged_plan
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent { relation_id: r2 },
            )));
        self.expected_retired.insert(lin2);
        let (target, keeper) = match mode {
            MergeMode::InPlaceKeepParent => {
                self.kept_relations.push((r1, lin1, name1));
                (
                    LineageRecordTarget::Relation(r1),
                    LineageMergeKeeper::KeepParent(name1),
                )
            }
            MergeMode::RecreatedKeepParent => {
                retire_relation(staged, r1, ver);
                self.changed.push(RecordRef::Relation(r1));
                self.merged_plan
                    .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                        DeleteRelationIntent { relation_id: r1 },
                    )));
                self.expected_retired.insert(lin1);
                let rep = push_relation(staged, ver, anchors.0, anchors.1);
                self.changed.push(RecordRef::Relation(rep));
                self.expected_recreated_keepers
                    .insert(LineageRecordTarget::Relation(rep), name1);
                (
                    LineageRecordTarget::Relation(rep),
                    LineageMergeKeeper::KeepParent(name1),
                )
            }
            MergeMode::MintNew => {
                retire_relation(staged, r1, ver);
                self.changed.push(RecordRef::Relation(r1));
                self.expected_retired.insert(lin1);
                let m = push_relation(staged, ver, anchors.0, anchors.1);
                self.changed.push(RecordRef::Relation(m));
                (
                    LineageRecordTarget::Relation(m),
                    LineageMergeKeeper::MintNew {
                        birth: seam_birth(
                            occ,
                            "merged_seam",
                            vec![name1, name2],
                            &format!("rm-{commit_no}-{op_idx}"),
                            m,
                        ),
                    },
                )
            }
        };
        self.postures.push(LineagePostureClaim::Merge {
            parents: vec![
                (LineageRecordTarget::Relation(r1), name1),
                (LineageRecordTarget::Relation(r2), name2),
            ],
            target,
            keeper,
        });
    }
}

fn pick_two_distinct<T>(vec: &mut Vec<T>, a: usize, b: usize) -> (T, T) {
    let idx_a = a % vec.len();
    let mut idx_b = b % vec.len();
    if idx_b == idx_a {
        idx_b = (idx_a + 1) % vec.len();
    }
    let (hi, lo) = if idx_a > idx_b {
        (idx_a, idx_b)
    } else {
        (idx_b, idx_a)
    };
    let second = vec.remove(hi);
    let first = vec.remove(lo);
    (first, second)
}
