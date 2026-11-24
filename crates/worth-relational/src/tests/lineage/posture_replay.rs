use std::collections::BTreeMap;
use worth_relational_contracts::internal_port::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalizationFixturePort as _;

use crate::facade::identity::VersionId;
use crate::facade::lineage::{
    LineageEventKind, LineageEventRecord, LineageMergeKeeper, LineagePostureClaim,
    LineageRecordTarget, LineageSplitChild, LineageSubject, PersistentNameHolder,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::lineage::authority::posture_fixture::LineageCommitPostureInput;
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
            retired_lineages: BTreeMap::new(),
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
    let s1_lin = c1.relation_lineages_for_test()[&s1];
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
            retired_lineages: BTreeMap::new(),
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

#[test]
fn default_empty_posture_path_emits_exact_create_replace_retire_and_zero_unpostured_relations() {
    use crate::facade::identity::{KindId, PartitionId};
    use crate::facade::lineage::LineageDecisionKind;
    use crate::facade::transactions::{
        AspectFieldPatch, DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent,
        EntitySpec, MutationIntent, RelationMutationIntent, ReplaceEntityIntent,
    };
    use crate::symbols::data::ClientKey;

    let allocator = LineageIdentityAllocator::new();
    let mut staged = empty_working_state();
    let empty_input = LineageCommitPostureInput::default();

    // Commit 1: Two entities passed in reverse EntityId order + one unpostured
    // fieldless relation. Default path must sort entity Creates by EntityId and
    // emit zero relation events (preserving Home PR C compatibility).
    let e0 = push_entity(&mut staged, VersionId(1));
    let e1 = push_entity(&mut staged, VersionId(1));
    let r0 = push_relation(&mut staged, VersionId(1), e0, e1);
    let c1 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[
            RecordRef::Relation(r0),
            RecordRef::Entity(e1),
            RecordRef::Entity(e0),
        ],
        &empty_input,
    )
    .expect("default commit 1 finalizes cleanly");
    let e0_lin = staged_entity_lineage(&staged, e0).unwrap();
    let e1_lin = staged_entity_lineage(&staged, e1).unwrap();
    let c1_ev = c1.artifact().event_batch().events();
    assert_eq!(c1_ev.len(), 2, "unpostured relation must emit no lineage");
    assert!(c1.relation_lineages_for_test().is_empty());
    assert_eq!(c1_ev[0].kind(), LineageEventKind::Create);
    assert_eq!(c1_ev[0].targets(), &[e0_lin]);
    assert_eq!(c1_ev[1].kind(), LineageEventKind::Create);
    assert_eq!(c1_ev[1].targets(), &[e1_lin]);
    assert!(
        e0_lin < e1_lin,
        "Creates must be assigned in ascending EntityId order"
    );

    // Commit 2: Default Delete(e0), Replace(e1 -> e2), and unpostured
    // Delete(r0) emit exact Create(e2), Retire(e0), Replace(e1 -> e2) and
    // zero relation events.
    retire_entity(&mut staged, e0, VersionId(2));
    retire_entity(&mut staged, e1, VersionId(2));
    let e2 = push_entity(&mut staged, VersionId(2));
    retire_relation(&mut staged, r0, VersionId(2));
    let merged_plan = vec![
        MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
            entity_id: e0,
        })),
        MutationIntent::Entity(EntityMutationIntent::Replace(ReplaceEntityIntent {
            entity_id: e1,
            replacement: EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("e2"),
                fields: AspectFieldPatch::default(),
            },
        })),
        MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
            relation_id: r0,
        })),
    ];
    let c2 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &merged_plan,
        &[
            RecordRef::Entity(e0),
            RecordRef::Entity(e1),
            RecordRef::Entity(e2),
            RecordRef::Relation(r0),
        ],
        &empty_input,
    )
    .expect("default commit 2 finalizes cleanly");
    let e2_lin = staged_entity_lineage(&staged, e2).unwrap();
    let c2_ev = c2.artifact().event_batch().events();
    assert_eq!(c2_ev.len(), 3);
    assert_eq!(c2_ev[0].kind(), LineageEventKind::Create);
    assert_eq!(c2_ev[0].targets(), &[e2_lin]);
    assert_eq!(c2_ev[1].kind(), LineageEventKind::Retire);
    assert_eq!(c2_ev[1].sources(), &[e0_lin]);
    assert_eq!(c2_ev[2].kind(), LineageEventKind::Replace);
    assert_eq!(c2_ev[2].sources(), &[e1_lin]);
    assert_eq!(c2_ev[2].targets(), &[e2_lin]);
    assert!(c2_ev.iter().all(|e| e.subject() == LineageSubject::Entity));
    let decisions: Vec<_> = c2
        .artifact()
        .decision_log()
        .decisions()
        .iter()
        .map(|d| d.kind().clone())
        .collect();
    assert_eq!(
        decisions,
        vec![
            LineageDecisionKind::CreateAccepted,
            LineageDecisionKind::RetireAccepted,
            LineageDecisionKind::ReplaceAccepted,
        ]
    );
}

#[test]
fn unpostured_replace_routes_to_free_entity_and_never_hijacks_posture_claimed_entity() {
    use crate::facade::identity::{KindId, PartitionId};
    use crate::facade::transactions::{
        AspectFieldPatch, EntityMutationIntent, EntitySpec, MutationIntent, ReplaceEntityIntent,
    };
    use crate::symbols::data::ClientKey;

    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();
    let mut staged = empty_working_state();

    // Commit 1: seed basis entity e0 under the default unpostured path.
    let e0 = push_entity(&mut staged, VersionId(1));
    finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[RecordRef::Entity(e0)],
        &LineageCommitPostureInput::default(),
    )
    .expect("seed commit succeeds");
    let e0_lin = staged_entity_lineage(&staged, e0).unwrap();

    // Commit 2: retire e0 with an unpostured Replace intent, create `e_claimed`
    // (claimed by an explicit Create posture) BEFORE free `e_free` of the same kind.
    retire_entity(&mut staged, e0, VersionId(2));
    let e_claimed = push_entity(&mut staged, VersionId(2));
    let e_free = push_entity(&mut staged, VersionId(2));
    let merged_plan = vec![MutationIntent::Entity(EntityMutationIntent::Replace(
        ReplaceEntityIntent {
            entity_id: e0,
            replacement: EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("e_free"),
                fields: AspectFieldPatch::default(),
            },
        },
    ))];
    let c2 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &merged_plan,
        &[
            RecordRef::Entity(e0),
            RecordRef::Entity(e_claimed),
            RecordRef::Entity(e_free),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry,
            live_holders: BTreeMap::new(),
            live_lineages: BTreeMap::from([(PersistentNameHolder::Entity(e0), e0_lin)]),
            retired_lineages: BTreeMap::new(),
            postures: vec![LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(e_claimed),
                birth: face_birth("occ.r3", "authored", vec![], "claimed", e_claimed),
            }],
        },
    )
    .expect("Replace must route to free created entity without hijacking claimed entity");

    let claimed_lin = staged_entity_lineage(&staged, e_claimed).unwrap();
    let free_lin = staged_entity_lineage(&staged, e_free).unwrap();
    assert_ne!(claimed_lin, free_lin);

    let events = c2.artifact().event_batch().events();
    let claimed_create = events
        .iter()
        .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [claimed_lin])
        .expect("claimed entity receives posture Create event");
    assert!(claimed_create.name_birth().is_some());

    let free_create = events
        .iter()
        .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [free_lin])
        .expect("free entity receives default Create event");
    assert!(free_create.name_birth().is_none());
    assert!(free_create.kept_name().is_none());

    let replace_events: Vec<_> = events
        .iter()
        .filter(|e| e.kind() == LineageEventKind::Replace)
        .collect();
    assert_eq!(replace_events.len(), 1);
    assert_eq!(replace_events[0].sources(), &[e0_lin]);
    assert_eq!(
        replace_events[0].targets(),
        &[free_lin],
        "unpostured Replace must pair with free entity, never posture-claimed entity"
    );
}
