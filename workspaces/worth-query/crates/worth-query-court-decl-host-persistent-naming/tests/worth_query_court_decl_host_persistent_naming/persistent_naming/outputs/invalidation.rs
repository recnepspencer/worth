//! A decision that resolved a split piece's name is invalidated when a later
//! installed operation retires or splits that piece; an unrelated operation
//! leaves it current.
use super::*;

/// Seeds `base`, splits it into `n1`/`n2` and returns `n1`'s name.
fn split_world(token: &str) -> (World, WorthQueryName<Member>) {
    let world = World::install();
    world.committed(
        &format!("{token}-seed"),
        Plan {
            creates: keys(&["base", "other"]),
            steps: vec![
                Step::Birth("base".into(), birth("base", "authored")),
                Step::Birth("other".into(), birth("other", "authored")),
            ],
            ..Plan::default()
        },
    );
    world.committed(
        &format!("{token}-split"),
        Plan {
            reads: keys(&["base"]),
            deletes: keys(&["base"]),
            creates: keys(&["n1", "n2"]),
            steps: vec![Step::Split(
                read("base"),
                vec![
                    ("n1".into(), piece("cut", 1, None)),
                    ("n2".into(), piece("cut", 2, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    let n1 = named(&world.probe(token, &["n1"], 0), "n1");
    (world, n1)
}

/// Prepares a probe resolving `n1`, runs `plan`, then commits the probe.
fn race(token: &str, plan: Plan) -> Outcome {
    let (world, n1) = split_world(token);
    probe::hold(token, "n1", n1);
    let key = format!("{token}:raced-probe");
    let mut request = world
        .request()
        .advanced()
        .mutate(probe::Intent {
            token: token.to_owned(),
            keys: keys(&["n1"]),
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
        resolution(&probe::take(token), "n1"),
        &resolved_to("n1", "n1")
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
fn retiring_a_resolved_split_piece_invalidates_the_decision() {
    let outcome = race(
        "retire-race",
        Plan {
            reads: keys(&["n1"]),
            deletes: keys(&["n1"]),
            steps: vec![Step::Retire(read("n1"))],
            ..Plan::default()
        },
    );
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn splitting_a_resolved_split_piece_invalidates_the_decision() {
    let outcome = race(
        "resplit-race",
        Plan {
            reads: keys(&["n1"]),
            deletes: keys(&["n1"]),
            creates: keys(&["n1a", "n1b"]),
            steps: vec![Step::Split(
                read("n1"),
                vec![
                    ("n1a".into(), piece("recut", 1, None)),
                    ("n1b".into(), piece("recut", 2, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn an_unrelated_retirement_leaves_the_decision_current() {
    let outcome = race(
        "unrelated-race",
        Plan {
            reads: keys(&["other"]),
            deletes: keys(&["other"]),
            steps: vec![Step::Retire(read("other"))],
            ..Plan::default()
        },
    );
    assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
}
