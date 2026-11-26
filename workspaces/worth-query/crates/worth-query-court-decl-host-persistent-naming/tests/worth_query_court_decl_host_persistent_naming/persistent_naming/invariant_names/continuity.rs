//! An installed invariant reads names the commit continues rather than
//! mints: a birth whose identity is a retired name restores it, a merge
//! keeps a parent's name onto a new record, and an in-commit regeneration
//! keeps the retired holder's name. A retired name's birth reads alike in
//! both views and in an independent tracked reader.
use super::courts::{birth_of, evaluated, gone, Evaluated};
use super::*;
use crate::persistent_naming::outputs::sculpt::{Keeper, Step};
use crate::persistent_naming::outputs::{birth, keys, read};

#[test]
fn a_birth_of_a_retired_name_restores_it_and_its_birth_reads_alike() {
    let world = WatchWorld::install();
    world.committed("watch-restore-seed", authored(&["r1"]));
    let [r1] = world.names("watch-restore-seed", &["r1"], 0)[..] else {
        unreachable!()
    };
    world.committed(
        "watch-restore-retire",
        Plan {
            reads: keys(&["r1"]),
            deletes: keys(&["r1"]),
            steps: vec![Step::Retire(read("r1"))],
            ..Plan::default()
        },
    );
    world.take();
    // A later commit re-mints exactly the retired birth: Relational
    // classifies it as a Restore of the retired name, not a fresh birth.
    world.committed(
        "watch-restore",
        Plan {
            creates: keys(&["r2"]),
            steps: vec![Step::Birth("r2".into(), birth("r1", "authored"))],
            ..Plan::default()
        },
    );
    let watched = evaluated(&world.take(), "r2");
    assert_eq!(
        watched.proposed,
        Ok(r1),
        "the restored record carries the name"
    );
    assert_eq!(world.names("watch-restore-oracle", &["r2"], 1), [r1]);
    // The retired name's birth is recorded: both views and an independent
    // tracked reader read the same birth.
    let restored = birth_of(&watched);
    assert_eq!(
        watched.committed_birth.as_ref(),
        Some(&Ok(restored.clone()))
    );
    let oracle = world.births("watch-restore-births", &[("r1", r1)], 2);
    assert_eq!(&oracle[0], restored);
}

#[test]
fn a_merge_keeping_a_parents_name_onto_a_new_record_reads_that_name() {
    let world = WatchWorld::install();
    world.committed("watch-onto-seed", authored(&["o1", "o2"]));
    let [o1, o2] = world.names("watch-onto-seed", &["o1", "o2"], 0)[..] else {
        unreachable!()
    };
    world.take();
    world.committed(
        "watch-onto",
        Plan {
            reads: keys(&["o1", "o2"]),
            deletes: keys(&["o1", "o2"]),
            creates: keys(&["onto"]),
            steps: vec![Step::Merge(
                vec![read("o1"), read("o2")],
                Keeper::Onto(read("o1"), "onto".into()),
            )],
            ..Plan::default()
        },
    );
    let seen = world.take();
    let onto = evaluated(&seen, "onto");
    unseen_in_committed_record(&onto);
    assert_eq!(
        onto.proposed,
        Ok(o1),
        "the kept parent's name moves onto it"
    );
    assert_eq!(birth_of(&onto).name(), o1);
    for (key, name) in [("o1", o1), ("o2", o2)] {
        let (before, reobserved, _) = gone(&seen, key);
        assert_eq!((before, reobserved), (Ok(name), false));
    }
    assert_eq!(world.names("watch-onto-oracle", &["onto"], 1), [o1]);
}

#[test]
fn an_in_commit_regeneration_keeps_the_retired_holders_name() {
    let world = WatchWorld::install();
    world.committed("watch-regen-seed", authored(&["g1"]));
    let [g1] = world.names("watch-regen-seed", &["g1"], 0)[..] else {
        unreachable!()
    };
    world.take();
    world.committed(
        "watch-regen",
        Plan {
            reads: keys(&["g1"]),
            deletes: keys(&["g1"]),
            creates: keys(&["g1r"]),
            steps: vec![
                Step::Retire(read("g1")),
                Step::Birth("g1r".into(), birth("g1", "authored")),
            ],
            ..Plan::default()
        },
    );
    let seen = world.take();
    let regenerated = evaluated(&seen, "g1r");
    unseen_in_committed_record(&regenerated);
    assert_eq!(regenerated.proposed, Ok(g1));
    let (before, reobserved, _) = gone(&seen, "g1");
    assert_eq!((before, reobserved), (Ok(g1), false));
}

/// A new record carrying a name the committed view already records: only
/// the record itself is new there.
fn unseen_in_committed_record(seen: &Evaluated) {
    assert_eq!(seen.committed, None, "the record is new");
}
