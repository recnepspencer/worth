//! Contradictory or missing output names refuse typed with zero effects.
use super::*;
use worth_query_host::facade::application_naming::LineagePostureKind;

fn seeded(world: &World, token: &str, members: &[&str]) {
    world.committed(
        token,
        Plan {
            creates: keys(members),
            steps: members
                .iter()
                .map(|key| Step::Birth((*key).to_owned(), birth(&format!("occ-{key}"), "authored")))
                .collect(),
            ..Plan::default()
        },
    );
}

#[test]
fn a_split_whose_parent_survives_refuses_without_effects() {
    let world = World::install();
    seeded(&world, "unretired", &["p"]);
    let outcome = world.sculpt(
        "unretired-split",
        Plan {
            reads: keys(&["p"]),
            creates: keys(&["c1"]),
            steps: vec![Step::Split(
                read("p"),
                vec![("c1".into(), piece("cut", 1, None))],
            )],
            ..Plan::default()
        },
    );
    assert!(
        matches!(
            naming_refusal(&outcome),
            Some(LineageNameDenial::LineagePostureMismatch {
                posture: LineagePostureKind::Split,
                ..
            })
        ),
        "{outcome:?}"
    );
    assert!(!world.live("c1") && world.live("p"));
}

#[test]
fn a_merge_keeper_outside_its_parents_refuses_without_effects() {
    let world = World::install();
    seeded(&world, "keeper", &["x", "y", "z"]);
    let outcome = world.sculpt(
        "keeper-merge",
        Plan {
            reads: keys(&["x", "y", "z"]),
            deletes: keys(&["y"]),
            steps: vec![Step::Merge(
                vec![read("x"), read("y")],
                Keeper::InPlace(read("z")),
            )],
            ..Plan::default()
        },
    );
    assert!(
        matches!(
            naming_refusal(&outcome),
            Some(LineageNameDenial::InvalidMergeKeeper { .. })
        ),
        "{outcome:?}"
    );
    assert!(world.live("y"), "the refused merge deleted nothing");
}

#[test]
fn colliding_sibling_discriminators_refuse_naming_both() {
    let world = World::install();
    seeded(&world, "siblings", &["s"]);
    let outcome = world.sculpt(
        "siblings-split",
        Plan {
            reads: keys(&["s"]),
            deletes: keys(&["s"]),
            creates: keys(&["s1", "s2"]),
            steps: vec![Step::Split(
                read("s"),
                vec![
                    ("s1".into(), piece("cut", 1, None)),
                    ("s2".into(), piece("cut", 1, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    match naming_refusal(&outcome) {
        Some(LineageNameDenial::DuplicateBirth { first, second, .. }) => assert_ne!(first, second),
        other => panic!("colliding siblings refuse DuplicateBirth: {other:?} / {outcome:?}"),
    }
    assert!(world.live("s") && !world.live("s1") && !world.live("s2"));
}

#[test]
fn a_schemed_output_without_a_birth_still_refuses_missing_birth() {
    let world = World::install();
    let outcome = world.sculpt(
        "unnamed",
        Plan {
            creates: keys(&["bare"]),
            ..Plan::default()
        },
    );
    assert!(
        matches!(
            naming_refusal(&outcome),
            Some(LineageNameDenial::MissingBirth { .. })
        ),
        "{outcome:?}"
    );
    assert!(!world.live("bare"));
}

#[test]
fn naming_an_unschemed_output_refuses_before_any_effect() {
    let world = World::install();
    let refused = world
        .try_sculpt(
            "unschemed",
            Plan {
                creates: keys(&["owned"]),
                steps: vec![
                    Step::Birth("owned".into(), birth("owned", "authored")),
                    Step::NameOwnerLink("owned".into(), birth("owner-link", "seam")),
                ],
                ..Plan::default()
            },
        )
        .expect_err("naming an unschemed relation refuses while authoring");
    assert!(
        refused.contains(
            "PersistentName(UnschemedOutput { subject: Relation, subject_name: \"ModelMember\" })"
        ),
        "{refused}"
    );
    assert!(!world.live("owned"));
}
