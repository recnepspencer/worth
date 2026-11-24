//! The composed plan is judged whole: a name two stages both mint, a schemed
//! record a later stage leaves unnamed, and a record a later stage deletes
//! after an earlier stage named it each refuse the entire envelope typed,
//! with zero effects from any stage.
use super::*;
use worth_query_host::facade::application_naming::LineageRecordTarget;

fn created(key: &str, occurrence: &str) -> Plan {
    Plan {
        creates: keys(&[key]),
        steps: vec![Step::Birth(key.into(), birth(occurrence, "authored"))],
        ..Plan::default()
    }
}

#[test]
fn two_stages_minting_one_name_refuse_duplicate_birth_with_no_effect() {
    let world = World::install();
    let outcome = world.envelope(
        "coupled-duplicate",
        vec![
            stage("coupled-duplicate-1", created("x1", "coupled-same")),
            stage("coupled-duplicate-2", created("x2", "coupled-same")),
        ],
    );
    assert!(
        matches!(
            envelope_refusal(&outcome),
            Some(LineageNameDenial::DuplicateBirth { .. })
        ),
        "{outcome:?}"
    );
    assert!(!world.live("x1") && !world.live("x2"), "no stage committed");
    // Told apart, the same two stages commit.
    world.committed_envelope(
        "coupled-distinct",
        vec![
            stage("coupled-distinct-1", created("x1", "coupled-one")),
            stage("coupled-distinct-2", created("x2", "coupled-two")),
        ],
    );
    assert!(world.live("x1") && world.live("x2"));
}

#[test]
fn a_later_stage_leaving_a_member_unnamed_refuses_missing_birth_with_no_effect() {
    let world = World::install();
    let outcome = world.envelope(
        "coupled-missing",
        vec![
            stage("coupled-missing-1", created("y1", "coupled-y1")),
            stage(
                "coupled-missing-2",
                Plan {
                    creates: keys(&["y2", "y3"]),
                    steps: vec![scoped(
                        "coupled-y",
                        vec![ScopeStep::Name("y2".into(), MemberRole::Authored)],
                    )],
                    ..Plan::default()
                },
            ),
        ],
    );
    match envelope_refusal(&outcome) {
        Some(LineageNameDenial::MissingBirth {
            target: LineageRecordTarget::Entity(_),
            scheme_id,
        }) => assert_eq!(scheme_id, MEMBER_SCHEME),
        other => panic!("the unnamed member refuses MissingBirth: {other:?} / {outcome:?}"),
    }
    for key in ["y1", "y2", "y3"] {
        assert!(!world.live(key), "{key}: zero effects across every stage");
    }
}

#[test]
fn a_later_stage_unlinking_an_earlier_stages_seam_unnamed_refuses_with_no_effect() {
    let world = World::install();
    let outcome = world.envelope(
        "coupled-missing-seam",
        vec![
            stage(
                "coupled-missing-seam-1",
                Plan {
                    creates: keys(&["s1", "s2"]),
                    seams: vec![("bare".into(), "s1".into(), "s2".into())],
                    steps: vec![
                        Step::Birth("s1".into(), birth("coupled-s1", "authored")),
                        Step::Birth("s2".into(), birth("coupled-s2", "authored")),
                    ],
                    ..Plan::default()
                },
            ),
            stage("coupled-missing-seam-2", created("s3", "coupled-s3")),
        ],
    );
    match envelope_refusal(&outcome) {
        Some(LineageNameDenial::MissingBirth {
            target: LineageRecordTarget::Relation(_),
            scheme_id,
        }) => assert_eq!(scheme_id, SEAM_SCHEME),
        other => panic!("the unnamed seam refuses MissingBirth: {other:?} / {outcome:?}"),
    }
    for key in ["s1", "s2", "s3"] {
        assert!(!world.live(key), "{key}: zero effects across every stage");
    }
}

/// A later stage deletes the record an earlier stage created and named in
/// the same envelope. The authored stages' write guard refuses the envelope
/// typed before any plan is judged (`ConflictingIntent`, naming both
/// stages), so no name can outlive the record it was declared for.
#[test]
fn a_later_stage_deleting_an_earlier_stages_named_member_refuses_with_no_effect() {
    let world = World::install();
    let outcome = world.envelope(
        "coupled-redelete",
        vec![
            stage("coupled-redelete-1", created("z1", "coupled-z1")),
            stage(
                "coupled-redelete-2",
                Plan {
                    plain_reads: keys(&["z1"]),
                    deletes: keys(&["z1"]),
                    ..Plan::default()
                },
            ),
        ],
    );
    let Envelope::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = &outcome else {
        panic!("a name for a deleted record never commits: {outcome:?}")
    };
    let kind = denial.coupled_producer_denial().map(|denial| denial.kind());
    assert!(
        matches!(
            kind,
            Some(WorthQueryOutputDemandDenialKind::ConflictingIntent {
                earlier: 0,
                later: 1,
                ..
            })
        ),
        "typed stage conflict: {outcome:?}"
    );
    assert!(!world.live("z1"), "zero effects");
}
