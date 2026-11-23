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

/// One named seam `s-a` from `from` to `to`, minted as side 1.
fn one_seam(from: &str, to: &str) -> Plan {
    Plan {
        reads: keys(&[from, to]),
        seams: vec![("s-a".into(), from.into(), to.into())],
        steps: vec![Step::SeamBirth("s-a".into(), seam(1))],
        ..Plan::default()
    }
}

#[test]
fn deleting_an_endpoint_retires_its_named_seam_so_the_seam_re_mints() {
    let world = World::install();
    endpoints(&world);
    world.committed("link", one_seam("e1", "e2"));
    // `e1` is deleted with its seam still attached: no unlink, no posture.
    // The seam is retained dangling for audit, and its name ends with it.
    world.committed(
        "drop-endpoint",
        Plan {
            reads: keys(&["e1"]),
            deletes: keys(&["e1"]),
            ..Plan::default()
        },
    );
    assert!(!world.live("e1") && world.live("e2"));
    world.committed(
        "replacement",
        Plan {
            creates: keys(&["e3"]),
            steps: vec![Step::Birth("e3".into(), birth("e-3", "authored"))],
            ..Plan::default()
        },
    );
    // The same seam identity mints again: its name is not dangling live on
    // the retired relation (which would refuse `DuplicateBirth`).
    world.committed("relink", one_seam("e3", "e2"));
}

#[test]
fn unlinking_a_named_seam_retires_its_name() {
    let world = World::install();
    endpoints(&world);
    world.committed("link", one_seam("e1", "e2"));
    let live = world.sculpt("live-remint", one_seam("e2", "e1"));
    assert!(
        matches!(
            naming_refusal(&live),
            Some(LineageNameDenial::DuplicateBirth { .. })
        ),
        "the linked seam holds its name live: {live:?}"
    );
    world.committed(
        "unlink",
        Plan {
            reads: keys(&["e1", "e2"]),
            unlinks: vec![("e1".into(), "e2".into())],
            ..Plan::default()
        },
    );
    world.committed("relink", one_seam("e1", "e2"));
}
