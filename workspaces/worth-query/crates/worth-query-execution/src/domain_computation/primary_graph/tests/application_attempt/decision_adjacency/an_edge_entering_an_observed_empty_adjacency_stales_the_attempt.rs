//! an edge entering an observed empty adjacency stales the attempt.
use super::*;

#[test]
fn an_edge_entering_an_observed_empty_adjacency_stales_the_attempt() {
    let world = installed_authorization_world(false);
    let request = live_scope();
    let actor = world
        .application
        .with_host_advancement(|phase| {
            authenticated(
                &phase,
                &(phase).borrowed_request_for_test(),
                &world,
                "alice",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_principal(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                1,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let losing = world
        .application
        .with_host_advancement(|phase| {
            link_program(
                &phase, &world, &actor, &principal, &account, &request, "open", "losing",
            )
        })
        .expect("the fixture read owns its installed host call");
    let winner = world
        .application
        .with_host_advancement(|phase| {
            link_program(
                &phase, &world, &actor, &principal, &account, &request, "open", "winner",
            )
        })
        .expect("the fixture read owns its installed host call");

    assert!(matches!(
        world.application.compare_and_commit_application(
            winner,
            idempotency(31, 31),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let outcome = world.application.compare_and_commit_application(
        losing,
        idempotency(32, 32),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert_changed_decision(outcome, "the edge entering the sealed empty adjacency");
}
