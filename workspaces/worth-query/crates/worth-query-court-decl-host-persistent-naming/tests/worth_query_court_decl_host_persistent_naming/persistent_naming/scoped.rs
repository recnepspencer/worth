//! Owner-scoped resolution (public #133): one held member name is live in two
//! models at once. Unscoped resolution refuses with each candidate and the
//! model that owns it; `resolve_in` answers within one model and never falls
//! back to another; and re-owning a candidate invalidates a scoped decision.
//! The owner is the member's declared `ModelMember` chain, never name text.
use super::outputs::sculpt::{Plan, Step};
use super::outputs::{
    birth, exact, keys, named, piece, read, resolution, resolved_to, Outcome, World,
};
use super::*;
use probe::Observation;
use worth_query_decl::facade::application_naming::WorthQueryName;

/// Seeds `base` under model `a` and splits it into `kept`, which stays under
/// `a`, and `tool`, which belongs to model `b`. Returns `base`'s name.
fn two_owners(token: &str) -> (World, WorthQueryName<Member>) {
    let world = World::install();
    world.committed(
        &format!("{token}-seed"),
        Plan {
            creates: keys(&["base"]),
            steps: vec![Step::Birth("base".into(), birth("base", "authored"))],
            ..Plan::default()
        },
    );
    let base = named(&world.probe(token, &["base"], 0), "base");
    world.committed(
        &format!("{token}-split"),
        Plan {
            reads: keys(&["base"]),
            deletes: keys(&["base"]),
            creates: keys(&["kept", "tool"]),
            owners: vec![("tool".into(), "b".into())],
            steps: vec![Step::Split(
                read("base"),
                vec![
                    ("kept".into(), piece("cut", 1, None)),
                    ("tool".into(), piece("cut", 2, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    (world, base)
}

fn owned(
    label: &str,
    candidates: &[(WorthQueryName<Member>, &[&str])],
) -> Vec<(WorthQueryName<Member>, Vec<String>)> {
    let mut candidates: Vec<_> = candidates
        .iter()
        .map(|(name, owners)| (*name, keys(owners)))
        .collect();
    candidates.sort();
    assert!(!candidates.is_empty(), "{label}");
    candidates
}

#[test]
fn a_name_live_in_two_models_resolves_within_each_and_is_ambiguous_across_them() {
    let token = "scoped-two-owners";
    let (world, base) = two_owners(token);
    probe::hold_owned(token, "anywhere", base, exact());
    probe::hold_in(token, "in-a", base, exact(), "a");
    probe::hold_in(token, "in-b", base, exact(), "b");
    let observed = world.probe(token, &["kept", "tool"], 1);
    let (kept, tool) = (named(&observed, "kept"), named(&observed, "tool"));

    // Across models the name is ambiguous, and each candidate says whose it is.
    assert_eq!(
        resolution(&observed, "anywhere"),
        &Observation::AmbiguousOwned {
            label: "anywhere".into(),
            candidates: owned("anywhere", &[(kept, &["a"]), (tool, &["b"])]),
        }
    );
    // Within one model it is exact.
    assert_eq!(resolution(&observed, "in-a"), &resolved_to("in-a", "kept"));
    assert_eq!(resolution(&observed, "in-b"), &resolved_to("in-b", "tool"));
}

#[test]
fn an_owner_without_the_name_refuses_and_never_falls_back_to_another_owner() {
    let token = "scoped-no-fallback";
    let (world, base) = two_owners(token);
    // The tool piece retires: model `b` no longer holds any piece of `base`.
    world.committed(
        &format!("{token}-retire"),
        Plan {
            reads: keys(&["tool"]),
            deletes: keys(&["tool"]),
            steps: vec![Step::Retire(read("tool"))],
            ..Plan::default()
        },
    );
    probe::hold(token, "anywhere", base);
    probe::hold_in(token, "in-a", base, exact(), "a");
    probe::hold_in(token, "in-b", base, exact(), "b");
    let observed = world.probe(token, &["kept"], 1);
    let kept = named(&observed, "kept");

    // Unscoped, the one live piece answers; scoped to `b` it must not.
    assert_eq!(
        resolution(&observed, "anywhere"),
        &resolved_to("anywhere", "kept")
    );
    assert_eq!(resolution(&observed, "in-a"), &resolved_to("in-a", "kept"));
    assert_eq!(
        resolution(&observed, "in-b"),
        &Observation::NotInOwner {
            label: "in-b".into(),
            candidates: owned("in-b", &[(kept, &["a"])]),
        }
    );
}

#[test]
fn every_piece_one_model_owns_is_plural_or_ambiguous_within_it() {
    let token = "scoped-plural";
    let (world, base) = two_owners(token);
    // `kept` is cut again inside model `a`; `tool` stays whole in `b`.
    world.committed(
        &format!("{token}-recut"),
        Plan {
            reads: keys(&["kept"]),
            deletes: keys(&["kept"]),
            creates: keys(&["k1", "k2"]),
            steps: vec![Step::Split(
                read("kept"),
                vec![
                    ("k1".into(), piece("recut", 1, None)),
                    ("k2".into(), piece("recut", 2, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    use worth_query_decl::facade::application_naming::NameSelection;
    probe::hold_in(token, "exact-a", base, exact(), "a");
    probe::hold_in(token, "all-a", base, NameSelection::AllDescendants, "a");
    probe::hold_in(token, "all-b", base, NameSelection::AllDescendants, "b");
    let observed = world.probe(token, &["k1", "k2", "tool"], 1);
    let (k1, k2) = (named(&observed, "k1"), named(&observed, "k2"));

    // Two pieces in `a`: `Exact` refuses with only `a`'s candidates, and
    // `AllDescendants` returns both; `b`'s single piece is exact either way.
    assert_eq!(
        resolution(&observed, "exact-a"),
        &Observation::AmbiguousOwned {
            label: "exact-a".into(),
            candidates: owned("exact-a", &[(k1, &["a"]), (k2, &["a"])]),
        }
    );
    let Observation::Plural { keys: plural, .. } = resolution(&observed, "all-a") else {
        panic!("{observed:?}");
    };
    let mut plural = plural.clone();
    plural.sort();
    assert_eq!(plural, keys(&["k1", "k2"]));
    assert_eq!(
        resolution(&observed, "all-b"),
        &resolved_to("all-b", "tool")
    );
}

/// Prepares a probe resolving the live `tool` piece within model `b`,
/// commits `plan`, then commits the probe. The held name is live, so the
/// decision rests only on facts it can track: the piece and its owner hop.
fn race(token: &str, plan: Plan) -> Outcome {
    let (world, _) = two_owners(token);
    let tool = named(&world.probe(token, &["tool"], 1), "tool");
    probe::hold_in(token, "in-b", tool, exact(), "b");
    let key = format!("{token}:raced-probe");
    let mut request = world
        .request()
        .advanced()
        .mutate(probe::Intent {
            token: token.to_owned(),
            keys: keys(&["kept", "tool"]),
            value: 900,
        })
        .without_source()
        .idempotency(&key);
    let prepared = match request
        .prepare_in_program(
            &world.host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    {
        WorthQueryApplicationProgramMutationPreparation::Prepared(candidate) => candidate,
        WorthQueryApplicationProgramMutationPreparation::Settled(_) => panic!("fresh decision"),
    };
    assert_eq!(
        resolution(&probe::take(token), "in-b"),
        &resolved_to("in-b", "tool")
    );
    world.committed(&format!("{token}-race"), plan);
    prepared.commit(worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
}

fn stale(outcome: &Outcome) -> bool {
    matches!(
        outcome,
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
    )
}

#[test]
fn re_owning_a_candidate_invalidates_the_scoped_decision() {
    // `tool` moves from model `b` to model `a`: the owner hop the decision
    // read no longer reaches `b`.
    let outcome = race(
        "scoped-reown",
        Plan {
            reads: keys(&["tool"]),
            moves: vec![("tool".into(), "b".into(), "a".into())],
            ..Plan::default()
        },
    );
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn splitting_a_candidate_invalidates_the_scoped_decision() {
    let outcome = race(
        "scoped-resplit",
        Plan {
            reads: keys(&["tool"]),
            deletes: keys(&["tool"]),
            creates: keys(&["t1", "t2"]),
            owners: vec![("t1".into(), "b".into()), ("t2".into(), "b".into())],
            steps: vec![Step::Split(
                read("tool"),
                vec![
                    ("t1".into(), piece("recut", 1, None)),
                    ("t2".into(), piece("recut", 2, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn re_owning_another_member_leaves_the_scoped_decision_current() {
    // `kept` moves to model `b`: no hop of the resolved piece changed, and a
    // new member under model `a` changes no candidate at all.
    let outcome = race(
        "scoped-unrelated",
        Plan {
            reads: keys(&["kept"]),
            moves: vec![("kept".into(), "a".into(), "b".into())],
            creates: keys(&["bystander"]),
            steps: vec![Step::Birth(
                "bystander".into(),
                birth("bystander", "authored"),
            )],
            ..Plan::default()
        },
    );
    assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
}
