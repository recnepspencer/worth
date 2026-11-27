//! Slice 5D: a decision that read or resolved a seam's name is keyed on that
//! seam between its endpoints, so a concurrent operation unlinking,
//! retiring, splitting or merging it makes the earlier attempt's commit
//! stale. A seam resolution resting on name history (ended names on its
//! walked path, or a held name not yet born) seals on that history as a
//! `PersistentNameHistory` fact: a concurrent commit changing it stales the
//! decision, and an unrelated one leaves it current.
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
    let (observed, outcome) = prepare_then_commit(&world, token, plan);
    if by_name {
        assert_eq!(seam_named(&observed, "s"), s);
    } else {
        assert_eq!(
            seam_resolution(&observed, "s"),
            &seam_between("s", "m1", "m2")
        );
    }
    outcome
}

/// Prepares the probe registered under `token`, commits `plan` before the
/// prepared attempt commits, and returns what the prepared decision observed
/// and how its commit ended.
fn prepare_then_commit(world: &World, token: &str, plan: Plan) -> (Vec<SeamObservation>, Outcome) {
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
    world.committed(&format!("{token}-race"), plan);
    let outcome = prepared
        .commit(worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation);
    (observed, outcome)
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

/// Unlinks `s` and links another seam between the same members under the
/// same name (`Preserve`): the name and the endpoints stay, the seam changes.
fn replace() -> Plan {
    Plan {
        seam_reads: vec![seam_read("s", "m1", "m2")],
        seams: vec![("s2".into(), "m1".into(), "m2".into())],
        steps: vec![
            Step::SeamRetire(HeldSeam::Read("s".into())),
            Step::SeamBirth("s2".into(), seam("weld", "seam", 1)),
        ],
        ..unlink()
    }
}

#[test]
fn replacing_a_resolved_seam_between_the_same_members_invalidates_the_decision() {
    let outcome = race("seam-replace-race", false, replace());
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

#[test]
fn a_seam_handle_another_world_issued_is_foreign() {
    let elsewhere = members();
    elsewhere.committed(
        "foreign-elsewhere",
        link("s", "m1", "m2", seam("weld", "seam", 1)),
    );
    read_name(&elsewhere, "foreign-elsewhere", "m1", "m2");
    let world = members();
    world.committed(
        "foreign-here",
        link("s", "m1", "m2", seam("weld", "seam", 1)),
    );
    // The other world's reader issued the handle its probe kept.
    probe::seams::replay("foreign-here", "theirs", "foreign-elsewhere", "read");
    let (observed, ended) = world.probe_seams("foreign-here", &MEMBERS, 0);
    assert!(
        matches!(
            observed.as_slice(),
            [SeamObservation::NameRefused { label, refusal }]
                if label == "theirs" && refusal == "ForeignIdentity"
        ),
        "{observed:?}"
    );
    assert!(
        ended.contains("Committed"),
        "a handle refused before any read leaves nothing to veto: {ended}"
    );
}

fn sealed(ended: &str) -> bool {
    ended.contains("Committed")
}

/// Retires the unrelated seam `o` from `m3` to `m4`.
fn retire_unrelated() -> Plan {
    Plan {
        reads: keys(&["m3", "m4"]),
        seam_reads: vec![seam_read("o", "m3", "m4")],
        unlinks: vec![("m3".into(), "m4".into())],
        steps: vec![Step::SeamRetire(HeldSeam::Read("o".into()))],
        ..Plan::default()
    }
}

/// The seam name another world minted from `m1` to `m2`; this one has not
/// born it until it links the very same seam.
fn unborn_here() -> (WorthQueryName<Seam>, Plan) {
    let elsewhere = members();
    let plan = || link("s", "m1", "m2", seam("elsewhere", "seam", 7));
    elsewhere.committed("unborn-elsewhere", plan());
    (
        read_name(&elsewhere, "unborn-elsewhere", "m1", "m2"),
        plan(),
    )
}

#[test]
fn a_swallowed_unknown_seam_name_seals_on_its_awaited_birth() {
    let (foreign, _) = unborn_here();
    let (world, _) = seamed("unknown");
    let (resolved, ended) = resolve(&world, "unknown-held", foreign, exact());
    assert!(
        matches!(&resolved, SeamObservation::ResolveRefused { refusal, .. } if refusal.contains("UnknownName")),
        "{resolved:?}"
    );
    assert!(
        sealed(&ended),
        "an unborn name seals on its awaited birth: {ended}"
    );
}

#[test]
fn the_awaited_birth_stales_the_decision_and_an_unrelated_commit_does_not() {
    let (foreign, birth) = unborn_here();
    let world = members();
    world.committed(
        "awaited-o",
        link("o", "m3", "m4", seam("weld-o", "seam", 1)),
    );
    probe::seams::hold("awaited-born", "held", foreign, exact());
    let (observed, outcome) = prepare_then_commit(&world, "awaited-born", birth);
    assert!(
        matches!(seam_resolution(&observed, "held"), SeamObservation::ResolveRefused { refusal, .. } if refusal.contains("UnknownName")),
        "{observed:?}"
    );
    assert!(
        stale(&outcome),
        "birthing the awaited name stales: {outcome:?}"
    );

    let world = members();
    world.committed(
        "awaited-o",
        link("o", "m3", "m4", seam("weld-o", "seam", 1)),
    );
    probe::seams::hold("awaited-other", "held", foreign, exact());
    let (_, outcome) = prepare_then_commit(&world, "awaited-other", retire_unrelated());
    assert!(
        matches!(outcome, Outcome::Committed { .. }),
        "an unrelated commit leaves the awaited birth current: {outcome:?}"
    );
}

#[test]
fn a_seam_resolution_walking_ended_names_seals() {
    let (world, s) = seamed("ended");
    world.committed("ended-split", split());
    let (resolved, ended) = resolve(&world, "ended-all", s, NameSelection::AllDescendants);
    assert!(
        matches!(resolved, SeamObservation::Plural { .. }),
        "{resolved:?}"
    );
    assert!(
        sealed(&ended),
        "the ended path is keyed, not vetoed: {ended}"
    );
    let (resolved, ended) = resolve(&world, "ended-exact", s, exact());
    assert!(
        matches!(resolved, SeamObservation::Ambiguous { .. }),
        "{resolved:?}"
    );
    assert!(sealed(&ended), "{ended}");
    let (world, s) = seamed("ended-retire");
    world.committed("ended-retired", retire());
    let (resolved, ended) = resolve(&world, "ended-retired-held", s, exact());
    assert!(
        matches!(resolved, SeamObservation::Retired { .. }),
        "{resolved:?}"
    );
    assert!(sealed(&ended), "{ended}");
}

#[test]
fn a_walk_through_ended_names_stales_on_its_path_and_stays_current_off_it() {
    let walked = |token: &str, plan: Plan| {
        let (world, s) = seamed(token);
        world.committed(&format!("{token}-split"), split());
        probe::seams::hold(token, "held", s, NameSelection::AllDescendants);
        let (observed, outcome) = prepare_then_commit(&world, token, plan);
        assert!(
            matches!(
                seam_resolution(&observed, "held"),
                SeamObservation::Plural { .. }
            ),
            "{observed:?}"
        );
        outcome
    };
    // Retiring piece `p1`, a live successor on the walked path, stales.
    let retire_piece = Plan {
        reads: keys(&["m1", "m3"]),
        seam_reads: vec![seam_read("p1", "m1", "m3")],
        unlinks: vec![("m1".into(), "m3".into())],
        steps: vec![Step::SeamRetire(HeldSeam::Read("p1".into()))],
        ..Plan::default()
    };
    let outcome = walked("walk-piece", retire_piece);
    assert!(stale(&outcome), "{outcome:?}");
    let outcome = walked("walk-unrelated", retire_unrelated());
    assert!(
        matches!(outcome, Outcome::Committed { .. }),
        "an edit off the walked path leaves it current: {outcome:?}"
    );
}
