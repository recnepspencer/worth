use super::*;

#[test]
fn rejection_before_transaction_publishes_no_emit_causality() {
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
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "rejected",
                Some("must-not-publish"),
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.reject_next_commit_before_transaction();
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(32, 32),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        !matches!(
            outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
                | WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
        ),
        "rejected transaction claimed commit: {outcome:?}"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
}
