//! Slice 5D: a decision that read or resolved a seam's name is keyed on that
//! seam between its endpoints, so a concurrent operation unlinking,
//! retiring, splitting or merging it makes the earlier attempt's commit
//! stale. A seam resolution resting on what no fact key tracks never seals.
use super::seam_names::{link, members, read_name, resolve, seam, seam_read, MEMBERS};
use super::*;

/// Four members, a seam `s` from `m1` to `m2` and an unrelated seam `o`
/// from `m3` to `m4`; returns the world and `s`'s name.
fn seamed(token: &str) -> (World, WorthQueryName<Seam>) {
    let world = members();
    let linked = |seam_key: &str| format!("{token}-link-{seam_key}");
    world.committed(&linked("s"), link("s", "m1", "m2", seam("weld", "seam", 1)));
    world.committed(
        &linked("o"),
        link("o", "m3", "m4", seam("weld-o", "seam", 1)),
    );
    let name = read_name(&world, &format!("{token}-read"), "m1", "m2");
    (world, name)
}

/// Prepares a probe whose decision reads `s`, runs `plan`, then commits the
/// prepared probe. `by_name` reads through `relation_name_of`; otherwise the
/// decision only resolves the held name, with no traversal of its own.
fn race(token: &str, by_name: bool, plan: Plan) -> Outcome {
    let (world, s) = seamed(token);
    if by_name {
        probe::seams::name(token, "s", "m1", "m2");
    } else {
        probe::seams::hold(token, "s", s, exact());
    }
    let key = format!("{token}:raced-seam-probe");
    let mut request = world
        .request()
        .advanced()
        .mutate(probe::Intent {
            token: token.to_owned(),
            keys: keys(&MEMBERS),
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
    let observed = probe::seams::take(token);
    if by_name {
        assert_eq!(seam_named(&observed, "s"), s);
    } else {
        assert_eq!(
            seam_resolution(&observed, "s"),
            &seam_between("s", "m1", "m2")
        );
    }
    world.committed(&format!("{token}-race"), plan);
    prepared.commit(worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
}

fn stale(outcome: &Outcome) -> bool {
    matches!(
        outcome,
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
    )
}

/// Unlinks `s` with no declared posture: its name retires with it.
fn unlink() -> Plan {
    Plan {
        reads: keys(&["m1", "m2"]),
        unlinks: vec![("m1".into(), "m2".into())],
        ..Plan::default()
    }
}

fn retire() -> Plan {
    Plan {
        seam_reads: vec![seam_read("s", "m1", "m2")],
        steps: vec![Step::SeamRetire(HeldSeam::Read("s".into()))],
        ..unlink()
    }
}

fn split() -> Plan {
    Plan {
        reads: keys(&MEMBERS),
        seam_reads: vec![seam_read("s", "m1", "m2")],
        unlinks: vec![("m1".into(), "m2".into())],
        seams: vec![
            ("p1".into(), "m1".into(), "m3".into()),
            ("p2".into(), "m1".into(), "m4".into()),
        ],
        steps: vec![Step::SeamSplit(
            HeldSeam::Read("s".into()),
            vec![
                ("p1".into(), seam("cut", "seam_piece", 1)),
                ("p2".into(), seam("cut", "seam_piece", 2)),
            ],
        )],
        ..Plan::default()
    }
}

/// Merges `s` and the unrelated seam `o` into a seam minted over both.
fn merge() -> Plan {
    Plan {
        reads: keys(&MEMBERS),
        seam_reads: vec![seam_read("s", "m1", "m2"), seam_read("o", "m3", "m4")],
        unlinks: vec![("m1".into(), "m2".into()), ("m3".into(), "m4".into())],
        seams: vec![("fused".into(), "m2".into(), "m3".into())],
        steps: vec![Step::SeamMerge(
            vec![HeldSeam::Read("s".into()), HeldSeam::Read("o".into())],
            SeamKeeper::MintNew("fused".into(), seam("fuse", "seam_piece", 9)),
        )],
        ..Plan::default()
    }
}

#[test]
fn unlinking_a_resolved_seam_invalidates_the_decision() {
    let outcome = race("seam-unlink-race", false, unlink());
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn retiring_a_resolved_seam_invalidates_the_decision() {
    let outcome = race("seam-retire-race", false, retire());
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn splitting_a_resolved_seam_invalidates_the_decision() {
    let outcome = race("seam-split-race", false, split());
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn merging_a_resolved_seam_away_invalidates_the_decision() {
    let outcome = race("seam-merge-race", false, merge());
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn retiring_a_seam_whose_name_was_read_invalidates_the_decision() {
    let outcome = race("seam-named-race", true, retire());
    assert!(stale(&outcome), "{outcome:?}");
}

#[test]
fn retiring_an_unrelated_seam_leaves_the_decision_current() {
    let outcome = race(
        "seam-unrelated-race",
        false,
        Plan {
            reads: keys(&["m3", "m4"]),
            seam_reads: vec![seam_read("o", "m3", "m4")],
            unlinks: vec![("m3".into(), "m4".into())],
            steps: vec![Step::SeamRetire(HeldSeam::Read("o".into()))],
            ..Plan::default()
        },
    );
    assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
}

fn vetoed(ended: &str) -> bool {
    ended.contains("PersistentName(UntrackedDependency)")
}

#[test]
fn a_swallowed_unknown_seam_name_never_seals() {
    // A seam name born only in another world: this one never saw its birth.
    let elsewhere = members();
    elsewhere.committed(
        "unknown-elsewhere",
        link("s", "m1", "m2", seam("elsewhere", "seam", 7)),
    );
    let foreign = read_name(&elsewhere, "unknown-elsewhere", "m1", "m2");
    let (world, _) = seamed("unknown");
    let (resolved, ended) = resolve(&world, "unknown-held", foreign, exact());
    assert!(
        matches!(&resolved, SeamObservation::ResolveRefused { refusal, .. } if refusal.contains("UnknownName")),
        "{resolved:?}"
    );
    assert!(vetoed(&ended), "an unborn name has no fact key: {ended}");
}

#[test]
fn a_seam_resolution_walking_ended_names_never_seals() {
    let (world, s) = seamed("ended");
    world.committed("ended-split", split());
    let (resolved, ended) = resolve(&world, "ended-all", s, NameSelection::AllDescendants);
    assert!(
        matches!(resolved, SeamObservation::Plural { .. }),
        "{resolved:?}"
    );
    assert!(
        vetoed(&ended),
        "a later Restore revives the ended name: {ended}"
    );
    let (resolved, ended) = resolve(&world, "ended-exact", s, exact());
    assert!(
        matches!(resolved, SeamObservation::Ambiguous { .. }),
        "{resolved:?}"
    );
    assert!(vetoed(&ended), "{ended}");
    let (world, s) = seamed("ended-retire");
    world.committed("ended-retired", retire());
    let (resolved, ended) = resolve(&world, "ended-retired-held", s, exact());
    assert!(
        matches!(resolved, SeamObservation::Retired { .. }),
        "{resolved:?}"
    );
    assert!(vetoed(&ended), "{ended}");
}
