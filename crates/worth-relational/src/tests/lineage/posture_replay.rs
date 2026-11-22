use std::collections::BTreeMap;

use crate::facade::identity::VersionId;
use crate::facade::lineage::{
    LineageCommitPostureInput, LineageEventKind, LineageEventRecord, LineageMergeKeeper,
    LineagePostureClaim, LineageRecordTarget, LineageSplitChild, LineageSubject,
    PersistentNameHolder,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::runtime::LineageIdentityAllocator;

use super::posture_fixtures::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, push_relation, receipt_at,
    retire_entity, retire_relation, seam_birth, staged_entity_lineage,
};

fn run_replayable_sequence() -> Vec<LineageEventRecord> {
    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();
    let mut staged = empty_working_state();
    let mut all_events = Vec::new();

    let f1 = push_entity(&mut staged, VersionId(1));
    let f2 = push_entity(&mut staged, VersionId(1));
    let s1 = push_relation(&mut staged, VersionId(1), f1, f2);
    let c1 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[
            RecordRef::Entity(f1),
            RecordRef::Entity(f2),
            RecordRef::Relation(s1),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders: BTreeMap::new(),
            live_lineages: BTreeMap::new(),
            postures: vec![
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(f1),
                    birth: face_birth("occ.1", "authored", vec![], "f1", f1),
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(f2),
                    birth: face_birth("occ.1", "authored", vec![], "f2", f2),
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Relation(s1),
                    birth: seam_birth("occ.1", "seam", vec![], "s1", s1),
                },
            ],
        },
    )
    .unwrap();
    let c1_ev = c1.artifact().event_batch().events();
    all_events.extend(c1_ev.iter().cloned());

    let f1_lin = staged_entity_lineage(&staged, f1).unwrap();
    let f2_lin = staged_entity_lineage(&staged, f2).unwrap();
    let s1_lin = c1.relation_lineages()[&s1];
    let name_of = |lin| {
        c1_ev
            .iter()
            .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [lin])
            .unwrap()
            .name_birth()
            .unwrap()
            .identity()
    };
    let (f1_name, f2_name, s1_name) = (name_of(f1_lin), name_of(f2_lin), name_of(s1_lin));

    // Commit 2: Recreated-keeper entity Merge(KeepParent(f1_name)) AND relation Split.
    retire_entity(&mut staged, f1, VersionId(2));
    retire_entity(&mut staged, f2, VersionId(2));
    let f_replacement = push_entity(&mut staged, VersionId(2));
    retire_relation(&mut staged, s1, VersionId(2));
    let s1_a = push_relation(&mut staged, VersionId(2), f_replacement, f_replacement);
    let s1_b = push_relation(&mut staged, VersionId(2), f_replacement, f_replacement);

    let c2 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[],
        &[
            RecordRef::Entity(f1),
            RecordRef::Entity(f2),
            RecordRef::Entity(f_replacement),
            RecordRef::Relation(s1),
            RecordRef::Relation(s1_a),
            RecordRef::Relation(s1_b),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry,
            live_holders: BTreeMap::from([
                (f1_name, PersistentNameHolder::Entity(f1)),
                (f2_name, PersistentNameHolder::Entity(f2)),
                (s1_name, PersistentNameHolder::Relation(s1)),
            ]),
            live_lineages: BTreeMap::from([
                (PersistentNameHolder::Entity(f1), f1_lin),
                (PersistentNameHolder::Entity(f2), f2_lin),
                (PersistentNameHolder::Relation(s1), s1_lin),
            ]),
            postures: vec![
                LineagePostureClaim::Merge {
                    parents: vec![
                        (LineageRecordTarget::Entity(f1), f1_name),
                        (LineageRecordTarget::Entity(f2), f2_name),
                    ],
                    target: LineageRecordTarget::Entity(f_replacement),
                    keeper: LineageMergeKeeper::KeepParent(f1_name),
                },
                LineagePostureClaim::Split {
                    parent_target: LineageRecordTarget::Relation(s1),
                    parent_name: s1_name,
                    children: vec![
                        LineageSplitChild {
                            target: LineageRecordTarget::Relation(s1_a),
                            birth: seam_birth("occ.2", "seam_piece", vec![s1_name], "a", s1_a),
                        },
                        LineageSplitChild {
                            target: LineageRecordTarget::Relation(s1_b),
                            birth: seam_birth("occ.2", "seam_piece", vec![s1_name], "b", s1_b),
                        },
                    ],
                },
            ],
        },
    )
    .expect("recreated-keeper merge and relation split succeed");

    let c2_ev = c2.artifact().event_batch().events();
    let f_rep_lin = staged_entity_lineage(&staged, f_replacement).unwrap();
    let rep_create = c2_ev
        .iter()
        .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [f_rep_lin])
        .expect("recreated keeper emits Create with kept_name");
    assert_eq!(rep_create.kept_name(), Some(f1_name));
    assert!(rep_create.name_birth().is_none());

    let merge_ev = c2_ev
        .iter()
        .find(|e| e.kind() == LineageEventKind::Merge && e.subject() == LineageSubject::Entity)
        .expect("recreated keeper emits Merge");
    assert_eq!(merge_ev.targets(), &[f_rep_lin]);
    assert_eq!(merge_ev.kept_name(), Some(f1_name));

    all_events.extend(c2_ev.iter().cloned());
    all_events
}

#[test]
fn posture_replay_produces_identical_lineage_events_and_supports_recreated_keeper() {
    let run1 = run_replayable_sequence();
    let run2 = run_replayable_sequence();
    assert_eq!(run1, run2);
}
