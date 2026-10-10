use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account, WorthQueryApplicationCommitOutcome,
};

#[path = "product_races/concurrent_independent_attempts_preserve_one_product_winner_and_the_ex.rs"]
mod concurrent_independent_attempts_preserve_one_product_winner_and_the_ex;
#[test]
fn unrelated_product_drift_preserves_the_prepared_attempt_and_changed_facts_stale() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
    let unrelated = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "unrelated", &request)
        })
        .expect("the fixture read owns its installed host call");
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let commit_count = || {
        world
            .application
            .primary_provider
            .graph
            .with_runtime(request_execution, |runtime| {
                runtime.history().immutable_commit_count()
            })
    };
    let baseline = commit_count();

    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "first")
        })
        .expect("the fixture read owns its installed host call");
    let fresh = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "first")
        })
        .expect("the fixture read owns its installed host call");
    let losing = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "losing")
        })
        .expect("the fixture read owns its installed host call");
    let unrelated_program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &unrelated,
                &request,
                "unrelated-after",
            )
        })
        .expect("the fixture read owns its installed host call");

    let unrelated_outcome = world.application.compare_and_commit_application(
        unrelated_program,
        idempotency(1, 1),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(
            unrelated_outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "unexpected unrelated outcome: {unrelated_outcome:?}"
    );
    let current = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    assert_ne!(
        selected.product().selected_commit(),
        current.product().selected_commit()
    );
    assert_eq!(commit_count(), baseline + 1);
    let outcome = world.application.compare_and_commit_application(
        first,
        idempotency(2, 2),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(matches!(
        outcome,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_ne!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit(),
                current.product().selected_commit()
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(commit_count(), baseline + 2);
    assert!(matches!(
        world.application.compare_and_commit_application(
            fresh,
            idempotency(2, 2),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
    let outcome = world.application.compare_and_commit_application(
        losing,
        idempotency(3, 3),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    super::assert_changed_decision(outcome, "the sealed status changed");
    assert_eq!(commit_count(), baseline + 2);
}
