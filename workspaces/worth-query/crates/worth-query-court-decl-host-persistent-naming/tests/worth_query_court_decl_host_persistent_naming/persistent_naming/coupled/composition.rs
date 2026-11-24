//! An envelope's stages name what they create, split, merge and retire, and
//! every name is byte-identical to the one the same plans mint as separate
//! direct operations. The two schemes, parallel seams, a split across two
//! owners (#133) and a least-keeper merge in place (#314) all ride one
//! composed plan. A later stage reads at the envelope's applied candidate,
//! where names are not read (`CandidateBasis`), so it acts on names it holds.
use super::sculpt::Held;
use super::*;

/// Stage one: two members and their seam, named manually.
fn first() -> Plan {
    Plan {
        creates: keys(&["c1", "c2"]),
        seams: vec![("c-seam".into(), "c1".into(), "c2".into())],
        steps: vec![
            Step::Birth("c1".into(), birth("coupled-c1", "authored")),
            Step::Birth("c2".into(), birth("coupled-c2", "authored")),
            Step::SeamBirth("c-seam".into(), birth("coupled-seam", "seam_piece")),
        ],
        ..Plan::default()
    }
}

/// Stage two, through one naming scope: two members with two parallel
/// seams told apart by index, a split of a seeded member into pieces owned
/// by two models, a least-keeper merge in place and a retirement, each of a
/// name `held` (`direct`, `nested`, `foreign`, `loose`) carries.
fn second(held: &[WorthQueryName<Member>]) -> Plan {
    let (_, merged_away) = least(held);
    let held = |index: usize| Held::Name(held[index]);
    Plan {
        plain_reads: keys(&["direct", "nested", "foreign", "loose"]),
        deletes: keys(&[merged_away, "foreign", "loose"]),
        creates: keys(&["d1", "d2", "f1", "f2"]),
        owners: vec![("f1".into(), "a".into()), ("f2".into(), "b".into())],
        steps: vec![
            scoped(
                "coupled-d",
                vec![
                    ScopeStep::Name("d1".into(), MemberRole::Authored),
                    ScopeStep::CutBy {
                        key: "d2".into(),
                        side: outputs::CutSide::Positive,
                        of: "d1".into(),
                    },
                    ScopeStep::LinkSeamOf {
                        key: "d-seam-0".into(),
                        from: "d1".into(),
                        to: "d2".into(),
                        index: 0,
                        of: "d1".into(),
                    },
                    ScopeStep::LinkSeamOf {
                        key: "d-seam-1".into(),
                        from: "d2".into(),
                        to: "d1".into(),
                        index: 1,
                        of: "d1".into(),
                    },
                    ScopeStep::MergeLeast(vec![held(0), held(1)], None),
                ],
            ),
            Step::Split(
                held(2),
                vec![
                    ("f1".into(), piece("coupled-cut", 1, None)),
                    ("f2".into(), piece("coupled-cut", 2, None)),
                ],
            ),
            Step::Retire(held(3)),
        ],
        ..Plan::default()
    }
}

/// The member whose name sorts least among `direct` and `nested` keeps its
/// record; the other is merged away.
fn least(held: &[WorthQueryName<Member>]) -> (&'static str, &'static str) {
    if held[0] < held[1] {
        ("direct", "nested")
    } else {
        ("nested", "direct")
    }
}

const CREATED: [&str; 6] = ["c1", "c2", "d1", "d2", "f1", "f2"];

#[test]
fn envelope_stages_mint_the_names_direct_operations_mint() {
    let direct = World::install();
    let held = direct.names(
        "composed-held",
        &["direct", "nested", "foreign", "loose"],
        0,
    );
    direct.committed("direct-first", first());
    direct.committed("direct-second", second(&held));
    let expected = direct.names("composed-direct-names", &CREATED, 1);

    let coupled = World::install();
    assert_eq!(
        coupled.names(
            "composed-held",
            &["direct", "nested", "foreign", "loose"],
            0
        ),
        held,
        "both worlds seed the same names"
    );
    coupled.committed_envelope(
        "composed",
        vec![
            stage("composed-first", first()),
            stage("composed-second", second(&held)),
        ],
    );
    assert_eq!(
        coupled.names("composed-coupled-names", &CREATED, 1),
        expected,
        "each stage's outputs carry the names direct operations mint"
    );
    let (kept, merged_away) = least(&held);
    for key in [merged_away, "foreign", "loose"] {
        assert!(!coupled.live(key), "{key} was consumed by the envelope");
    }

    // Held names follow the composed lineage exactly as after the direct runs.
    for world in [&direct, &coupled] {
        for (label, name) in ["direct", "nested", "foreign", "loose"].iter().zip(&held) {
            probe::hold("composed-lineage", label, *name);
        }
        let observed = world.probe("composed-lineage", &[kept], 2);
        let resolution = |label| outputs::resolution(&observed, label).clone();
        assert_eq!(
            resolution(merged_away),
            outputs::resolved_to(merged_away, kept)
        );
        assert_eq!(resolution(kept), outputs::resolved_to(kept, kept));
        assert_eq!(
            resolution("loose"),
            probe::Observation::Retired {
                label: "loose".into()
            }
        );
        assert!(
            matches!(
                resolution("foreign"),
                probe::Observation::Ambiguous { .. } | probe::Observation::AmbiguousOwned { .. }
            ),
            "the split parent resolves to both owned pieces: {observed:?}"
        );
    }
}

#[test]
fn an_envelope_is_one_commit_whose_receipt_names_every_stage() {
    let world = World::install();
    let held = world.names(
        "composed-one-held",
        &["direct", "nested", "foreign", "loose"],
        0,
    );
    let outcome = world.envelope(
        "one-commit",
        vec![
            stage("one-first", first()),
            stage("one-second", second(&held)),
        ],
    );
    let Envelope::Committed(receipt) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(receipt.stage_count(), 2);
    drop(receipt);
    let replay = world.envelope(
        "one-commit",
        vec![
            stage("one-first", first()),
            stage("one-second", second(&held)),
        ],
    );
    assert!(
        matches!(
            replay,
            Envelope::AlreadyCommitted(_) | Envelope::RecoveredCommitted(_)
        ),
        "a replay re-mints nothing: {replay:?}"
    );
    assert_eq!(
        world.names("composed-one-names", &CREATED, 1).len(),
        CREATED.len()
    );
}
