//! pretransaction commit failure is proved aborted and applies nothing.
use super::*;

#[test]
fn pretransaction_commit_failure_is_proved_aborted_and_applies_nothing() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "atomic-replacement",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.reject_next_commit_before_transaction();
    assert!(matches!(
        world.application.compare_and_commit_application(
            rejected,
            idempotency(20, 20),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Aborted
    ));
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &live_scope())
        })
        .expect("the fixture read owns its installed host call");

    let retry = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "atomic-replacement",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            retry,
            idempotency(20, 20),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}
