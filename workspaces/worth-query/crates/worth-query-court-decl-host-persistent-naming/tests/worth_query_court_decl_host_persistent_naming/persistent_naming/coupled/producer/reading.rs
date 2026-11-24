//! A coupled producer reads committed names on the envelope's unpublished
//! candidate and names its outputs by them.
//!
//! The producer decides when the edit executes, not when it is prepared, so
//! a commit landing between the edit's preparation and its commit is
//! exactly what the producer reads: a member that commit retired refuses,
//! and an owner it changed is the owner `resolve_in` sees. No answer from
//! the earlier basis survives into the commit.
use super::super::candidate::{created, retired, seamed};
use super::reads::{observed, read_names, Observed, ReadPlan};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

/// Model `a`'s Initial reads `direct` (owned by `a`, not `b`) and seam `s1`
/// from `direct` to `nested`.
fn plan() -> ReadPlan {
    ReadPlan {
        member: "direct".into(),
        owner: "a".into(),
        other: "b".into(),
        seam: ("direct".into(), "nested".into()),
    }
}

/// A world with seam `s1`, and the committed names of `direct` and `s1`.
fn world(token: &str) -> (World, WorthQueryName<Member>, WorthQueryName<Seam>) {
    let world = seamed(token);
    let direct = world.names(&format!("{token}-basis"), &["direct"], 0)[0];
    let s1 = world.seam_name(&format!("{token}-basis-seam"), "direct", "nested");
    (world, direct, s1)
}

/// The names model `a`'s producer outputs carry: primary, secondary, seam.
fn outputs(world: &World, token: &str) -> (Vec<WorthQueryName<Member>>, WorthQueryName<Seam>) {
    let (p, s) = (primary_key("a"), secondary_key("a"));
    let members = world.names(token, &[&p, &s], 10);
    (members, world.seam_name(&format!("{token}-seam"), &p, &s))
}

#[test]
fn a_coupled_producer_names_its_outputs_by_names_it_read_on_the_candidate() {
    let (token, value) = ("producer-reads", 3101);
    let (coupled, direct, s1) = world(token);
    read_names(value, plan());
    let outcome = coupled.shape("a", value, false, token);
    assert!(
        matches!(outcome, outputs::Outcome::Committed { .. }),
        "{outcome:?}"
    );
    assert_eq!(
        observed(value),
        Some(Ok(Observed {
            member: direct,
            resolves_to_member: true,
            resolves_in_owner: true,
            not_in_other: true,
            seam: s1,
            seam_resolves_to_seam: true,
        })),
        "every read answered from the commit basis, on the candidate"
    );
    // A direct operation in a twin world (same seeds, same seam birth)
    // naming the same outputs by the same held names.
    let twin_token = format!("{token}-direct");
    let (twin, twin_direct, twin_s1) = world(token);
    assert_eq!((twin_direct, twin_s1), (direct, s1));
    let (p, s, occurrence) = (primary_key("a"), secondary_key("a"), occurrence("a"));
    let cut = outputs::SidedCut::By {
        cutter: direct.into(),
        side: outputs::CutSide::Positive,
        along: Some(s1.into()),
    };
    twin.committed(
        &twin_token,
        Plan {
            creates: vec![p.clone(), s.clone()],
            seams: vec![(seam_key("a"), p.clone(), s.clone())],
            steps: vec![
                Step::Birth(p.clone(), birth(&occurrence, "authored")),
                Step::Birth(
                    s.clone(),
                    birth(&occurrence, "piece").with_discriminator(&cut),
                ),
                Step::SeamBirthOf {
                    key: seam_key("a"),
                    birth: birth(&occurrence, "seam"),
                    index: 0,
                    of: p.clone(),
                },
            ],
            ..Plan::default()
        },
    );
    assert_eq!(
        outputs(&coupled, &format!("{token}-coupled")),
        outputs(&twin, &format!("{token}-twin")),
        "the producer's outputs carry exactly the names a direct operation mints"
    );
}

/// Prepares model `a`'s edit to `value`, lands `race` first, then commits
/// the prepared edit; returns its outcome and what its producer read.
fn raced(
    token: &str,
    value: u64,
    race: impl FnOnce(WorthQueryName<Member>) -> Plan,
) -> (World, outputs::Outcome, Option<Result<Observed, String>>) {
    let (world, direct, _) = world(token);
    read_names(value, plan());
    let key = format!("{token}:raced");
    let mut request = world
        .request()
        .advanced()
        .mutate(shape::Intent {
            key: "a".into(),
            value,
            collide: false,
        })
        .without_source()
        .idempotency(&key);
    let prepared = match request
        .prepare_in_program(&world.host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        WorthQueryApplicationProgramMutationPreparation::Prepared(candidate) => candidate,
        WorthQueryApplicationProgramMutationPreparation::Settled(_) => panic!("fresh decision"),
    };
    world.committed(&format!("{token}-race"), race(direct));
    let outcome = prepared.commit(ExecutionAllocationPolicy::SystemAllocation);
    (world, outcome, observed(value))
}

#[test]
fn an_unrelated_commit_first_leaves_every_producer_read_as_it_was() {
    let (_, outcome, read) = raced("producer-race-unrelated", 3102, |_| {
        created("unrelated", "producer-race-unrelated")
    });
    assert!(
        matches!(outcome, outputs::Outcome::Committed { .. }),
        "{outcome:?}"
    );
    assert!(
        matches!(&read, Some(Ok(observed)) if observed.resolves_in_owner && observed.not_in_other),
        "{read:?}"
    );
}

#[test]
fn a_member_retired_first_is_never_read_by_the_producer() {
    let (world, outcome, read) = raced("producer-race-retire", 3103, |direct| {
        retired("direct", direct)
    });
    assert!(
        !matches!(outcome, outputs::Outcome::Committed { .. }),
        "the producer cannot read the retired member: {outcome:?}"
    );
    assert!(matches!(read, Some(Err(_))), "{read:?}");
    assert!(!world.live(&primary_key("a")), "zero effects");
}

#[test]
fn an_owner_changed_first_is_the_owner_the_producer_resolves_in() {
    let (_, outcome, read) = raced("producer-race-reown", 3104, |_| Plan {
        plain_reads: keys(&["direct"]),
        moves: vec![("direct".into(), "a".into(), "b".into())],
        ..Plan::default()
    });
    assert!(
        matches!(outcome, outputs::Outcome::Committed { .. }),
        "{outcome:?}"
    );
    let Some(Ok(observed)) = read else {
        panic!("the producer read the re-owned member: {read:?}")
    };
    assert!(observed.resolves_to_member);
    assert!(
        !observed.resolves_in_owner && !observed.not_in_other,
        "`direct` is now held by `b`, not `a`: {observed:?}"
    );
}
