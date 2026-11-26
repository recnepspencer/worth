//! #426: a later member reads the names and births the envelope gives the
//! records earlier members created and named, exactly as the commit then
//! records them. A chain of members, each reading what the one before it
//! created, seals and commits; a member that refuses after reading such a
//! name leaves no trace of it.
use super::*;
use outputs::MemberCut;
use probe::births::{self, BirthObservation};
use worth_query_decl::facade::application_naming::RecordedNameBirth;

/// The birth observed under `label`.
fn born<'a>(observed: &'a [BirthObservation], label: &str) -> &'a RecordedNameBirth<Member> {
    observed
        .iter()
        .find_map(|observation| match observation {
            BirthObservation::Born { label: l, birth } if l == label => Some(birth),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{label}`'s birth reads: {observed:?}"))
}

const CHAIN: [&str; 6] = ["fresh", "p1", "p2", "merged", "again", "nested"];

/// Reads every `CHAIN` member's name and birth, and seam `s2`'s name.
fn chain_probe(token: &str) {
    probe::seams::name(token, "s2", "fresh", "nested");
    for key in &CHAIN[..5] {
        births::birth_of_member(token, key, key);
    }
}

/// Stage A creates `fresh` and seam `s2` from it, both named; stage B reads
/// `fresh`'s name, splits `direct`, merges `foreign` and `loose` into a
/// minted `merged`, and bears `again` under a retired name's birth (a
/// birth-classified `Restore`); stage C, the probe, reads them all.
#[test]
fn a_chain_of_stages_reads_every_earlier_birth_and_commits_it_exactly() {
    let token = "pend-chain";
    let world = World::install();
    let basis = world.names(
        &format!("{token}-basis"),
        &["direct", "foreign", "loose"],
        0,
    );
    world.committed(&format!("{token}-old"), created("old", "pend-chain-old"));
    let old = world.names(&format!("{token}-old-name"), &["old"], 1)[0];
    world.committed(&format!("{token}-retire"), retired("old", old));
    chain_probe(token);
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                "pend-chain-a",
                Plan {
                    creates: keys(&["fresh"]),
                    reads: keys(&["nested"]),
                    seams: vec![("s2".into(), "fresh".into(), "nested".into())],
                    steps: vec![
                        Step::Birth("fresh".into(), birth("pend-chain-fresh", "authored")),
                        Step::SeamBirth("s2".into(), seam("pend-chain-weld", "seam", 2)),
                    ],
                    ..Plan::default()
                },
            ),
            sculpted(
                "pend-chain-b",
                Plan {
                    reads: keys(&["fresh"]),
                    plain_reads: keys(&["direct", "foreign", "loose"]),
                    creates: keys(&["p1", "p2", "merged", "again"]),
                    deletes: keys(&["direct", "foreign", "loose"]),
                    steps: vec![
                        Step::Split(
                            sculpt::Held::Name(basis[0]),
                            vec![
                                ("p1".into(), outputs::piece(token, 1, None)),
                                ("p2".into(), outputs::piece(token, 2, None)),
                            ],
                        ),
                        Step::Merge(
                            vec![sculpt::Held::Name(basis[1]), sculpt::Held::Name(basis[2])],
                            sculpt::Keeper::MintNew(
                                "merged".into(),
                                birth("pend-chain-merge", "merged").with_discriminator(
                                    &MemberCut::Side {
                                        side: 0,
                                        cutter: None,
                                    },
                                ),
                            ),
                        ),
                        Step::Birth("again".into(), birth("pend-chain-old", "authored")),
                    ],
                    ..Plan::default()
                },
            ),
            probed(token, &CHAIN),
        ],
    );
    assert!(matches!(outcome, Envelope::Committed(_)), "{outcome:?}");
    let (names, seams, read) = (
        probe::take(token),
        probe::seams::take(token),
        births::take(token),
    );
    assert_eq!(outputs::named(&names, "again"), old, "the restore's name");
    let piece = |side| MemberCut::Side { side, cutter: None };
    assert_eq!(born(&read, "p1").decode_discriminator(), Ok(Some(piece(1))));
    assert_eq!(born(&read, "p2").decode_discriminator(), Ok(Some(piece(2))));
    assert_eq!(
        born(&read, "merged").decode_discriminator(),
        Ok(Some(piece(0)))
    );
    for key in &CHAIN[..5] {
        assert_eq!(
            born(&read, key).name(),
            outputs::named(&names, key),
            "{key}"
        );
    }
    // The committed world records exactly what the candidate read showed.
    let after = format!("{token}-after");
    chain_probe(&after);
    let committed = world.probe(&after, &CHAIN, 2);
    for key in CHAIN {
        assert_eq!(
            outputs::named(&committed, key),
            outputs::named(&names, key),
            "{key}"
        );
    }
    assert_eq!(
        outputs::seam_named(&probe::seams::take(&after), "s2"),
        outputs::seam_named(&seams, "s2")
    );
    assert_eq!(births::take(&after), read, "every birth, byte for byte");
}

/// A pending piece's name answers, but the parent it was split from ended:
/// the stage refuses, the envelope commits nothing, and the branch's names
/// keep no trace of the piece's birth until a commit really bears it.
#[test]
fn a_refusal_after_reading_a_pending_name_leaves_no_trace_of_it() {
    let token = "pend-abort";
    let world = World::install();
    let direct = world.names(&format!("{token}-basis"), &["direct"], 0)[0];
    probe::hold(token, "direct", direct);
    let split = Plan {
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
    };
    let outcome = world.parts(
        token,
        vec![
            sculpted("pend-abort-1", split.clone()),
            probed(token, &["p1"]),
        ],
    );
    let observed = probe::take(token);
    let pending = outputs::named(&observed, "p1");
    let refusal = refused(outputs::resolution(&observed, "direct"));
    assert!(entity_denial("EndedInCandidate", refusal), "{refusal}");
    nothing_committed(&world, &outcome, &["p1", "p2"]);
    assert!(world.live("direct"));
    let unborn = format!("{token}-unborn");
    births::birth_of(&unborn, "p1", pending);
    world.probe(&unborn, &[], 1);
    let unborn = births::take(&unborn);
    let [BirthObservation::BirthRefused { refusal, .. }] = unborn.as_slice() else {
        panic!("the aborted birth is not recorded")
    };
    assert!(refusal.contains("UnknownName"), "{refusal}");
    world.committed(&format!("{token}-commit"), split);
    let after = world.names(&format!("{token}-after"), &["p1"], 2)[0];
    assert_eq!(
        after, pending,
        "the name a real commit of that birth records"
    );
}
