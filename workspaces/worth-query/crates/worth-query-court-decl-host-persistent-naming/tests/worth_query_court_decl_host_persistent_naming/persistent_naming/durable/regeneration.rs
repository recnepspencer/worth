//! In-commit regeneration of same-occurrence records across Journal reopen.
//!
//! Covers both a regenerated same-occurrence pair (earlier role referenced by a
//! later role) and a same-role sibling referencing a regenerated name as a
//! `Basis` reference (P1-A): Journal replay must reopen serving every name
//! identically.
use super::super::outputs::sculpt::{Plan, ScopeStep, Step};
use super::super::outputs::{birth, keys, read, CutSide, SidedCut, World};
use super::super::world::home;
use super::*;
use worth_query_decl::facade::application_naming::{NameRef, OccurrencePath};

/// Names `first` authored and `second` cut by it, in one occurrence.
fn pair(first: &str, second: &str) -> Step {
    Step::Scoped(
        OccurrencePath::root("durable-pair").expect("a valid occurrence"),
        vec![
            ScopeStep::Name(first.into(), MemberRole::Authored),
            ScopeStep::CutBy {
                key: second.into(),
                side: CutSide::Positive,
                of: first.into(),
            },
        ],
    )
}

#[test]
fn a_regenerated_same_occurrence_pair_survives_a_journal_reopen() {
    let mut world = World::open_directory(home::location());
    world.committed(
        "rp-create",
        Plan {
            creates: keys(&["rp-f1", "rp-f2"]),
            steps: vec![pair("rp-f1", "rp-f2")],
            ..Plan::default()
        },
    );
    let names = world.names("rp-names", &["rp-f1", "rp-f2"], 0);
    world.committed(
        "rp-regenerate",
        Plan {
            reads: keys(&["rp-f1", "rp-f2"]),
            deletes: keys(&["rp-f1", "rp-f2"]),
            creates: keys(&["rp-g1", "rp-g2"]),
            steps: vec![
                Step::Retire(read("rp-f1")),
                Step::Retire(read("rp-f2")),
                pair("rp-g1", "rp-g2"),
            ],
            ..Plan::default()
        },
    );
    assert_eq!(
        world.names("rp-regenerated", &["rp-g1", "rp-g2"], 1),
        names,
        "both names regenerate onto the fresh records"
    );
    let mut held = Holdings::default();
    held.keys = keys(&["rp-f1", "rp-f2", "rp-g1", "rp-g2"]);
    held.member("f1", names[0]);
    held.member("f2", names[1]);
    crash_and_compare(&mut world, &held, "regenerate-pair", false);
}

#[test]
fn a_same_role_sibling_referencing_a_regenerated_name_survives_journal_reopen() {
    let mut world = World::open_directory(home::location());
    world.committed(
        "rs-create",
        Plan {
            creates: keys(&["rs-f1"]),
            steps: vec![Step::Birth("rs-f1".into(), birth("shared-occ", "piece"))],
            ..Plan::default()
        },
    );
    let f1 = world.names("rs-f1-name", &["rs-f1"], 0)[0];
    let sibling_cut = SidedCut::By {
        cutter: NameRef::Committed(f1),
        side: CutSide::Positive,
        along: None,
    };
    world.committed(
        "rs-regen-and-sibling",
        Plan {
            reads: keys(&["rs-f1"]),
            deletes: keys(&["rs-f1"]),
            creates: keys(&["rs-f1-next", "rs-sibling"]),
            steps: vec![
                Step::Retire(read("rs-f1")),
                Step::Birth("rs-f1-next".into(), birth("shared-occ", "piece")),
                Step::Birth(
                    "rs-sibling".into(),
                    birth("shared-occ", "piece").with_discriminator(&sibling_cut),
                ),
            ],
            ..Plan::default()
        },
    );
    let after = world.names("rs-after", &["rs-f1-next", "rs-sibling"], 1);
    assert_eq!(after[0], f1, "f1 regenerates onto the fresh record");
    let mut held = Holdings::default();
    held.keys = keys(&["rs-f1", "rs-f1-next", "rs-sibling"]);
    held.member("f1", after[0]);
    held.member("sibling", after[1]);
    crash_and_compare(&mut world, &held, "regenerate-same-role-sibling", false);
}
