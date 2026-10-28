//! Both prepared relation and entity sets need independent sensitivity.
use super::*;

#[test]
fn third_hop_only_edge_replacement_requires_the_fresh_custom_violation_and_its_lawful_twin() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let world = world::install(world::Graph::three_hops(true, vec![4]));
    let request = super::super::super::fixture::live_scope();
    let hostile = program::replace_next(&world, &request, 4);
    let before = world.checkpoint();
    let edges = world.edges();
    world.observation.arm();
    exact_violation(
        commit(&world, &request, hostile, 212),
        rule::Refusal {
            root: 0,
            blocked: 4,
            distance: 3,
        },
    );
    assert_deep_callbacks(&world);
    assert_eq!(world.checkpoint(), before);
    assert_eq!(world.edges(), edges);

    let lawful = program::replace_next(&world, &request, 5);
    world.observation.arm();
    assert!(matches!(
        commit(&world, &request, lawful, 213),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_deep_callbacks(&world);
    assert_eq!(world.edges(), [(0, 1), (1, 2), (2, 5)]);
}

#[test]
fn root_activation_requires_deep_entity_admission_and_its_lawful_twin() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    for blocked in [true, false] {
        let world = world::install(world::Graph::three_hops(
            false,
            if blocked { vec![3] } else { vec![] },
        ));
        let request = super::super::super::fixture::live_scope();
        let program = program::set_active(&world, &request, true);
        let before = world.checkpoint();
        let edges = world.edges();
        world.observation.arm();
        let outcome = commit(&world, &request, program, 214);
        if blocked {
            exact_violation(
                outcome,
                rule::Refusal {
                    root: 0,
                    blocked: 3,
                    distance: 3,
                },
            );
            assert_eq!(world.checkpoint(), before);
        } else {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
            assert_ne!(world.checkpoint(), before);
        }
        assert_deep_callbacks(&world);
        assert_eq!(world.edges(), edges);
    }
}
