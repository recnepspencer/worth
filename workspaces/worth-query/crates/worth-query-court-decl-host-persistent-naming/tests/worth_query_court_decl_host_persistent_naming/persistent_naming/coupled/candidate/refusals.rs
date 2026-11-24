//! Candidate-basis reads an earlier stage made false refuse typed: a record
//! the candidate created has no published name, and a name whose record the
//! candidate ended (deleted, retired, split or unlinked) no longer
//! designates it. Each refusal vetoes sealing, so the envelope commits
//! nothing from any stage.
use super::*;

#[test]
fn names_of_records_an_earlier_stage_created_refuse_unpublished() {
    let token = "cand-created";
    let world = World::install();
    probe::seams::name(token, "s2", "direct", "nested");
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                "cand-created-1",
                Plan {
                    creates: keys(&["fresh"]),
                    reads: keys(&["direct", "nested"]),
                    seams: vec![("s2".into(), "direct".into(), "nested".into())],
                    steps: vec![
                        Step::Birth("fresh".into(), birth("cand-created-fresh", "authored")),
                        Step::SeamBirth("s2".into(), seam("cand-created-weld", "seam", 2)),
                    ],
                    ..Plan::default()
                },
            ),
            probed(token, &["fresh", "direct", "nested"]),
        ],
    );
    let observed = probe::take(token);
    let fresh = observed
        .iter()
        .find(|o| matches!(o, probe::Observation::NameRefused { key, .. } if key == "fresh"))
        .unwrap_or_else(|| panic!("`fresh` is read and refused: {observed:?}"));
    assert!(
        entity_denial("UnpublishedCandidateRecord", refused(fresh)),
        "{fresh:?}"
    );
    let seams = probe::seams::take(token);
    let s2 = seams
        .iter()
        .find_map(|o| match o {
            probe::seams::SeamObservation::NameRefused { label, refusal } if label == "s2" => {
                Some(refusal)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("`s2` is read and refused: {seams:?}"));
    assert!(relation_denial("UnpublishedCandidateRecord", s2), "{s2}");
    nothing_committed(&world, &outcome, &["fresh"]);
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

#[test]
fn a_split_in_an_earlier_stage_refuses_the_parent_and_unpublished_pieces() {
    let token = "cand-split";
    let world = World::install();
    let direct = world.names(&format!("{token}-basis"), &["direct"], 0)[0];
    probe::hold(token, "direct", direct);
    probe::hold_selecting(
        token,
        "direct-all",
        direct,
        worth_query_decl::facade::application_naming::NameSelection::AllDescendants,
    );
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                "cand-split-1",
                Plan {
                    plain_reads: keys(&["direct"]),
                    creates: keys(&["p1", "p2"]),
                    deletes: keys(&["direct"]),
                    steps: vec![Step::Split(
                        sculpt::Held::Name(direct),
                        vec![
                            ("p1".into(), outputs::piece("cand-split", 1, None)),
                            ("p2".into(), outputs::piece("cand-split", 2, None)),
                        ],
                    )],
                    ..Plan::default()
                },
            ),
            probed(token, &["p1"]),
        ],
    );
    let observed = probe::take(token);
    let p1 = observed
        .iter()
        .find(|o| matches!(o, probe::Observation::NameRefused { key, .. } if key == "p1"))
        .unwrap_or_else(|| panic!("piece `p1` is read and refused: {observed:?}"));
    assert!(
        entity_denial("UnpublishedCandidateRecord", refused(p1)),
        "{p1:?}"
    );
    for label in ["direct", "direct-all"] {
        let refusal = refused(outputs::resolution(&observed, label));
        assert!(
            entity_denial("EndedInCandidate", refusal),
            "{label}: {refusal}"
        );
    }
    nothing_committed(&world, &outcome, &["p1", "p2"]);
    assert!(world.live("direct"));
}
