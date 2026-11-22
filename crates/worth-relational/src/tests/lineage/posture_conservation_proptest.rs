use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;

use crate::facade::identity::{EntityId, LineageId, RelationId, VersionId};
use crate::facade::lineage::{
    LineageCommitPostureInput, LineageEventKind, LineagePostureClaim, LineageRecordTarget,
    LineageSplitChild, LineageSubject, PersistentNameHolder, PersistentNameIdentity,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::runtime::LineageIdentityAllocator;

use super::posture_fixtures::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, push_relation, receipt_at,
    retire_entity, retire_relation, seam_birth, stage_entity_merge, stage_relation_merge,
    staged_entity_lineage,
};

#[derive(Clone, Copy, Debug)]
enum StepOp {
    CreateEntity,
    CreateRelation,
    PreserveEntity(usize),
    PreserveRelation(usize),
    SplitEntity(usize, u8),
    SplitRelation(usize, u8),
    MergeEntities {
        a: usize,
        b: usize,
        keep_parent: bool,
    },
    MergeRelations {
        a: usize,
        b: usize,
        keep_parent: bool,
    },
    RetireEntity(usize),
    RetireRelation(usize),
}

fn arb_step() -> impl Strategy<Value = StepOp> {
    prop_oneof![
        Just(StepOp::CreateEntity),
        Just(StepOp::CreateRelation),
        (0usize..8).prop_map(StepOp::PreserveEntity),
        (0usize..8).prop_map(StepOp::PreserveRelation),
        (0usize..8, 2u8..=3).prop_map(|(i, n)| StepOp::SplitEntity(i, n)),
        (0usize..8, 2u8..=3).prop_map(|(i, n)| StepOp::SplitRelation(i, n)),
        (0usize..8, 0usize..8, any::<bool>())
            .prop_map(|(a, b, keep_parent)| { StepOp::MergeEntities { a, b, keep_parent } }),
        (0usize..8, 0usize..8, any::<bool>())
            .prop_map(|(a, b, keep_parent)| { StepOp::MergeRelations { a, b, keep_parent } }),
        (0usize..8).prop_map(StepOp::RetireEntity),
        (0usize..8).prop_map(StepOp::RetireRelation),
    ]
}

fn run_conservation_sequence(ops: &[StepOp]) {
    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();
    let mut staged = empty_working_state();

    let anchor_a = push_entity(&mut staged, VersionId(1));
    let anchor_b = push_entity(&mut staged, VersionId(1));
    let mut live_entities: Vec<(EntityId, LineageId, PersistentNameIdentity)> = Vec::new();
    let mut live_relations: Vec<(RelationId, LineageId, PersistentNameIdentity)> = Vec::new();
    let mut live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder> = BTreeMap::new();
    let mut live_lineages: BTreeMap<PersistentNameHolder, LineageId> = BTreeMap::new();
    let mut all_known_lineages: BTreeSet<LineageId> = BTreeSet::new();
    let mut retired_lineages: BTreeSet<LineageId> = BTreeSet::new();

    let seed = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[RecordRef::Entity(anchor_a), RecordRef::Entity(anchor_b)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders: BTreeMap::new(),
            live_lineages: BTreeMap::new(),
            postures: vec![
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(anchor_a),
                    birth: face_birth("occ.seed", "authored", vec![], "anchor-a", anchor_a),
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(anchor_b),
                    birth: face_birth("occ.seed", "authored", vec![], "anchor-b", anchor_b),
                },
            ],
        },
    )
    .expect("seed commit succeeds");
    for ev in seed.artifact().event_batch().events() {
        let lin = ev.targets()[0];
        let name = ev.name_birth().unwrap().identity();
        all_known_lineages.insert(lin);
        let ent = if staged_entity_lineage(&staged, anchor_a) == Some(lin) {
            anchor_a
        } else {
            anchor_b
        };
        live_entities.push((ent, lin, name));
        live_holders.insert(name, PersistentNameHolder::Entity(ent));
        live_lineages.insert(PersistentNameHolder::Entity(ent), lin);
    }

    for (idx, &op) in ops.iter().enumerate() {
        let commit_no = (idx as u64) + 2;
        let ver = VersionId(commit_no);
        let occ = format!("occ.step.{commit_no}");
        let mut changed = Vec::new();
        let mut postures = Vec::new();
        let mut expected_retired: BTreeSet<LineageId> = BTreeSet::new();

        match op {
            StepOp::CreateEntity => {
                let e = push_entity(&mut staged, ver);
                changed.push(RecordRef::Entity(e));
                postures.push(LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(e),
                    birth: face_birth(&occ, "authored", vec![], &format!("e-{commit_no}"), e),
                });
            }
            StepOp::CreateRelation => {
                let r = push_relation(&mut staged, ver, anchor_a, anchor_b);
                changed.push(RecordRef::Relation(r));
                postures.push(LineagePostureClaim::Create {
                    target: LineageRecordTarget::Relation(r),
                    birth: seam_birth(&occ, "seam", vec![], &format!("r-{commit_no}"), r),
                });
            }
            StepOp::PreserveEntity(pick) if !live_entities.is_empty() => {
                let (e, _, name) = live_entities[pick % live_entities.len()];
                postures.push(LineagePostureClaim::Preserve {
                    target: LineageRecordTarget::Entity(e),
                    name,
                });
            }
            StepOp::PreserveRelation(pick) if !live_relations.is_empty() => {
                let (r, _, name) = live_relations[pick % live_relations.len()];
                postures.push(LineagePostureClaim::Preserve {
                    target: LineageRecordTarget::Relation(r),
                    name,
                });
            }
            StepOp::SplitEntity(pick, pieces) if !live_entities.is_empty() => {
                let (parent_e, parent_lin, parent_name) =
                    live_entities.remove(pick % live_entities.len());
                retire_entity(&mut staged, parent_e, ver);
                changed.push(RecordRef::Entity(parent_e));
                expected_retired.insert(parent_lin);
                let children = (0..pieces)
                    .map(|piece| {
                        let c = push_entity(&mut staged, ver);
                        changed.push(RecordRef::Entity(c));
                        LineageSplitChild {
                            target: LineageRecordTarget::Entity(c),
                            birth: face_birth(
                                &occ,
                                "cut_piece",
                                vec![parent_name],
                                &format!("ep-{commit_no}-{piece}"),
                                c,
                            ),
                        }
                    })
                    .collect();
                postures.push(LineagePostureClaim::Split {
                    parent_target: LineageRecordTarget::Entity(parent_e),
                    parent_name,
                    children,
                });
            }
            StepOp::SplitRelation(pick, pieces) if !live_relations.is_empty() => {
                let (parent_r, parent_lin, parent_name) =
                    live_relations.remove(pick % live_relations.len());
                retire_relation(&mut staged, parent_r, ver);
                changed.push(RecordRef::Relation(parent_r));
                expected_retired.insert(parent_lin);
                let children = (0..pieces)
                    .map(|piece| {
                        let c = push_relation(&mut staged, ver, anchor_a, anchor_b);
                        changed.push(RecordRef::Relation(c));
                        LineageSplitChild {
                            target: LineageRecordTarget::Relation(c),
                            birth: seam_birth(
                                &occ,
                                "seam_piece",
                                vec![parent_name],
                                &format!("rp-{commit_no}-{piece}"),
                                c,
                            ),
                        }
                    })
                    .collect();
                postures.push(LineagePostureClaim::Split {
                    parent_target: LineageRecordTarget::Relation(parent_r),
                    parent_name,
                    children,
                });
            }
            StepOp::MergeEntities { a, b, keep_parent } if live_entities.len() >= 2 => {
                postures.push(stage_entity_merge(
                    &mut staged,
                    &mut live_entities,
                    ver,
                    &occ,
                    commit_no,
                    a,
                    b,
                    keep_parent,
                    &mut changed,
                    &mut expected_retired,
                ));
            }
            StepOp::MergeRelations { a, b, keep_parent } if live_relations.len() >= 2 => {
                postures.push(stage_relation_merge(
                    &mut staged,
                    &mut live_relations,
                    (anchor_a, anchor_b),
                    ver,
                    &occ,
                    commit_no,
                    a,
                    b,
                    keep_parent,
                    &mut changed,
                    &mut expected_retired,
                ));
            }
            StepOp::RetireEntity(pick) if live_entities.len() > 2 => {
                let (e, lin, name) = live_entities.remove(pick % live_entities.len());
                retire_entity(&mut staged, e, ver);
                changed.push(RecordRef::Entity(e));
                expected_retired.insert(lin);
                postures.push(LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Entity(e),
                    name,
                });
            }
            StepOp::RetireRelation(pick) if !live_relations.is_empty() => {
                let (r, lin, name) = live_relations.remove(pick % live_relations.len());
                retire_relation(&mut staged, r, ver);
                changed.push(RecordRef::Relation(r));
                expected_retired.insert(lin);
                postures.push(LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Relation(r),
                    name,
                });
            }
            _ => continue,
        }

        let prepared = finalize_postures_in_working_state(
            &allocator,
            &mut staged,
            &receipt_at(commit_no, commit_no),
            &[],
            &changed,
            &LineageCommitPostureInput {
                scheme_registry: registry.clone(),
                live_holders: live_holders.clone(),
                live_lineages: live_lineages.clone(),
                postures,
            },
        )
        .expect("valid posture sequence must finalize cleanly");

        let mut observed_retired_sources = BTreeSet::new();
        for ev in prepared.artifact().event_batch().events() {
            match ev.kind() {
                LineageEventKind::Create => {
                    assert_eq!(ev.targets().len(), 1);
                    let lin = ev.targets()[0];
                    assert!(all_known_lineages.insert(lin), "duplicate created lineage");
                    let birth = ev.name_birth().expect("named Create carries birth");
                    match ev.subject() {
                        LineageSubject::Entity => {
                            let e = changed
                                .iter()
                                .find_map(|r| match r {
                                    RecordRef::Entity(id)
                                        if staged_entity_lineage(&staged, *id) == Some(lin) =>
                                    {
                                        Some(*id)
                                    }
                                    _ => None,
                                })
                                .expect("created entity lineage missing in staged arena");
                            live_entities.push((e, lin, birth.identity()));
                        }
                        LineageSubject::Relation => {
                            let (&r, _) = prepared
                                .relation_lineages()
                                .iter()
                                .find(|(_, l)| **l == lin)
                                .expect("created relation lineage recorded");
                            live_relations.push((r, lin, birth.identity()));
                        }
                    }
                }
                LineageEventKind::Retire | LineageEventKind::Split | LineageEventKind::Merge => {
                    for &src in ev.sources() {
                        if expected_retired.contains(&src) {
                            assert!(
                                observed_retired_sources.insert(src),
                                "retired source appeared twice in one commit"
                            );
                        }
                    }
                    for &tgt in ev.targets() {
                        assert!(
                            all_known_lineages.contains(&tgt) && !retired_lineages.contains(&tgt),
                            "dangling or retired target lineage {tgt:?}"
                        );
                    }
                }
                LineageEventKind::Replace => {}
            }
        }

        assert_eq!(observed_retired_sources, expected_retired);
        retired_lineages.extend(expected_retired);
        live_holders.clear();
        live_lineages.clear();
        let mut live_head_set = BTreeSet::new();
        for &(e, lin, name) in &live_entities {
            assert!(!retired_lineages.contains(&lin) && live_head_set.insert(lin));
            assert!(live_holders
                .insert(name, PersistentNameHolder::Entity(e))
                .is_none());
            live_lineages.insert(PersistentNameHolder::Entity(e), lin);
        }
        for &(r, lin, name) in &live_relations {
            assert!(!retired_lineages.contains(&lin) && live_head_set.insert(lin));
            assert!(live_holders
                .insert(name, PersistentNameHolder::Relation(r))
                .is_none());
            live_lineages.insert(PersistentNameHolder::Relation(r), lin);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn many_to_many_entity_and_relation_lineage_conservation_holds(
        ops in prop::collection::vec(arb_step(), 4..=18)
    ) {
        run_conservation_sequence(&ops);
    }
}
