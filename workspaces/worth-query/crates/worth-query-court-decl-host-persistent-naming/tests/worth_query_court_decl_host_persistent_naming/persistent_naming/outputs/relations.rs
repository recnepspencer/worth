//! Installed operations name the relations they link. Parallel same-role
//! seams of one occurrence are told apart by discriminator alone: a name
//! never includes its endpoints (and Relational holds one relation of a kind
//! per endpoint pair, so parallel seams run between different pairs).
use super::*;

fn seam(index: u64) -> worth_query_host::facade::application_naming::WorthQueryOutputBirth {
    birth("seam-feature", "seam").with_discriminator(&SeamSide { index })
}

fn endpoints(world: &World) {
    world.committed(
        "endpoints",
        Plan {
            creates: keys(&["e1", "e2"]),
            steps: vec![
                Step::Birth("e1".into(), birth("e-1", "authored")),
                Step::Birth("e2".into(), birth("e-2", "authored")),
            ],
            ..Plan::default()
        },
    );
}

fn parallel(first: u64, second: u64) -> Plan {
    Plan {
        reads: keys(&["e1", "e2"]),
        seams: vec![
            ("s-a".into(), "e1".into(), "e2".into()),
            ("s-b".into(), "e2".into(), "e1".into()),
        ],
        steps: vec![
            Step::SeamBirth("s-a".into(), seam(first)),
            Step::SeamBirth("s-b".into(), seam(second)),
        ],
        ..Plan::default()
    }
}

#[test]
fn parallel_same_role_seams_with_distinct_discriminators_are_born_named() {
    let world = World::install();
    endpoints(&world);
    world.committed("parallel", parallel(1, 2));
}

#[test]
fn parallel_same_role_seams_with_one_discriminator_collide() {
    let world = World::install();
    endpoints(&world);
    let outcome = world.sculpt("collide", parallel(3, 3));
    assert!(
        matches!(
            naming_refusal(&outcome),
            Some(LineageNameDenial::DuplicateBirth { .. })
        ),
        "endpoints never distinguish same-role seams: {outcome:?}"
    );
    // The same plan with distinct sides still commits: nothing partial landed.
    world.committed("after-collide", parallel(3, 4));
}

#[test]
fn an_unnamed_seam_of_a_schemed_relation_refuses_missing_birth() {
    let world = World::install();
    endpoints(&world);
    let outcome = world.sculpt(
        "unnamed-seam",
        Plan {
            reads: keys(&["e1", "e2"]),
            seams: vec![("bare".into(), "e1".into(), "e2".into())],
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
}
