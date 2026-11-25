//! A decision that resolved a name an envelope's stage minted is stale once
//! a later envelope splits, merges or retires that output, and stays current
//! across an unrelated envelope.
use super::sculpt::{Held, Keeper};
use super::*;

/// An envelope whose two stages birth `n1` and `other`; returns their names.
fn born(token: &str) -> (World, WorthQueryName<Member>, WorthQueryName<Member>) {
    let world = World::install();
    let named_plan = |key: &str| Plan {
        creates: keys(&[key]),
        steps: vec![Step::Birth(
            key.into(),
            birth(&format!("{token}-{key}"), "authored"),
        )],
        ..Plan::default()
    };
    world.committed_envelope(
        &format!("{token}-born"),
        vec![
            stage(&format!("{token}-born-1"), named_plan("n1")),
            stage(&format!("{token}-born-2"), named_plan("other")),
        ],
    );
    let names = world.names(token, &["n1", "other"], 0);
    (world, names[0], names[1])
}

/// Prepares a probe resolving `n1`, runs the envelope `race` builds from the
/// two held names, then commits the probe.
fn race(
    token: &str,
    race: impl FnOnce(WorthQueryName<Member>, WorthQueryName<Member>) -> Vec<Stage>,
) -> outputs::Outcome {
    let (world, n1, other) = born(token);
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
        outputs::resolution(&probe::take(token), "n1"),
        &outputs::resolved_to("n1", "n1")
    );
    world.committed_envelope(&format!("{token}-race"), race(n1, other));
    prepared.commit(worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
}

fn stale(outcome: &outputs::Outcome) -> bool {
    matches!(
        outcome,
        outputs::Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
    )
}

fn retire(key: &str, name: WorthQueryName<Member>) -> Plan {
    Plan {
        plain_reads: keys(&[key]),
        deletes: keys(&[key]),
        steps: vec![Step::Retire(Held::Name(name))],
        ..Plan::default()
    }
}

fn add(key: &str, occurrence: &str) -> Plan {
    Plan {
        creates: keys(&[key]),
        steps: vec![Step::Birth(key.into(), birth(occurrence, "authored"))],
        ..Plan::default()
    }
}

#[test]
fn an_envelope_retiring_a_resolved_output_invalidates_the_decision() {
    let token = "coupled-retire-race";
    let outcome = race(token, |n1, _| {
        vec![
            stage(&format!("{token}-1"), add("fresh", "coupled-retire-fresh")),
            stage(&format!("{token}-2"), retire("n1", n1)),
        ]
    });
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn an_envelope_splitting_a_resolved_output_invalidates_the_decision() {
    let token = "coupled-split-race";
    let outcome = race(token, |n1, _| {
        vec![stage(
            &format!("{token}-1"),
            Plan {
                plain_reads: keys(&["n1"]),
                deletes: keys(&["n1"]),
                creates: keys(&["n1a", "n1b"]),
                owners: vec![("n1b".into(), "b".into())],
                steps: vec![Step::Split(
                    Held::Name(n1),
                    vec![
                        ("n1a".into(), piece("coupled-recut", 1, None)),
                        ("n1b".into(), piece("coupled-recut", 2, None)),
                    ],
                )],
                ..Plan::default()
            },
        )]
    });
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn an_envelope_merging_a_resolved_output_away_invalidates_the_decision() {
    let token = "coupled-merge-race";
    let outcome = race(token, |n1, other| {
        vec![stage(
            &format!("{token}-1"),
            Plan {
                plain_reads: keys(&["n1", "other"]),
                deletes: keys(&["n1"]),
                steps: vec![Step::Merge(
                    vec![Held::Name(n1), Held::Name(other)],
                    Keeper::InPlace(Held::Name(other)),
                )],
                ..Plan::default()
            },
        )]
    });
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn an_unrelated_envelope_leaves_the_decision_current() {
    let token = "coupled-unrelated-race";
    let outcome = race(token, |_, other| {
        vec![
            stage(
                &format!("{token}-1"),
                add("fresh", "coupled-unrelated-fresh"),
            ),
            stage(&format!("{token}-2"), retire("other", other)),
        ]
    });
    assert!(
        matches!(outcome, outputs::Outcome::Committed { .. }),
        "{outcome:?}"
    );
}
