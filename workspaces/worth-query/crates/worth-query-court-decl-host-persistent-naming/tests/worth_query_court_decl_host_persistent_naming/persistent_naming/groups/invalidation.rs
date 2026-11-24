//! A decision that read a group is keyed on that read: a later commit that
//! splits, merges, dissolves or re-members the group it read stales it, and
//! an unrelated commit leaves it current.
use super::super::outputs::{sculpt, Outcome};
use super::*;
use GroupRole::{Host, Member as Plain};

/// Seeds two loops: `{h1, x}` and `{h2, y}`.
fn two_loops(token: &str) -> (World, Vec<WorthQueryName<Member>>) {
    let (world, n) = seeded(token, &["h1", "x", "h2", "y"]);
    let decl = loops(
        vec![
            group(&[(Host, n[0]), (Plain, n[1])]),
            group(&[(Host, n[2]), (Plain, n[3])]),
        ],
        &[],
    );
    declare(&world, &format!("{token}-loops"), decl);
    (world, n)
}

/// Prepares a decision reading `reads`, lets `race` commit, then commits it.
fn race(world: &World, token: &str, reads: Vec<GroupRead>, race: GroupDecl) -> Outcome {
    sculpt::author(
        token,
        Plan {
            creates: keys(&[&format!("{token}-marker")]),
            steps: vec![Step::Birth(
                format!("{token}-marker"),
                birth(token, "authored"),
            )],
            group_reads: reads,
            ..Plan::default()
        },
    );
    let key = format!("{token}:raced");
    let mut request = world
        .request()
        .advanced()
        .mutate(sculpt::Intent {
            token: token.to_owned(),
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
    declare(world, &format!("{token}-race"), race);
    prepared.commit(worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
}

fn stale(outcome: &Outcome) -> bool {
    matches!(
        outcome,
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
    )
}

#[test]
fn regrouping_the_read_member_stales_the_decision() {
    let (world, n) = two_loops("group-stale-member");
    let read = vec![GroupRead::LoopOf(Held::Name(n[1]))];
    // `x` moves into the other loop: what the decision read changed.
    let outcome = race(
        &world,
        "group-stale-member",
        read,
        loops(
            vec![group(&[(Host, n[2]), (Plain, n[3]), (Plain, n[1])])],
            &[],
        ),
    );
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn dissolving_the_resolved_loop_stales_the_decision() {
    let (world, n) = two_loops("group-stale-resolve");
    let held = loop_of(&world, "group-stale-resolve-r", n[0]).unwrap();
    let read = vec![GroupRead::Loop(held, NameSelection::Exact)];
    let outcome = race(
        &world,
        "group-stale-resolve",
        read,
        loops(Vec::new(), &[held]),
    );
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn an_unrelated_regrouping_leaves_the_decision_current() {
    let (world, n) = two_loops("group-current");
    let read = vec![GroupRead::LoopOf(Held::Name(n[1]))];
    // Only the other loop changes; `x`'s loop is untouched.
    let decl = loops(vec![group(&[(Host, n[3])])], &[]);
    let outcome = race(&world, "group-current", read, decl);
    assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
}
