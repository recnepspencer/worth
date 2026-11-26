//! Candidate-basis reads an earlier stage made false refuse typed: a record
//! the candidate created that no stage named has no name, a name whose
//! record the candidate ended (deleted, retired, split or unlinked) no
//! longer designates it, and an earlier stage's naming plan the commit would
//! refuse fails every pending read closed. Each refusal vetoes sealing, so
//! the envelope commits nothing from any stage.
use super::*;

/// A record an earlier stage created but named nothing has no name yet:
/// `fresh`'s owner link (`ModelMember` has no scheme) refuses unpublished,
/// though `fresh` itself reads its pending name, and the envelope commits
/// nothing.
#[test]
fn an_unnamed_record_an_earlier_stage_created_refuses_unpublished() {
    let token = "cand-created";
    let world = World::install();
    probe::owner_links::name(token, "link", "fresh");
    let outcome = world.parts(
        token,
        vec![
            sculpted("cand-created-1", created("fresh", "cand-created-fresh")),
            probed(token, &["fresh"]),
        ],
    );
    let observed = probe::take(token);
    assert!(
        matches!(&observed[..], [probe::Observation::Named { .. }]),
        "{observed:?}"
    );
    let links = probe::owner_links::take(token);
    let [probe::owner_links::LinkObservation::NameRefused { refusal, .. }] = links.as_slice()
    else {
        panic!("the unnamed link is read and refused: {links:?}")
    };
    assert!(
        relation_denial("UnpublishedCandidateRecord", refusal),
        "{refusal}"
    );
    nothing_committed(&world, &outcome, &["fresh"]);
}

/// An earlier stage's plan the commit would refuse fails the read closed:
/// it refuses with that exact naming denial, the stage cannot seal, and the
/// envelope commits nothing.
fn refuses_with_plan_denial(token: &str, stages: Vec<Part>, denial: &str) {
    let world = World::install();
    let mut parts = stages;
    parts.push(probed(token, &["x"]));
    let outcome = world.parts(token, parts);
    let observed = probe::take(token);
    let [probe::Observation::NameRefused { refusal, .. }] = observed.as_slice() else {
        panic!("`x` is read and refused: {observed:?}")
    };
    assert!(
        refusal.starts_with(&format!("CandidateNaming(Naming({denial}")),
        "{refusal}"
    );
    nothing_committed(&world, &outcome, &["x", "y"]);
}

#[test]
fn a_missing_birth_in_an_earlier_stage_refuses_the_read_closed() {
    let bare = Plan {
        creates: keys(&["x"]),
        ..Plan::default()
    };
    refuses_with_plan_denial(
        "cand-missing",
        vec![sculpted("cand-missing-1", bare)],
        "MissingBirth",
    );
}

#[test]
fn a_duplicate_birth_across_earlier_stages_refuses_the_read_closed() {
    refuses_with_plan_denial(
        "cand-duplicate",
        vec![
            sculpted("cand-duplicate-1", created("x", "cand-duplicate")),
            sculpted("cand-duplicate-2", created("y", "cand-duplicate")),
        ],
        "DuplicateBirth",
    );
}

#[test]
fn names_whose_records_an_earlier_stage_ended_refuse_ended() {
    let token = "cand-ended";
    let world = seamed(token);
    let loose = world.names(&format!("{token}-basis"), &["loose"], 0)[0];
    let s1 = world.seam_name(&format!("{token}-basis-seam"), "direct", "nested");
    probe::hold(token, "loose", loose);
    probe::hold_in(token, "loose-in-b", loose, outputs::exact(), "b");
    probe::seams::hold(token, "s1", s1, outputs::exact());
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                "cand-ended-1",
                Plan {
                    reads: keys(&["direct", "nested"]),
                    plain_reads: keys(&["loose"]),
                    deletes: keys(&["loose"]),
                    unlinks: vec![("direct".into(), "nested".into())],
                    steps: vec![
                        Step::Retire(sculpt::Held::Name(loose)),
                        Step::SeamRetire(sculpt::HeldSeam::Name(s1)),
                    ],
                    ..Plan::default()
                },
            ),
            probed(token, &["loose", "direct", "nested"]),
        ],
    );
    let observed = probe::take(token);
    assert!(
        observed.contains(&probe::Observation::Absent {
            key: "loose".into()
        }),
        "a deleted record has no handle to ask its name with: {observed:?}"
    );
    for label in ["loose", "loose-in-b"] {
        let refusal = refused(outputs::resolution(&observed, label));
        assert!(
            entity_denial("EndedInCandidate", refusal),
            "{label}: {refusal}"
        );
    }
    let seams = probe::seams::take(token);
    match outputs::seam_resolution(&seams, "s1") {
        probe::seams::SeamObservation::ResolveRefused { refusal, .. } => {
            assert!(relation_denial("EndedInCandidate", refusal), "{refusal}")
        }
        other => panic!("the unlinked seam's name refuses: {other:?}"),
    }
    nothing_committed(&world, &outcome, &[]);
    assert!(world.live("loose"), "the retirement never committed");
}
