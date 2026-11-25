//! The two traps a candidate-basis resolution must not fall into:
//!
//! - a held name resolving to siblings `{p1, p2}` at the basis, after an
//!   earlier stage retired `p1`, never silently becomes `Exact(p2)` nor the
//!   stale `Ambiguous({p1, p2})`;
//! - a held name `Retired` or unknown at the basis, after an earlier stage
//!   regenerated or bore it on a created record, never answers the stale
//!   `Retired` / `UnknownName`. The committed world after such a stage
//!   resolves the name live, proving the basis answer false.
use super::*;
use worth_query_decl::facade::application_naming::NameSelection;

/// `direct` split by a committed run into `p1` and `p2`; their names.
fn split(world: &World, token: &str) -> [WorthQueryName<Member>; 3] {
    let direct = world.names(&format!("{token}-basis"), &["direct"], 0)[0];
    world.committed(
        &format!("{token}-split"),
        Plan {
            plain_reads: keys(&["direct"]),
            creates: keys(&["p1", "p2"]),
            deletes: keys(&["direct"]),
            steps: vec![Step::Split(
                sculpt::Held::Name(direct),
                vec![
                    ("p1".into(), outputs::piece(token, 1, None)),
                    ("p2".into(), outputs::piece(token, 2, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    let pieces = world.names(&format!("{token}-pieces"), &["p1", "p2"], 1);
    [direct, pieces[0], pieces[1]]
}

#[test]
fn an_unpublished_sibling_retirement_never_collapses_an_ambiguous_name() {
    let token = "cand-sibling";
    let world = World::install();
    let [direct, p1, p2] = split(&world, token);
    // At the basis the parent's name is ambiguous between both pieces.
    let basis = format!("{token}-ambiguous");
    probe::hold(&basis, "direct", direct);
    let observed = world.probe(&basis, &["p1", "p2"], 2);
    match outputs::resolution(&observed, "direct") {
        probe::Observation::Ambiguous { candidates, .. } => {
            let mut candidates = candidates.clone();
            candidates.sort();
            let mut pieces = vec![p1, p2];
            pieces.sort();
            assert_eq!(candidates, pieces);
        }
        other => panic!("the split parent's name is ambiguous at the basis: {other:?}"),
    }
    probe::hold(token, "direct", direct);
    probe::hold_selecting(token, "direct-all", direct, NameSelection::AllDescendants);
    let outcome = world.parts(
        token,
        vec![
            sculpted(&format!("{token}-1"), retired("p1", p1)),
            probed(token, &["p2"]),
        ],
    );
    let observed = probe::take(token);
    for label in ["direct", "direct-all"] {
        let refusal = refused(outputs::resolution(&observed, label));
        assert!(
            entity_denial("EndedInCandidate", refusal),
            "{label}: never Exact(p2) nor stale Ambiguous: {refusal}"
        );
    }
    assert_eq!(
        outputs::named(&observed, "p2"),
        p2,
        "the survivor still reads"
    );
    nothing_committed(&world, &outcome, &[]);
    assert!(world.live("p1"));
}

#[test]
fn a_name_an_earlier_stage_regenerates_never_reads_as_retired() {
    let token = "cand-regenerate";
    let world = World::install();
    world.committed(&format!("{token}-born"), created("old", token));
    let name = world.names(&format!("{token}-basis"), &["old"], 0)[0];
    world.committed(&format!("{token}-retire"), retired("old", name));
    let basis = format!("{token}-retired");
    probe::hold(&basis, "held", name);
    let observed = world.probe(&basis, &[], 1);
    assert_eq!(
        outputs::resolution(&observed, "held"),
        &probe::Observation::Retired {
            label: "held".into()
        }
    );
    // An earlier stage regenerates the exact name on a created record.
    probe::hold(token, "held", name);
    let outcome = world.parts(
        token,
        vec![
            sculpted(&format!("{token}-1"), created("again", token)),
            probed(token, &[]),
        ],
    );
    let observed = probe::take(token);
    assert_eq!(
        refused(outputs::resolution(&observed, "held")),
        "CandidateNameHistory"
    );
    nothing_committed(&world, &outcome, &["again"]);
    // Committed, the same stage makes the name live again: `Retired` was false.
    world.committed(&format!("{token}-commit"), created("again", token));
    let after = format!("{token}-after");
    probe::hold(&after, "held", name);
    let observed = world.probe(&after, &["again"], 2);
    assert_eq!(
        outputs::resolution(&observed, "held"),
        &outputs::resolved_to("held", "again")
    );
}

#[test]
fn a_name_an_earlier_stage_bears_never_reads_as_unknown() {
    let token = "cand-bear";
    // The name another world minted: unknown here until a stage bears it.
    let elsewhere = World::install();
    elsewhere.committed(&format!("{token}-elsewhere"), created("born", token));
    let name = elsewhere.names(&format!("{token}-minted"), &["born"], 0)[0];
    let world = World::install();
    let basis = format!("{token}-unknown");
    probe::hold(&basis, "held", name);
    let observed = world.probe(&basis, &[], 1);
    assert!(
        refused(outputs::resolution(&observed, "held")).contains("UnknownName"),
        "{observed:?}"
    );
    probe::hold(token, "held", name);
    let outcome = world.parts(
        token,
        vec![
            sculpted(&format!("{token}-1"), created("born", token)),
            probed(token, &[]),
        ],
    );
    let observed = probe::take(token);
    assert_eq!(
        refused(outputs::resolution(&observed, "held")),
        "CandidateNameHistory"
    );
    nothing_committed(&world, &outcome, &["born"]);
    world.committed(&format!("{token}-commit"), created("born", token));
    let after = format!("{token}-after");
    probe::hold(&after, "held", name);
    let observed = world.probe(&after, &["born"], 2);
    assert_eq!(
        outputs::resolution(&observed, "held"),
        &outputs::resolved_to("held", "born")
    );
}
