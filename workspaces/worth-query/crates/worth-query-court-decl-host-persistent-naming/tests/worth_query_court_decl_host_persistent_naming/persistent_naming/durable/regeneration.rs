//! In-commit regeneration of a same-occurrence pair across Journal reopen.
//!
//! One commit retires `f1` and `f2` (cut by `f1`, referencing it as a
//! same-occurrence name) and re-creates both in the same occurrence: both
//! names regenerate onto fresh records. Journal replay must rebuild `f2`'s
//! reference to the regenerated `f1` as a same-occurrence reference, not a
//! basis one, so the Home reopens serving both names exactly as before.
use super::super::outputs::sculpt::{Plan, ScopeStep, Step};
use super::super::outputs::{keys, read, CutSide, World};
use super::super::world::home;
use super::*;
use worth_query_decl::facade::application_naming::OccurrencePath;

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
