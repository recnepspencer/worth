//! Relation and group resolution courts over real Phase 2 finalization and
//! Phase 3 group naming transitions.

use crate::facade::history::CommitId;
use crate::facade::lineage::{
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameKey,
    PersistentNameResolution, PersistentNameTarget,
};

use super::name_group_world::{member, GroupWorld};
use super::name_index_fixtures::{
    all, ambiguous, bounded, exact, exact_key, main_branch, metrics, no_match, plural_keys, probes,
    rebuild, resolve, retired, side, unknown,
};
use super::name_world::{Keep, LineageWorld, Subject};

#[test]
fn relation_split_retire_merge_lifecycle_matches_the_entity_contract() {
    let mut world = LineageWorld::new();
    let seam = world.create(Subject::Relation, "s");
    let other = world.create(Subject::Relation, "o");
    world.commit(); // 2
    let pieces = world.split(world.name_of(&seam), &["top", "bottom"]);
    world.commit(); // 3
    let (top, bottom) = (world.key_of(&pieces[0]), world.key_of(&pieces[1]));
    let held = world.key_of(&seam);
    assert!(matches!(held, PersistentNameKey::Relation(_)));
    let index = rebuild(world.log());
    assert_eq!(
        resolve(&index, held, 3, &exact()),
        ambiguous(vec![top, bottom])
    );
    assert_eq!(plural_keys(&resolve(&index, held, 3, &all())).len(), 2);
    assert_eq!(
        exact_key(&resolve(&index, held, 3, &side("bottom"))),
        bottom
    );
    let Ok(PersistentNameResolution::Exact(PersistentNameTarget::Relation { relation, .. })) =
        resolve(&index, held, 2, &exact())
    else {
        panic!("a live seam resolves to its relation record");
    };
    assert_eq!(
        crate::facade::lineage::PersistentNameHolder::Relation(relation),
        seam
    );

    world.retire(world.name_of(&pieces[0]));
    world.commit(); // 4
    let kept = world.merge(
        &[world.name_of(&pieces[1]), world.name_of(&other)],
        Keep::InPlace,
    );
    world.commit(); // 5
    let index = rebuild(world.log());
    assert_eq!(exact_key(&resolve(&index, held, 4, &exact())), bottom);
    assert_eq!(
        exact_key(&resolve(&index, held, 5, &exact())),
        world.key_of(&kept)
    );
    assert_eq!(
        exact_key(&resolve(&index, world.key_of(&other), 5, &exact())),
        bottom
    );
    world.retire(world.name_of(&kept));
    world.commit(); // 6
    let index = rebuild(world.log());
    assert_eq!(resolve(&index, held, 6, &exact()), retired(6));
    assert_eq!(resolve(&index, top, 6, &all()), retired(4));
}

fn group_index(log: &[PersistentNameCommittedBatch]) -> PersistentNameBranchIndex {
    PersistentNameBranchIndex::rebuild_from_log(main_branch(), None, log).unwrap()
}

#[test]
fn group_split_retire_revive_merge_lifecycle_resolves_by_ending_transitions() {
    let mut world = GroupWorld::new();
    let mut log = Vec::new();
    let mut commit = |world: &mut GroupWorld, groups: &[&[u8]]| {
        let transition = world.step(groups).expect("valid group transition");
        let next = CommitId(log.len() as u64 + 1);
        log.push(PersistentNameCommittedBatch::new(next).with_published_group_transitions(transition));
        log.clone()
    };
    commit(&mut world, &[&[1, 2, 3, 4]]); // 1: G born
    let g = PersistentNameKey::Group(world.group_of(1));
    commit(&mut world, &[&[1, 2], &[3, 4]]); // 2: G kept, H split off
    let h = PersistentNameKey::Group(world.group_of(3));
    let index = group_index(&commit(&mut world, &[&[3, 4]])); // 3: G retires alone
                                                              // A kept group answers for itself; the fragment that split off while it
                                                              // stayed live never answers for it, even after it retires.
    assert_eq!(exact_key(&resolve(&index, g, 2, &exact())), g);
    assert_eq!(metrics(&index, g, 2), probes(1, 0, 0));
    assert_eq!(resolve(&index, g, 3, &exact()), retired(3));
    assert_eq!(exact_key(&resolve(&index, h, 3, &exact())), h);

    let index = group_index(&commit(&mut world, &[&[1], &[3, 4]])); // 4: G revived
    assert_eq!(
        world.group_of(1),
        match g {
            PersistentNameKey::Group(group) => group,
            _ => unreachable!(),
        }
    );
    assert_eq!(exact_key(&resolve(&index, g, 4, &exact())), g);
    assert_eq!(resolve(&index, g, 3, &all()), retired(3));
    assert_eq!(exact_key(&resolve(&index, g, 2, &exact())), g);

    let index = group_index(&commit(&mut world, &[&[1, 3, 4]])); // 5: H merged into G
    assert_eq!(exact_key(&resolve(&index, h, 5, &exact())), g);
    assert_eq!(exact_key(&resolve(&index, h, 4, &exact())), h);
    assert_eq!(metrics(&index, h, 5), probes(2, 1, 1));

    let index = group_index(&commit(&mut world, &[&[3], &[4]])); // 6: G splits away
    let (three, four) = (
        PersistentNameKey::Group(world.group_of(3)),
        PersistentNameKey::Group(world.group_of(4)),
    );
    assert_eq!(
        three, h,
        "the successor around H's anchor and sources revives H"
    );
    assert_eq!(
        resolve(&index, g, 6, &exact()),
        ambiguous(vec![three, four])
    );
    assert_eq!(plural_keys(&resolve(&index, g, 6, &all())).len(), 2);
    assert_eq!(
        exact_key(&resolve(&index, g, 6, &bounded(&[member(4)]))),
        four
    );
    assert_eq!(
        resolve(&index, g, 6, &bounded(&[member(1)])),
        no_match(vec![three, four])
    );
    // Groups carry no birth side, so a side filter never claims one.
    assert_eq!(
        resolve(&index, g, 6, &side("left")),
        no_match(vec![three, four])
    );
    // Carriers follow the group's membership at each basis.
    assert_eq!(exact_key(&resolve(&index, g, 5, &bounded(&[member(3)]))), g);
    assert_eq!(
        resolve(&index, g, 4, &bounded(&[member(3)])),
        no_match(vec![g])
    );
    assert_eq!(exact_key(&resolve(&index, h, 5, &exact())), g);
    assert_eq!(exact_key(&resolve(&index, h, 6, &exact())), h);

    let index = group_index(&commit(&mut world, &[&[4]])); // 7
    assert_eq!(resolve(&index, h, 7, &exact()), retired(7));
    assert_eq!(exact_key(&resolve(&index, g, 7, &exact())), four);
    let never = PersistentNameKey::Group(
        worth_foundational::facade::AggregateIdentity::from_bytes([3; 32]),
    );
    assert_eq!(resolve(&index, never, 7, &exact()), unknown(never));
    assert_eq!(resolve(&index, four, 5, &exact()), unknown(four));
    assert_eq!(
        world.live_groups(),
        vec![match four {
            PersistentNameKey::Group(group) => group,
            _ => unreachable!(),
        }]
    );
}
