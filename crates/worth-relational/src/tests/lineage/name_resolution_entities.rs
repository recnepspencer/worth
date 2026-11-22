//! Entity resolution courts over real Phase 2 finalization batches.

use crate::facade::history::{BranchId, CommitId};
use crate::facade::lineage::{
    PersistentNameBranchIndex, PersistentNameKey, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameTarget,
};

use super::name_group_world::member;
use super::name_index_fixtures::{
    all, ambiguous, bounded, exact, exact_key, main_branch, metrics, no_match, plural_keys, probes,
    rebuild, resolve, retired, side, unknown,
};
use super::name_world::{side_of, Keep, LineageWorld, Subject};

#[test]
fn entity_split_retire_merge_lifecycle_resolves_at_head_and_every_historical_basis() {
    let mut world = LineageWorld::new();
    let face = world.create(Subject::Entity, "f");
    let cutter_one = world.create(Subject::Entity, "c1");
    let cutter_two = world.create(Subject::Entity, "c2");
    world.commit(); // 2
    let (f, c1, c2) = (
        world.name_of(&face),
        world.name_of(&cutter_one),
        world.name_of(&cutter_two),
    );
    let pieces = world.split(f, &["left", "right"]);
    world.bind_carriers(&pieces[0], &[c1]);
    world.bind_carriers(&pieces[1], &[c1, c2]);
    world.commit(); // 3
    let (left, right) = (world.key_of(&pieces[0]), world.key_of(&pieces[1]));
    let held = world.key_of(&face);
    let index = rebuild(world.log());

    // Two live pieces: Exact refuses with both, never picking the least one.
    assert_eq!(
        resolve(&index, held, 3, &exact()),
        ambiguous(vec![left, right])
    );
    assert_eq!(plural_keys(&resolve(&index, held, 3, &all())), {
        let mut both = vec![left, right];
        both.sort_unstable();
        both
    });
    assert_eq!(exact_key(&resolve(&index, held, 3, &side("left"))), left);
    assert_eq!(exact_key(&resolve(&index, held, 3, &side("right"))), right);
    assert_eq!(
        resolve(&index, held, 3, &side("top")),
        no_match(vec![left, right])
    );
    assert_eq!(
        resolve(&index, held, 3, &bounded(&[c1])),
        ambiguous(vec![left, right])
    );
    assert_eq!(exact_key(&resolve(&index, held, 3, &bounded(&[c2]))), right);
    assert_eq!(
        exact_key(&resolve(&index, held, 3, &bounded(&[c1, c2]))),
        right
    );
    assert_eq!(
        resolve(&index, held, 3, &bounded(&[member(9)])),
        no_match(vec![left, right])
    );
    // Before the split the face itself answers; before its birth it is unknown.
    assert_eq!(exact_key(&resolve(&index, held, 2, &exact())), held);
    assert_eq!(resolve(&index, held, 1, &all()), unknown(held));
    // A live held name is one index probe and no lineage walk.
    assert_eq!(metrics(&index, left, 3), probes(1, 0, 0));
    assert_eq!(metrics(&index, held, 3), probes(3, 1, 2));

    world.retire(world.name_of(&pieces[1]));
    world.commit(); // 4
    let survivor = world.name_of(&pieces[0]);
    let merged = world.merge(&[survivor, c1], Keep::MintNew);
    world.commit(); // 5
    let merged_key = world.key_of(&merged);
    let index = rebuild(world.log());
    assert_eq!(exact_key(&resolve(&index, held, 4, &exact())), left);
    assert_eq!(
        resolve(&index, held, 4, &side("right")),
        no_match(vec![left])
    );
    assert_eq!(resolve(&index, right, 4, &exact()), retired(4));
    assert_eq!(exact_key(&resolve(&index, held, 5, &exact())), merged_key);
    assert_eq!(
        exact_key(&resolve(&index, world.key_of(&cutter_one), 5, &all())),
        merged_key
    );
    // Historical bases keep their own answers after later commits.
    assert_eq!(
        resolve(&index, held, 3, &exact()),
        ambiguous(vec![left, right])
    );
    assert_eq!(exact_key(&resolve(&index, held, 2, &exact())), held);

    world.retire(world.name_of(&merged));
    world.commit(); // 6
    let index = rebuild(world.log());
    for selection in [exact(), all(), side("left"), bounded(&[c1])] {
        assert_eq!(resolve(&index, held, 6, &selection), retired(6));
    }
    assert_eq!(resolve(&index, right, 6, &exact()), retired(4));
    assert_eq!(exact_key(&resolve(&index, held, 5, &exact())), merged_key);
}

#[test]
fn keep_parent_merges_keep_the_keeper_name_in_place_and_on_a_recreated_record() {
    let mut world = LineageWorld::new();
    let [a, b, c] = ["a", "b", "c"].map(|label| world.create(Subject::Entity, label));
    world.commit(); // 2
    let (a_name, b_name, c_name) = (world.name_of(&a), world.name_of(&b), world.name_of(&c));
    world.merge(&[a_name, b_name], Keep::InPlace);
    world.commit(); // 3
    let recreated = world.merge(&[a_name, c_name], Keep::Recreated);
    world.commit(); // 4
    let index = rebuild(world.log());
    let keeper = world.key_of(&a);

    let in_place = resolve(&index, world.key_of(&b), 3, &exact());
    assert_eq!(exact_key(&in_place), keeper);
    let Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity { entity, .. })) = in_place
    else {
        panic!("expected the in-place keeper entity");
    };
    let crate::facade::lineage::PersistentNameHolder::Entity(original) = a else {
        unreachable!()
    };
    assert_eq!(entity, original, "an in-place keeper keeps its record");

    let Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity { entity, .. })) =
        resolve(&index, keeper, 4, &exact())
    else {
        panic!("expected the recreated keeper entity");
    };
    let crate::facade::lineage::PersistentNameHolder::Entity(moved) = recreated else {
        unreachable!()
    };
    assert_eq!(
        entity, moved,
        "a recreated keeper moves its name to the new record"
    );
    assert_eq!(
        exact_key(&resolve(&index, world.key_of(&c), 4, &exact())),
        keeper
    );
    // At the earlier basis the keeper still designates its original record.
    let Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity { entity, .. })) =
        resolve(&index, keeper, 3, &exact())
    else {
        panic!("expected the original keeper entity");
    };
    assert_eq!(entity, original);
}

#[test]
fn diamond_split_then_merge_visits_the_merged_successor_once() {
    let mut world = LineageWorld::new();
    let d = world.create(Subject::Entity, "d");
    world.commit(); // 2
    let pieces = world.split(world.name_of(&d), &["one", "two"]);
    world.commit(); // 3
    let names: Vec<_> = pieces.iter().map(|piece| world.name_of(piece)).collect();
    let merged = world.merge(&names, Keep::MintNew);
    world.commit(); // 4
    let index = rebuild(world.log());
    let held = world.key_of(&d);
    assert_eq!(
        exact_key(&resolve(&index, held, 4, &exact())),
        world.key_of(&merged)
    );
    assert_eq!(metrics(&index, held, 4), probes(4, 3, 4));
    assert_eq!(
        resolve(&index, held, 3, &exact()),
        ambiguous(pieces.iter().map(|piece| world.key_of(piece)).collect())
    );
}

#[test]
fn unpostured_replace_carries_no_name_so_the_held_name_retires_there() {
    let mut world = LineageWorld::new();
    let p = world.create(Subject::Entity, "p");
    world.commit(); // 2
    world.replace(world.name_of(&p));
    world.commit(); // 3
    let index = rebuild(world.log());
    let held = world.key_of(&p);
    assert_eq!(resolve(&index, held, 3, &all()), retired(3));
    assert_eq!(exact_key(&resolve(&index, held, 2, &exact())), held);
}

#[test]
fn unknown_names_and_pruned_bases_refuse_typed_in_constant_work() {
    let mut world = LineageWorld::new();
    let face = world.create(Subject::Entity, "f");
    world.commit(); // 2
    let face_name = world.name_of(&face);
    let held = world.key_of(&face);
    let index = rebuild(world.log());
    let never_born = PersistentNameKey::Entity(member(200));
    assert_eq!(
        resolve(&index, never_born, 2, &exact()),
        unknown(never_born)
    );
    assert_eq!(metrics(&index, never_born, 2), probes(1, 0, 0));
    // An entity name never answers when asked as a relation name.
    let as_relation = PersistentNameKey::Relation(face_name);
    assert_eq!(
        resolve(&index, as_relation, 2, &exact()),
        unknown(as_relation)
    );

    let pruned =
        PersistentNameBranchIndex::rebuild_from_log(main_branch(), Some(CommitId(2)), world.log())
            .unwrap();
    assert_eq!(
        resolve(&pruned, held, 1, &exact()),
        Err(PersistentNameResolutionDenial::BasisPruned {
            basis: CommitId(1),
            retention_floor: CommitId(2),
        })
    );
    assert_eq!(metrics(&pruned, held, 1), probes(0, 0, 0));
    assert_eq!(exact_key(&resolve(&pruned, held, 2, &exact())), held);
    let mut advanced = pruned.clone();
    advanced.advance_retention_floor(CommitId(1));
    assert_eq!(
        advanced.retention_floor(),
        Some(CommitId(2)),
        "floors never move back"
    );
    advanced.advance_retention_floor(CommitId(3));
    assert!(matches!(
        resolve(&advanced, held, 2, &exact()),
        Err(PersistentNameResolutionDenial::BasisPruned { .. })
    ));
    assert_eq!(pruned.branch_id(), &BranchId("main".to_owned()));
}

#[test]
fn empty_filters_refuse_and_sideless_births_never_match_a_side() {
    let mut world = LineageWorld::new();
    let plain = world.create(Subject::Entity, "");
    let sided = world.create(Subject::Entity, "s");
    world.commit(); // 2
    let index = rebuild(world.log());
    let (plain_key, sided_key) = (world.key_of(&plain), world.key_of(&sided));
    // A birth without a discriminator has no side, exactly like a group.
    assert_eq!(index.names[&plain_key].side(), None);
    assert_eq!(
        index.names[&sided_key].side(),
        Some(side_of("s").as_slice())
    );
    for held in [plain_key, sided_key] {
        for empty in [side(""), bounded(&[])] {
            let outcome = index.resolve(held, CommitId(2), &empty);
            assert_eq!(
                outcome.result,
                Err(PersistentNameResolutionDenial::EmptySelectionFilter),
                "an empty filter never matches as a wildcard"
            );
            assert_eq!(outcome.metrics, probes(0, 0, 0));
        }
    }
    assert_eq!(
        resolve(&index, plain_key, 2, &side("s")),
        no_match(vec![plain_key])
    );
    assert_eq!(
        exact_key(&resolve(&index, sided_key, 2, &side("s"))),
        sided_key
    );
    assert_eq!(
        resolve(&index, sided_key, 2, &bounded(&[member(1)])),
        no_match(vec![sided_key])
    );
}
