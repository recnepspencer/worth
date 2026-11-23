//! Contradictory or missing output names refuse typed with zero effects.
use super::*;
use worth_query_host::facade::application_naming::{
    LineagePostureKind, WorthQueryOutputNameDenial,
};

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
    let refused = authoring_refusal(
        &world,
        "unschemed",
        Plan {
            creates: keys(&["owned"]),
            steps: vec![
                Step::Birth("owned".into(), birth("owned", "authored")),
                Step::NameOwnerLink("owned".into(), birth("owner-link", "seam")),
            ],
            ..Plan::default()
        },
    );
    assert_eq!(refused, sculpt::Refusal::Unschemed("ModelMember".into()));
    assert!(!world.live("owned"));
}

/// Runs `plan`, which the handler must refuse while authoring, and returns
/// the typed refusal its `persistent_name_denial()` carried.
fn authoring_refusal(world: &World, token: &str, plan: Plan) -> sculpt::Refusal {
    let untyped = world
        .try_sculpt(token, plan)
        .expect_err("the output name refuses while authoring");
    sculpt::authoring_refusal(token)
        .unwrap_or_else(|| panic!("{token}: no typed output-name refusal behind {untyped}"))
}

#[test]
fn a_split_piece_without_a_discriminator_refuses_while_authoring() {
    let world = World::install();
    seeded(&world, "undiscriminated", &["u"]);
    let refused = authoring_refusal(
        &world,
        "undiscriminated-split",
        Plan {
            reads: keys(&["u"]),
            deletes: keys(&["u"]),
            creates: keys(&["u1", "u2"]),
            steps: vec![Step::Split(
                read("u"),
                vec![
                    ("u1".into(), piece("cut", 1, None)),
                    ("u2".into(), birth("cut", "piece")),
                ],
            )],
            ..Plan::default()
        },
    );
    assert_eq!(
        refused,
        sculpt::Refusal::MissingSplitDiscriminator {
            role: "piece".into()
        }
    );
    assert!(world.live("u") && !world.live("u1") && !world.live("u2"));
}

#[test]
fn naming_a_record_the_candidate_did_not_create_refuses_while_authoring() {
    let world = World::install();
    seeded(&world, "existing", &["x"]);
    let refused = authoring_refusal(
        &world,
        "existing-birth",
        Plan {
            reads: keys(&["x"]),
            steps: vec![Step::Birth("x".into(), birth("again", "authored"))],
            ..Plan::default()
        },
    );
    assert_eq!(refused, sculpt::Refusal::NotCreated);
}

#[test]
fn a_posture_on_a_name_that_ended_refuses_before_staging() {
    let world = World::install();
    seeded(&world, "ended", &["gone", "stays"]);
    let gone = named(&world.probe("ended", &["gone"], 0), "gone");
    world.committed(
        "ended-delete",
        Plan {
            reads: keys(&["gone"]),
            deletes: keys(&["gone"]),
            ..Plan::default()
        },
    );
    // A court-held name outlives its record: retiring it again designates no
    // live record at the commit basis.
    let outcome = world.sculpt(
        "ended-retire",
        Plan {
            reads: keys(&["stays"]),
            deletes: keys(&["stays"]),
            steps: vec![Step::Retire(Held::Name(gone))],
            ..Plan::default()
        },
    );
    assert!(
        matches!(
            output_refusal(&outcome),
            Some(WorthQueryOutputNameDenial::HeldNameNotLive { .. })
        ),
        "{outcome:?}"
    );
    assert!(
        world.live("stays"),
        "the refused retirement deleted nothing"
    );
}
