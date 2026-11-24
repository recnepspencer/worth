//! Completeness of every installed name scheme (`require_all_births_named`):
//! a commit that creates two records of a schemed kind and names only one
//! refuses `MissingBirth` for the unnamed one, with zero effects, for
//! entities and relations alike. The guard is not opt-in: every scheme an
//! application installs is complete.
use super::outputs::sculpt::{Plan, ScopeStep, Step};
use super::outputs::{birth, keys, naming_refusal, World};
use super::*;
use worth_query_host::facade::application_naming::{
    LineageNameDenial, LineageRecordTarget, OccurrencePath,
};

#[test]
fn a_commit_naming_one_of_two_created_members_refuses_without_effects() {
    let world = World::install();
    let outcome = world.sculpt(
        "complete-members",
        Plan {
            creates: keys(&["named", "bare"]),
            steps: vec![Step::Scoped(
                OccurrencePath::root("complete").unwrap(),
                vec![ScopeStep::Name("named".into(), MemberRole::Authored)],
            )],
            ..Plan::default()
        },
    );
    match naming_refusal(&outcome) {
        Some(LineageNameDenial::MissingBirth {
            target: LineageRecordTarget::Entity(_),
            scheme_id,
        }) => assert_eq!(scheme_id, MEMBER_SCHEME),
        other => panic!("the unnamed member refuses MissingBirth: {other:?} / {outcome:?}"),
    }
    assert!(!world.live("named") && !world.live("bare"), "zero effects");
}

#[test]
fn a_commit_naming_one_of_two_created_seams_refuses_without_effects() {
    let world = World::install();
    let outcome = world.sculpt(
        "complete-seams",
        Plan {
            creates: keys(&["e1", "e2"]),
            seams: vec![
                ("named".into(), "e1".into(), "e2".into()),
                ("bare".into(), "e2".into(), "e1".into()),
            ],
            steps: vec![
                Step::Birth("e1".into(), birth("complete-e1", "authored")),
                Step::Birth("e2".into(), birth("complete-e2", "authored")),
                Step::SeamBirth("named".into(), birth("complete-seam", "seam_piece")),
            ],
            ..Plan::default()
        },
    );
    match naming_refusal(&outcome) {
        Some(LineageNameDenial::MissingBirth {
            target: LineageRecordTarget::Relation(_),
            scheme_id,
        }) => assert_eq!(scheme_id, SEAM_SCHEME),
        other => panic!("the unnamed seam refuses MissingBirth: {other:?} / {outcome:?}"),
    }
    assert!(
        !world.live("e1") && !world.live("e2"),
        "the refused commit leaves zero effects, named births included"
    );
}
