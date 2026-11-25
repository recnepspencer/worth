//! Phase 5 behavioral courts for append-only undo/redo/checkout continuity,
//! multi-epoch `Split -> Restore -> Split` acyclicity, and independent
//! occurrence isolation.

use crate::facade::history::CommitId;
use crate::facade::lineage::{
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameHolder,
    PersistentNameKey, PersistentNameResolution, PersistentNameTarget,
};
use worth_relational_lineage::internal_port::lineage::index::batch::PersistentNameCommittedBatchFixturePort as _;

use super::name_index_fixtures::{
    all, exact, main_branch, plural_keys, rebuild, resolve, retired, side, unknown,
};
use super::name_world::{LineageWorld, Subject};

#[test]
fn append_only_undo_redo_checkout_continuity() {
    let mut world = LineageWorld::new();
    let mut index = PersistentNameBranchIndex::new(main_branch(), None);
    let mut snapshots: Vec<Vec<String>> = Vec::new();
    let record_step = |batch: PersistentNameCommittedBatch,
                       idx: &mut PersistentNameBranchIndex,
                       snaps: &mut Vec<Vec<String>>| {
        idx.apply(&batch).expect("commit ingests");
        let serialized: Vec<String> = batch
            .lineage_events_for_test()
            .iter()
            .map(|e| serde_json::to_string(e).expect("serialize lineage event"))
            .collect();
        snaps.push(serialized);
    };
    record_step(world.log()[0].clone(), &mut index, &mut snapshots);

    // Commit 2: Create face P
    let p_h1 = world.create(Subject::Entity, "base");
    let c2 = world.commit();
    record_step(c2, &mut index, &mut snapshots);
    let p_name = world.name_of(&p_h1);
    let p_key = PersistentNameKey::Entity(p_name);

    // Commit 3: Split P into L and R
    let pieces = world.split(p_name, &["left", "right"]);
    let c3 = world.commit();
    record_step(c3, &mut index, &mut snapshots);
    let (l_name, r_name) = (world.name_of(&pieces[0]), world.name_of(&pieces[1]));
    let (l_key, r_key) = (
        PersistentNameKey::Entity(l_name),
        PersistentNameKey::Entity(r_name),
    );

    // Commit 4: Retire L and R (delete)
    world.retire(l_name);
    world.retire(r_name);
    let c4 = world.commit();
    record_step(c4, &mut index, &mut snapshots);

    // Commit 5: Restore P (undo split+delete) -> fresh generation!
    let p_h2 = world.restore(p_name, false);
    assert_ne!(p_h2, p_h1);
    let c5 = world.commit();
    record_step(c5, &mut index, &mut snapshots);

    // Commit 6: In-commit Regenerate P -> fresh generation!
    let p_h3 = world.regenerate(p_name);
    assert_ne!(p_h3, p_h2);
    assert_ne!(p_h3, p_h1);
    let c6 = world.commit();
    record_step(c6, &mut index, &mut snapshots);

    // Commit 7: Retire P
    world.retire(p_name);
    let c7 = world.commit();
    record_step(c7, &mut index, &mut snapshots);

    // Commit 8: Restore P via birth (checkout) -> fresh generation!
    let p_h4 = world.restore(p_name, true);
    assert_ne!(p_h4, p_h3);
    assert_ne!(p_h4, p_h2);
    assert_ne!(p_h4, p_h1);
    let c8 = world.commit();
    record_step(c8, &mut index, &mut snapshots);

    // Commit 9: Split P AGAIN into new children D1, D2 -> must follow commit-9 edges only, never commit-3's dead L/R!
    let pieces2 = world.split(p_name, &["north", "south"]);
    let c9 = world.commit();
    record_step(c9, &mut index, &mut snapshots);
    let (d1_key, d2_key) = (
        PersistentNameKey::Entity(world.name_of(&pieces2[0])),
        PersistentNameKey::Entity(world.name_of(&pieces2[1])),
    );

    // Arch 8/14: ZERO prior serialized event bytes ever mutated!
    for (idx, expected_bytes) in snapshots.iter().enumerate() {
        let actual: Vec<String> = world.log()[idx]
            .lineage_events_for_test()
            .iter()
            .map(|e| serde_json::to_string(e).unwrap())
            .collect();
        assert_eq!(
            &actual, expected_bytes,
            "prior commit {idx} event bytes mutated"
        );
    }
    // Arch 18: rebuild equivalence!
    assert_eq!(rebuild(world.log()), index);

    // Historical basis accuracy across every single epoch:
    assert_eq!(resolve(&index, p_key, 1, &exact()), unknown(p_key));
    assert!(matches!(
        resolve(&index, p_key, 2, &exact()),
        Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity { entity, .. }))
            if PersistentNameHolder::Entity(entity) == p_h1
    ));
    assert_eq!(plural_keys(&resolve(&index, p_key, 3, &all())), {
        let mut expected = vec![l_key, r_key];
        expected.sort_unstable();
        expected
    });
    assert_eq!(
        resolve(&index, p_key, 3, &side("left")),
        resolve(&index, l_key, 3, &exact())
    );
    assert_eq!(resolve(&index, p_key, 4, &exact()), retired(4));
    assert_eq!(resolve(&index, r_key, 4, &exact()), retired(4));
    assert!(matches!(
        resolve(&index, p_key, 5, &exact()),
        Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity { entity, .. }))
            if PersistentNameHolder::Entity(entity) == p_h2
    ));
    assert_eq!(
        index
            .resolve(p_key, CommitId(5), &exact())
            .metrics
            .edge_visits,
        0
    );
    assert!(matches!(
        resolve(&index, p_key, 6, &exact()),
        Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity { entity, .. }))
            if PersistentNameHolder::Entity(entity) == p_h3
    ));
    assert_eq!(resolve(&index, p_key, 7, &exact()), retired(7));
    assert!(matches!(
        resolve(&index, p_key, 8, &exact()),
        Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity { entity, .. }))
            if PersistentNameHolder::Entity(entity) == p_h4
    ));
    assert_eq!(plural_keys(&resolve(&index, p_key, 9, &all())), {
        let mut expected = vec![d1_key, d2_key];
        expected.sort_unstable();
        expected
    });
    assert_eq!(
        resolve(&index, p_key, 9, &side("north")),
        resolve(&index, d1_key, 9, &exact())
    );
}

#[test]
fn independent_occurrences_produce_distinct_names() {
    use super::posture_fixtures::{
        cad_scheme_registry, empty_working_state, face_birth, push_entity, push_relation,
        receipt_at, seam_birth,
    };
    use crate::facade::identity::VersionId;
    use crate::facade::lineage::{LineagePostureClaim, LineageRecordTarget, LineageSubject};
    use crate::facade::transactions::RecordRef;
    use crate::lineage::authority::finalize_postures_in_working_state_for_test;
    use crate::lineage::authority::posture_fixture::LineageCommitPostureInput;
    use crate::runtime::LineageIdentityAllocator;

    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();
    let mut staged = empty_working_state();
    let f1 = push_entity(&mut staged, VersionId(1));
    let f2 = push_entity(&mut staged, VersionId(1));
    let s1 = push_relation(&mut staged, VersionId(1), f1, f2);
    let s2 = push_relation(&mut staged, VersionId(1), f1, f2);
    let prepared = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[
            RecordRef::Entity(f1),
            RecordRef::Entity(f2),
            RecordRef::Relation(s1),
            RecordRef::Relation(s2),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry,
            postures: vec![
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(f1),
                    birth: face_birth("occ.alpha", "authored", vec![], "identical-payload", f1),
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(f2),
                    birth: face_birth("occ.beta", "authored", vec![], "identical-payload", f2),
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Relation(s1),
                    birth: seam_birth("occ.alpha", "seam", vec![], "identical-payload", s1),
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Relation(s2),
                    birth: seam_birth("occ.beta", "seam", vec![], "identical-payload", s2),
                },
            ],
            ..Default::default()
        },
    )
    .expect("independent occurrences with identical payloads finalize cleanly");
    let events = prepared.artifact().event_batch().events();
    let entity_names: Vec<_> = events
        .iter()
        .filter(|e| e.subject() == LineageSubject::Entity)
        .map(|e| e.name_birth().unwrap().identity())
        .collect();
    let rel_names: Vec<_> = events
        .iter()
        .filter(|e| e.subject() == LineageSubject::Relation)
        .map(|e| e.name_birth().unwrap().identity())
        .collect();
    assert_eq!(entity_names.len(), 2);
    assert_ne!(entity_names[0], entity_names[1]);
    assert_eq!(rel_names.len(), 2);
    assert_ne!(rel_names[0], rel_names[1]);
}
