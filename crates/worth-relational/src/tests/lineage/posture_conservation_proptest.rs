use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;

use crate::facade::identity::{EntityId, LineageId, RelationId, VersionId};
use crate::facade::lineage::{
    LineageCommitPostureInput, LineageEventKind, LineagePostureClaim, LineageRecordTarget,
    LineageSubject, PersistentNameHolder, PersistentNameIdentity,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::runtime::LineageIdentityAllocator;

use super::posture_conservation_ops::{arb_step, CommitBatchAccumulator, StepOp};
use super::posture_fixtures::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, receipt_at,
    staged_entity_lineage,
};

fn run_multi_op_conservation_sequence(commit_batches: &[Vec<StepOp>]) {
    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();
    let mut staged = empty_working_state();

    let anchor_a = push_entity(&mut staged, VersionId(1));
    let anchor_b = push_entity(&mut staged, VersionId(1));
    let mut live_entities: Vec<(EntityId, LineageId, PersistentNameIdentity)> = Vec::new();
    let mut live_relations: Vec<(RelationId, LineageId, PersistentNameIdentity)> = Vec::new();
    let mut live_unnamed_entities: Vec<(EntityId, LineageId)> = Vec::new();
    let mut live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder> = BTreeMap::new();
    let mut live_lineages: BTreeMap<PersistentNameHolder, LineageId> = BTreeMap::new();
    let mut all_known_lineages: BTreeSet<LineageId> = BTreeSet::new();
    let mut retired_lineages: BTreeSet<LineageId> = BTreeSet::new();

    // Commit 1 seeds two anchor faces so relations always have live endpoints.
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
            retired_lineages: BTreeMap::new(),
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

    for (batch_idx, ops) in commit_batches.iter().enumerate() {
        let commit_no = (batch_idx as u64) + 2;
        let ver = VersionId(commit_no);
        let mut avail_entities = live_entities.clone();
        let mut avail_relations = live_relations.clone();
        let mut acc = CommitBatchAccumulator::new();

        for (op_idx, &op) in ops.iter().enumerate() {
            acc.apply_op(
                &mut staged,
                &mut avail_entities,
                &mut avail_relations,
                (anchor_a, anchor_b),
                ver,
                commit_no,
                op_idx,
                op,
            );
        }
        if acc.changed.is_empty() && acc.postures.is_empty() && acc.merged_plan.is_empty() {
            continue;
        }

        let prepared = finalize_postures_in_working_state(
            &allocator,
            &mut staged,
            &receipt_at(commit_no, commit_no),
            &acc.merged_plan,
            &acc.changed,
            &LineageCommitPostureInput {
                scheme_registry: registry.clone(),
                live_holders: live_holders.clone(),
                live_lineages: live_lineages.clone(),
                retired_lineages: BTreeMap::new(),
                postures: acc.postures,
            },
        )
        .expect("valid multi-op posture + default intent commit must finalize cleanly");

        // Carry forward untouched basis records plus explicitly preserved/in-place-kept records.
        live_entities = avail_entities;
        live_entities.extend(acc.kept_entities);
        live_relations = avail_relations;
        live_relations.extend(acc.kept_relations);

        let mut observed_retired_sources = BTreeSet::new();
        let mut observed_recreated_keepers = BTreeMap::new();
        let mut observed_unnamed_entities = BTreeSet::new();

        for ev in prepared.artifact().event_batch().events() {
            match ev.kind() {
                LineageEventKind::Create => {
                    assert_eq!(ev.targets().len(), 1);
                    let lin = ev.targets()[0];
                    assert!(all_known_lineages.insert(lin), "duplicate created lineage");
                    match ev.subject() {
                        LineageSubject::Entity => {
                            let e = acc
                                .changed
                                .iter()
                                .find_map(|r| match r {
                                    RecordRef::Entity(id)
                                        if staged_entity_lineage(&staged, *id) == Some(lin) =>
                                    {
                                        Some(*id)
                                    }
                                    _ => None,
                                })
                                .expect("created entity lineage installed in staged arena");
                            match (ev.name_birth(), ev.kept_name()) {
                                (Some(birth), None) => {
                                    live_entities.push((e, lin, birth.identity()));
                                }
                                (None, Some(kept)) => {
                                    observed_recreated_keepers
                                        .insert(LineageRecordTarget::Entity(e), kept);
                                    live_entities.push((e, lin, kept));
                                }
                                (None, None) => {
                                    assert!(observed_unnamed_entities.insert(e));
                                    live_unnamed_entities.push((e, lin));
                                }
                                (Some(_), Some(_)) => {
                                    panic!("Create cannot carry both name_birth and kept_name");
                                }
                            }
                        }
                        LineageSubject::Relation => {
                            let (&r, _) = prepared
                                .relation_lineages()
                                .iter()
                                .find(|(_, l)| **l == lin)
                                .expect("created relation lineage recorded");
                            match (ev.name_birth(), ev.kept_name()) {
                                (Some(birth), None) => {
                                    live_relations.push((r, lin, birth.identity()));
                                }
                                (None, Some(kept)) => {
                                    observed_recreated_keepers
                                        .insert(LineageRecordTarget::Relation(r), kept);
                                    live_relations.push((r, lin, kept));
                                }
                                _ => {
                                    panic!("postured Relation Create must carry birth or kept_name")
                                }
                            }
                        }
                    }
                }
                LineageEventKind::Retire
                | LineageEventKind::Split
                | LineageEventKind::Merge
                | LineageEventKind::Replace
                | LineageEventKind::Restore => {
                    for &src in ev.sources() {
                        let is_inplace_keeper =
                            ev.kind() == LineageEventKind::Merge && ev.targets() == [src];
                        if !is_inplace_keeper {
                            assert!(
                                acc.expected_retired.contains(&src),
                                "unexpected non-retired source {src:?} in {:?}",
                                ev.kind()
                            );
                            assert!(
                                observed_retired_sources.insert(src),
                                "retired source {src:?} appeared twice in one commit"
                            );
                        }
                    }
                    for &tgt in ev.targets() {
                        assert!(
                            all_known_lineages.contains(&tgt)
                                && !retired_lineages.contains(&tgt)
                                && !acc.expected_retired.contains(&tgt),
                            "dangling or retired target lineage {tgt:?}"
                        );
                    }
                }
            }
        }

        // Conservation invariants after EVERY multi-op commit:
        assert_eq!(observed_retired_sources, acc.expected_retired);
        assert_eq!(observed_recreated_keepers, acc.expected_recreated_keepers);
        assert_eq!(observed_unnamed_entities, acc.expected_unnamed_entities);
        retired_lineages.extend(acc.expected_retired);

        live_holders.clear();
        live_lineages.clear();
        let mut live_head_set = BTreeSet::new();
        for &(e, lin, name) in &live_entities {
            assert!(!retired_lineages.contains(&lin) && live_head_set.insert(lin));
            assert!(
                live_holders
                    .insert(name, PersistentNameHolder::Entity(e))
                    .is_none(),
                "duplicate live persistent name across entities"
            );
            live_lineages.insert(PersistentNameHolder::Entity(e), lin);
        }
        for &(r, lin, name) in &live_relations {
            assert!(!retired_lineages.contains(&lin) && live_head_set.insert(lin));
            assert!(
                live_holders
                    .insert(name, PersistentNameHolder::Relation(r))
                    .is_none(),
                "duplicate live persistent name across relations"
            );
            live_lineages.insert(PersistentNameHolder::Relation(r), lin);
        }
        for &(_, lin) in &live_unnamed_entities {
            assert!(!retired_lineages.contains(&lin) && live_head_set.insert(lin));
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn many_to_many_entity_and_relation_lineage_conservation_holds(
        batches in prop::collection::vec(prop::collection::vec(arb_step(), 1..=4), 3..=10)
    ) {
        run_multi_op_conservation_sequence(&batches);
    }
}
