//! Birth reads on an applied candidate (#345): a visible birth is the
//! candidate's answer whatever an earlier stage did to the record, while a
//! lookup that found no birth refuses, since the unpublished candidate may
//! bear exactly that birth.
use super::*;
use probe::births::{self, BirthObservation};

#[test]
fn a_later_stage_reads_the_birth_of_a_name_an_earlier_stage_retired() {
    let token = "cand-births";
    let world = seamed(token);
    let direct = world.names(&format!("{token}-basis"), &["direct"], 0)[0];
    births::birth_of(token, "direct", direct);
    let outcome = world.parts(
        token,
        vec![
            sculpted("cand-births-1", retired("direct", direct)),
            probed(token, &[]),
        ],
    );
    assert!(matches!(outcome, Envelope::Committed(_)), "{outcome:?}");
    let observed = births::take(token);
    let [BirthObservation::Born { birth, .. }] = observed.as_slice() else {
        panic!("the birth is read: {observed:?}");
    };
    assert_eq!(birth.name(), direct);
    assert_eq!(
        (birth.occurrence_key(), birth.role_path()),
        ("worth.bootstrap-seed/direct", "authored"),
        "a seed's occurrence is its bootstrap key"
    );
    assert!(
        !world.live("direct"),
        "the earlier stage's retirement landed"
    );
}

#[test]
fn a_lookup_of_a_birth_an_earlier_stage_may_bear_refuses_and_commits_nothing() {
    let token = "cand-births-unborn";
    let world = seamed(token);
    births::name_for_birth(
        token,
        "fresh",
        outputs::birth("cand-births-fresh", "authored"),
        Vec::new(),
    );
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                "cand-births-unborn-1",
                created("fresh", "cand-births-fresh"),
            ),
            probed(token, &[]),
        ],
    );
    let observed = births::take(token);
    let [BirthObservation::LookupRefused { refusal, .. }] = observed.as_slice() else {
        panic!("the lookup refuses: {observed:?}");
    };
    assert_eq!(refusal, "CandidateNameHistory");
    nothing_committed(&world, &outcome, &["fresh"]);
}

#[test]
fn the_birth_of_a_name_an_earlier_stage_may_bear_refuses_and_commits_nothing() {
    // The name a fresh world's birth of `fresh` mints, read from a world that
    // already committed that very birth.
    let token = "cand-births-held";
    let committed = seamed(&format!("{token}-elsewhere"));
    committed.committed(
        &format!("{token}-elsewhere-fresh"),
        created("fresh", "cand-births-held-fresh"),
    );
    let held = committed.names(&format!("{token}-elsewhere-names"), &["fresh"], 0)[0];

    let world = seamed(token);
    births::birth_of(token, "fresh", held);
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                "cand-births-held-1",
                created("fresh", "cand-births-held-fresh"),
            ),
            probed(token, &[]),
        ],
    );
    let observed = births::take(token);
    let [BirthObservation::BirthRefused { refusal, .. }] = observed.as_slice() else {
        panic!("the birth read refuses: {observed:?}");
    };
    assert_eq!(refusal, "CandidateNameHistory");
    nothing_committed(&world, &outcome, &["fresh"]);
}
