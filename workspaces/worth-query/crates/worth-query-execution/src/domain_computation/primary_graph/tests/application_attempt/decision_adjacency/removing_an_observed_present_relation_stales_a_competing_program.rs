//! removing an observed present relation stales a competing program.
use super::*;

#[test]
fn removing_an_observed_present_relation_stales_a_competing_program() {
    let world = installed_authorization_world(true);
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
    let winner = world
        .application
        .with_host_advancement(|phase| {
            unlink_program(&phase, &world, &actor, &principal, &account, &request)
        })
        .expect("the fixture read owns its installed host call");
    let loser = world
        .application
        .with_host_advancement(|phase| {
            unlink_program(&phase, &world, &actor, &principal, &account, &request)
        })
        .expect("the fixture read owns its installed host call");

    assert!(matches!(
        world.application.compare_and_commit_application(
            winner,
            idempotency(36, 36),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let outcome = world.application.compare_and_commit_application(
        loser,
        idempotency(37, 37),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert_changed_decision(outcome, "removing the retained relation");
    world
        .application
        .with_host_advancement(|phase| {
            assert_membership_absent(
                &phase, &world, &actor, &principal, &account, &request, "open",
            )
        })
        .expect("the fixture read owns its installed host call");
}
