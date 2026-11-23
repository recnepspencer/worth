//! Slice 5D: an installed operation creates an authored member and, in the
//! same commit, a piece and a seam whose discriminators reference it. The
//! names are the ones the two-commit route gives, whatever the creation
//! order, and a reference that breaks its role or occurrence law refuses
//! while authoring with zero effects.
use super::*;

/// One feature authored in one commit: member `a`, its piece `b` and the
/// seam `s` between them, `b` and `s` both told apart by `a`. `creates`
/// orders the creations; `feature` is the authored occurrence.
fn feature(creates: &[&str], feature: &str) -> Plan {
    Plan {
        creates: keys(creates),
        seams: vec![("s".into(), "a".into(), "b".into())],
        steps: vec![
            Step::Birth("a".into(), birth(feature, "authored")),
            Step::BirthCutBy {
                key: "b".into(),
                birth: birth(feature, "piece"),
                side: 1,
                of: "a".into(),
            },
            Step::SeamBirthOf {
                key: "s".into(),
                birth: birth(feature, "seam"),
                index: 1,
                of: "a".into(),
            },
        ],
        ..Plan::default()
    }
}

/// The names of `a`, `b` and the seam between them.
fn names(
    world: &World,
    token: &str,
) -> (
    WorthQueryName<Member>,
    WorthQueryName<Member>,
    WorthQueryName<Seam>,
) {
    probe::seams::name(token, "s", "a", "b");
    let observed = world.probe(token, &["a", "b"], 0);
    let seams = probe::seams::take(token);
    (
        named(&observed, "a"),
        named(&observed, "b"),
        seam_named(&seams, "s"),
    )
}

#[test]
fn an_output_references_a_name_born_by_the_same_commit() {
    let world = World::install();
    world.committed("same-commit", feature(&["a", "b"], "feature"));
    let (a, b, s) = names(&world, "same-commit-names");
    assert_ne!(a, b);

    // A rebuilt world creating the same outputs in another order, beside an
    // unrelated authored member, mints byte-identical names.
    let rebuilt = World::install();
    let mut permuted = feature(&["z", "b", "a"], "feature");
    permuted
        .steps
        .insert(0, Step::Birth("z".into(), birth("unrelated", "authored")));
    rebuilt.committed("same-commit-permuted", permuted);
    assert_eq!(names(&rebuilt, "same-commit-permuted-names"), (a, b, s));

    // The two-commit route: `a` is born first and `b`'s discriminator embeds
    // its committed name. The piece's name is the same either way.
    let staged = World::install();
    staged.committed(
        "same-commit-staged-a",
        Plan {
            creates: keys(&["a"]),
            steps: vec![Step::Birth("a".into(), birth("feature", "authored"))],
            ..Plan::default()
        },
    );
    let held = named(&staged.probe("same-commit-staged-a", &["a"], 0), "a");
    assert_eq!(held, a);
    staged.committed(
        "same-commit-staged-b",
        Plan {
            creates: keys(&["b"]),
            steps: vec![Step::Birth("b".into(), piece("feature", 1, Some(held)))],
            ..Plan::default()
        },
    );
    assert_eq!(
        named(&staged.probe("same-commit-staged-b", &["b"], 1), "b"),
        b
    );

    // Another authored occurrence of the same shape mints other names.
    let other = World::install();
    other.committed("same-commit-other", feature(&["a", "b"], "feature-2"));
    let (a2, b2, s2) = names(&other, "same-commit-other-names");
    assert!(a2 != a && b2 != b && s2 != s);
}

/// Runs `plan`, which the handler must refuse while authoring, and returns
/// the typed refusal with nothing created.
fn refused(token: &str, plan: Plan) -> sculpt::Refusal {
    let world = World::install();
    let untyped = world
        .try_sculpt(token, plan)
        .expect_err("the reference refuses while authoring");
    assert!(
        !world.live("a") && !world.live("b"),
        "a refused reference leaves no record"
    );
    sculpt::authoring_refusal(token)
        .unwrap_or_else(|| panic!("{token}: no typed output-name refusal behind {untyped}"))
}

/// Creates `a` and `b` and names them with `steps`.
fn pair(steps: Vec<Step>) -> Plan {
    Plan {
        creates: keys(&["a", "b"]),
        steps,
        ..Plan::default()
    }
}

fn cut_by(key: &str, of: &str, named: WorthQueryOutputBirth) -> Step {
    Step::BirthCutBy {
        key: key.into(),
        birth: named,
        side: 1,
        of: of.into(),
    }
}

#[test]
fn a_same_commit_reference_to_a_later_role_refuses_while_authoring() {
    let refusal = refused(
        "same-commit-later-role",
        pair(vec![
            Step::Birth("b".into(), piece("feature", 1, None)),
            cut_by("a", "b", birth("feature", "authored")),
        ]),
    );
    assert_eq!(
        refusal,
        sculpt::Refusal::NonPrecedingRole {
            role: "authored".into(),
            referenced: "piece".into(),
        }
    );
}

#[test]
fn a_same_commit_reference_to_the_same_role_refuses_while_authoring() {
    let refusal = refused(
        "same-commit-same-role",
        pair(vec![
            Step::Birth("a".into(), piece("feature", 1, None)),
            cut_by("b", "a", birth("feature", "piece")),
        ]),
    );
    assert_eq!(
        refusal,
        sculpt::Refusal::NonPrecedingRole {
            role: "piece".into(),
            referenced: "piece".into(),
        }
    );
}

#[test]
fn a_same_commit_reference_into_another_occurrence_refuses_while_authoring() {
    let refusal = refused(
        "same-commit-cross-occurrence",
        pair(vec![
            Step::Birth("a".into(), birth("feature", "authored")),
            cut_by("b", "a", birth("feature-2", "piece")),
        ]),
    );
    assert_eq!(refusal, sculpt::Refusal::CrossOccurrence);
}

#[test]
fn a_same_commit_reference_to_an_unnamed_output_refuses_while_authoring() {
    // `a` is created but not yet named when `b` asks for its name.
    let refusal = refused(
        "same-commit-unnamed",
        pair(vec![cut_by("b", "a", birth("feature", "piece"))]),
    );
    assert_eq!(refusal, sculpt::Refusal::UnnamedReference("Member".into()));
}
