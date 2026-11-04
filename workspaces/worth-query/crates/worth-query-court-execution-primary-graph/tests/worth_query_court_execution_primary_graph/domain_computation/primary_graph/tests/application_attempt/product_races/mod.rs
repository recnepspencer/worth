pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

#[test]
fn unrelated_product_drift_preserves_the_prepared_attempt_and_changed_facts_stale() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let unrelated = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "unrelated",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let commit_count = || {
        (*(*(*world.application.runtime_for_test()).primary_provider_for_test()).graph_for_test())
            .with_runtime(request_execution, |runtime| {
                runtime.history().immutable_commit_count()
            })
    };
    let baseline = commit_count();

    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "first")
        })
        .expect("the fixture read owns its installed host call");
    let fresh = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "first")
        })
        .expect("the fixture read owns its installed host call");
    let losing = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "losing")
        })
        .expect("the fixture read owns its installed host call");
    let unrelated_program = world
        .application
        .with_host_advancement(|phase| {
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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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
            world.selected_product(&(phase).test_host_request_for_test())
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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(matches!(
        outcome,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    world
        .application
        .with_host_advancement(|phase| {
            assert_ne!(
                world
                    .selected_product(&(phase).test_host_request_for_test(),)
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
    let outcome = world.application.compare_and_commit_application(
        losing,
        idempotency(3, 3),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::decision_currentness::assert_changed_decision(outcome, "the sealed status changed");
    assert_eq!(commit_count(), baseline + 2);
}

#[path = "concurrent_independent_attempts_preserve_one_product_winner_and_the_ex/mod.rs"]
pub mod concurrent_independent_attempts_preserve_one_product_winner_and_the_ex;
