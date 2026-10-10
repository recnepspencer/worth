//! sealed adjacency membership supplies exact unlink evidence.
use super::*;

#[test]
fn sealed_adjacency_membership_supplies_exact_unlink_evidence() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let actor = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            unlink_program(&phase, &world, &actor, &principal, &account, &request)
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::Committed(_) =
        world.application.compare_and_commit_application(
            program,
            idempotency(35, 35),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the exact relation carried by adjacency evidence must be unlinkable");
    };
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_membership_absent(
                &phase, &world, &actor, &principal, &account, &request, "open",
            )
        })
        .expect("the fixture read owns its installed host call");
}
