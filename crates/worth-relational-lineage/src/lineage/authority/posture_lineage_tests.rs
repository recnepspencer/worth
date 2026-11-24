use std::collections::BTreeMap;
use worth_relational_contracts::internal_port::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalizationFixturePort as _;

use super::finalize_postures_in_working_state_for_test;
use super::posture_fixture::LineageCommitPostureInput;
use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::lineage::data::{
    LineageDecisionKind, LineageEventKind, LineageMergeKeeper, LineagePostureClaim,
    LineageRecordTarget, LineageSplitChild, LineageSubject, PersistentNameHolder,
};
use crate::contracts::transactions::data::intents::mutation_intent::{
    DeleteRelationIntent, MutationIntent, RelationMutationIntent,
};
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::runtime::state::subsystems::lineage_identity::LineageIdentityAllocator;

use super::posture_fixture::records::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, push_relation, receipt_at,
    retire_entity, retire_relation, seam_birth, staged_entity_lineage,
};

#[test]
fn entity_and_relation_split_merge_happy_path_emits_canonical_lineage() {
    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();

    // Commit 1: Create 2 faces (f_a, f_b) and 2 parallel seam relations (s1, s2)
    // between the exact same (f_a, f_b) endpoints with distinct discriminators.
    let mut staged = empty_working_state();
    let f_a = push_entity(&mut staged, VersionId(1));
    let f_b = push_entity(&mut staged, VersionId(1));
    let s1 = push_relation(&mut staged, VersionId(1), f_a, f_b);
    let s2 = push_relation(&mut staged, VersionId(1), f_a, f_b);

    let c1_input = LineageCommitPostureInput {
        scheme_registry: registry.clone(),
        live_holders: BTreeMap::new(),
        live_lineages: BTreeMap::new(),
        retired_lineages: BTreeMap::new(),
        postures: vec![
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(f_a),
                birth: face_birth("occ.base", "authored", vec![], "face-a", f_a),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(f_b),
                birth: face_birth("occ.base", "authored", vec![], "face-b", f_b),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(s1),
                birth: seam_birth("occ.base", "seam", vec![], "seam-top", s1),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(s2),
                birth: seam_birth("occ.base", "seam", vec![], "seam-bottom", s2),
            },
        ],
    };
    let c1 = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[
            RecordRef::Entity(f_a),
            RecordRef::Entity(f_b),
            RecordRef::Relation(s1),
            RecordRef::Relation(s2),
        ],
        &c1_input,
    )
    .expect("initial entity + parallel relation births succeed");

    let c1_events = c1.artifact().event_batch().events();
    assert_eq!(c1_events.len(), 4);
    let entity_creates: Vec<_> = c1_events
        .iter()
        .filter(|e| e.subject() == LineageSubject::Entity && e.kind() == LineageEventKind::Create)
        .collect();
    let rel_creates: Vec<_> = c1_events
        .iter()
        .filter(|e| e.subject() == LineageSubject::Relation && e.kind() == LineageEventKind::Create)
        .collect();
    assert_eq!(entity_creates.len(), 2);
    assert_eq!(rel_creates.len(), 2);

    let f_a_lin = staged_entity_lineage(&staged, f_a).unwrap();
    let f_b_lin = staged_entity_lineage(&staged, f_b).unwrap();
    let s1_lin = c1.relation_lineages_for_test()[&s1];
    let s2_lin = c1.relation_lineages_for_test()[&s2];

    let name_for_lineage =
        |events: &[crate::contracts::lineage::data::LineageEventRecord],
         lin: crate::contracts::identity::data::primitives::LineageId| {
            events
                .iter()
                .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [lin])
                .and_then(|e| e.name_birth())
                .expect("create event carries birth")
                .identity()
        };
    let f_a_name = name_for_lineage(c1_events, f_a_lin);
    let f_b_name = name_for_lineage(c1_events, f_b_lin);
    let s1_name = name_for_lineage(c1_events, s1_lin);
    let s2_name = name_for_lineage(c1_events, s2_lin);
    // Parallel same-role relations between identical (f_a, f_b) have distinct names.
    assert_ne!(s1_name, s2_name);

    let mut live_holders = BTreeMap::from([
        (f_a_name, PersistentNameHolder::Entity(f_a)),
        (f_b_name, PersistentNameHolder::Entity(f_b)),
        (s1_name, PersistentNameHolder::Relation(s1)),
        (s2_name, PersistentNameHolder::Relation(s2)),
    ]);
    let mut live_lineages = BTreeMap::from([
        (PersistentNameHolder::Entity(f_a), f_a_lin),
        (PersistentNameHolder::Entity(f_b), f_b_lin),
        (PersistentNameHolder::Relation(s1), s1_lin),
        (PersistentNameHolder::Relation(s2), s2_lin),
    ]);

    // Commit 2: Split f_a into (f_a1, f_a2) AND split s1 into (s1_a, s1_b),
    // leaving parallel relation s2 preserved and untouched.
    retire_entity(&mut staged, f_a, VersionId(2));
    let f_a1 = push_entity(&mut staged, VersionId(2));
    let f_a2 = push_entity(&mut staged, VersionId(2));
    retire_relation(&mut staged, s1, VersionId(2));
    let s1_a = push_relation(&mut staged, VersionId(2), f_a1, f_b);
    let s1_b = push_relation(&mut staged, VersionId(2), f_a2, f_b);

    let c2_input = LineageCommitPostureInput {
        scheme_registry: registry.clone(),
        live_holders: live_holders.clone(),
        live_lineages: live_lineages.clone(),
        retired_lineages: BTreeMap::new(),
        postures: vec![
            LineagePostureClaim::Split {
                parent_target: LineageRecordTarget::Entity(f_a),
                parent_name: f_a_name,
                children: vec![
                    LineageSplitChild {
                        target: LineageRecordTarget::Entity(f_a1),
                        birth: face_birth("occ.split.1", "cut_piece", vec![f_a_name], "left", f_a1),
                    },
                    LineageSplitChild {
                        target: LineageRecordTarget::Entity(f_a2),
                        birth: face_birth(
                            "occ.split.1",
                            "cut_piece",
                            vec![f_a_name],
                            "right",
                            f_a2,
                        ),
                    },
                ],
            },
            LineagePostureClaim::Split {
                parent_target: LineageRecordTarget::Relation(s1),
                parent_name: s1_name,
                children: vec![
                    LineageSplitChild {
                        target: LineageRecordTarget::Relation(s1_a),
                        birth: seam_birth(
                            "occ.split.1",
                            "seam_piece",
                            vec![s1_name],
                            "seam-left",
                            s1_a,
                        ),
                    },
                    LineageSplitChild {
                        target: LineageRecordTarget::Relation(s1_b),
                        birth: seam_birth(
                            "occ.split.1",
                            "seam_piece",
                            vec![s1_name],
                            "seam-right",
                            s1_b,
                        ),
                    },
                ],
            },
            LineagePostureClaim::Preserve {
                target: LineageRecordTarget::Relation(s2),
                name: s2_name,
            },
        ],
    };
    let c2 = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[],
        &[
            RecordRef::Entity(f_a),
            RecordRef::Entity(f_a1),
            RecordRef::Entity(f_a2),
            RecordRef::Relation(s1),
            RecordRef::Relation(s1_a),
            RecordRef::Relation(s1_b),
        ],
        &c2_input,
    )
    .expect("entity + relation split succeeds");

    let c2_events = c2.artifact().event_batch().events();
    let f_a1_lin = staged_entity_lineage(&staged, f_a1).unwrap();
    let f_a2_lin = staged_entity_lineage(&staged, f_a2).unwrap();
    let s1_a_lin = c2.relation_lineages_for_test()[&s1_a];
    let s1_b_lin = c2.relation_lineages_for_test()[&s1_b];

    let entity_split = c2_events
        .iter()
        .find(|e| e.kind() == LineageEventKind::Split && e.subject() == LineageSubject::Entity)
        .expect("entity Split event emitted");
    assert_eq!(entity_split.sources(), &[f_a_lin]);
    assert_eq!(entity_split.targets(), &[f_a1_lin, f_a2_lin]);

    let rel_split = c2_events
        .iter()
        .find(|e| e.kind() == LineageEventKind::Split && e.subject() == LineageSubject::Relation)
        .expect("relation Split event emitted");
    assert_eq!(rel_split.sources(), &[s1_lin]);
    assert_eq!(rel_split.targets(), &[s1_a_lin, s1_b_lin]);
    // Parallel seam s2 was never retired or referenced by s1's split.
    assert!(c2_events.iter().all(|e| !e.sources().contains(&s2_lin)));
    assert!(
        c2.artifact()
            .decision_log()
            .decisions()
            .iter()
            .filter(|d| d.kind() == &LineageDecisionKind::SplitAccepted)
            .count()
            == 2
    );

    let f_a1_name = name_for_lineage(c2_events, f_a1_lin);
    let f_a2_name = name_for_lineage(c2_events, f_a2_lin);
    let s1_a_name = name_for_lineage(c2_events, s1_a_lin);
    let s1_b_name = name_for_lineage(c2_events, s1_b_lin);
    assert_ne!(f_a1_name, f_a2_name);
    assert_ne!(s1_a_name, s1_b_name);
    assert_ne!(s1_a_name, s2_name);
    assert_ne!(s1_b_name, s2_name);

    // Update basis maps after commit 2.
    live_holders.remove(&f_a_name);
    live_holders.remove(&s1_name);
    live_holders.insert(f_a1_name, PersistentNameHolder::Entity(f_a1));
    live_holders.insert(f_a2_name, PersistentNameHolder::Entity(f_a2));
    live_holders.insert(s1_a_name, PersistentNameHolder::Relation(s1_a));
    live_holders.insert(s1_b_name, PersistentNameHolder::Relation(s1_b));
    live_lineages.remove(&PersistentNameHolder::Entity(f_a));
    live_lineages.remove(&PersistentNameHolder::Relation(s1));
    live_lineages.insert(PersistentNameHolder::Entity(f_a1), f_a1_lin);
    live_lineages.insert(PersistentNameHolder::Entity(f_a2), f_a2_lin);
    live_lineages.insert(PersistentNameHolder::Relation(s1_a), s1_a_lin);
    live_lineages.insert(PersistentNameHolder::Relation(s1_b), s1_b_lin);

    // Commit 3:
    // - Merge faces (f_a1, f_a2) with KeepParent(f_a1_name) preserved in place (f_a2 retired).
    // - Merge relations (s1_a, s1_b) with MintNew into s_merged (both parents retired).
    retire_entity(&mut staged, f_a2, VersionId(3));
    retire_relation(&mut staged, s1_a, VersionId(3));
    retire_relation(&mut staged, s1_b, VersionId(3));
    let s_merged = push_relation(&mut staged, VersionId(3), f_a1, f_b);

    let c3_input = LineageCommitPostureInput {
        scheme_registry: registry.clone(),
        live_holders: live_holders.clone(),
        live_lineages: live_lineages.clone(),
        retired_lineages: BTreeMap::new(),
        postures: vec![
            LineagePostureClaim::Merge {
                parents: vec![
                    (LineageRecordTarget::Entity(f_a1), f_a1_name),
                    (LineageRecordTarget::Entity(f_a2), f_a2_name),
                ],
                target: LineageRecordTarget::Entity(f_a1),
                keeper: LineageMergeKeeper::KeepParent(f_a1_name),
            },
            LineagePostureClaim::Merge {
                parents: vec![
                    (LineageRecordTarget::Relation(s1_a), s1_a_name),
                    (LineageRecordTarget::Relation(s1_b), s1_b_name),
                ],
                target: LineageRecordTarget::Relation(s_merged),
                keeper: LineageMergeKeeper::MintNew {
                    birth: seam_birth(
                        "occ.merge.1",
                        "merged_seam",
                        vec![s1_b_name, s1_a_name],
                        "union-seam",
                        s_merged,
                    ),
                },
            },
        ],
    };
    let c3 = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(3, 3),
        &[],
        &[
            RecordRef::Entity(f_a2),
            RecordRef::Relation(s1_a),
            RecordRef::Relation(s1_b),
            RecordRef::Relation(s_merged),
        ],
        &c3_input,
    )
    .expect("entity KeepParent merge + relation MintNew merge succeed");

    let c3_events = c3.artifact().event_batch().events();
    let entity_merge = c3_events
        .iter()
        .find(|e| e.kind() == LineageEventKind::Merge && e.subject() == LineageSubject::Entity)
        .expect("entity Merge event emitted");
    let mut expected_f_sources = vec![f_a1_lin, f_a2_lin];
    expected_f_sources.sort_unstable();
    assert_eq!(entity_merge.sources(), expected_f_sources.as_slice());
    assert_eq!(entity_merge.targets(), &[f_a1_lin]);
    assert_eq!(entity_merge.kept_name(), Some(f_a1_name));

    let s_merged_lin = c3.relation_lineages_for_test()[&s_merged];
    let rel_merge = c3_events
        .iter()
        .find(|e| e.kind() == LineageEventKind::Merge && e.subject() == LineageSubject::Relation)
        .expect("relation Merge event emitted");
    let mut expected_s_sources = vec![s1_a_lin, s1_b_lin];
    expected_s_sources.sort_unstable();
    assert_eq!(rel_merge.sources(), expected_s_sources.as_slice());
    assert_eq!(rel_merge.targets(), &[s_merged_lin]);
    assert_eq!(rel_merge.kept_name(), None);

    let merged_create = c3_events
        .iter()
        .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [s_merged_lin])
        .expect("MintNew merge emitted Create event");
    let merged_birth = merged_create
        .name_birth()
        .expect("MintNew merge carries birth record");
    let mut expected_parent_names = vec![s1_a_name, s1_b_name];
    expected_parent_names.sort_unstable();
    assert_eq!(
        merged_birth.parts().parent_identities(),
        expected_parent_names.as_slice()
    );
    assert_ne!(merged_birth.identity(), s1_a_name);
    assert_ne!(merged_birth.identity(), s1_b_name);
    assert_eq!(
        c3.artifact()
            .decision_log()
            .decisions()
            .iter()
            .filter(|d| d.kind() == &LineageDecisionKind::MergeAccepted)
            .count(),
        2
    );

    // Commit 4: Tracked RelationMutationIntent::Delete without explicit Retire
    // posture closes the MutationIntent::Relation(_) => {} gap cleanly.
    retire_relation(&mut staged, s2, VersionId(4));
    let c4_input = LineageCommitPostureInput {
        scheme_registry: registry,
        live_holders,
        live_lineages,
        retired_lineages: BTreeMap::new(),
        postures: Vec::new(),
    };
    let c4 = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(4, 4),
        &[MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent { relation_id: s2 },
        ))],
        &[RecordRef::Relation(s2)],
        &c4_input,
    )
    .expect("tracked relation delete emits Retire lineage event");
    let c4_events = c4.artifact().event_batch().events();
    assert_eq!(c4_events.len(), 1);
    assert_eq!(c4_events[0].kind(), LineageEventKind::Retire);
    assert_eq!(c4_events[0].subject(), LineageSubject::Relation);
    assert_eq!(c4_events[0].sources(), &[s2_lin]);
    assert!(c4_events[0].targets().is_empty());
}
