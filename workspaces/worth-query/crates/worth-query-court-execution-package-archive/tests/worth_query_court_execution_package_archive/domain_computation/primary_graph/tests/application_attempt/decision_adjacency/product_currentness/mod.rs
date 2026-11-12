use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::authenticated;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::link_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::resolved_principal;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_two_principal_authorization_world;

#[test]
fn unrelated_adjacency_growth_preserves_prepared_decision_facts() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_two_principal_authorization_world(false);
    let request = live_scope();
    let alice = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                "alice",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let bob = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                "bob",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let alice_identity = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            resolved_principal(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                1,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let bob_identity = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            resolved_principal(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                2,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let first_account = world
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
    let second_account = world
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
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    let commit_count = || {
        (*(*(*world.application.runtime_for_test()).primary_provider_for_test()).graph_for_test())
            .with_runtime(request_execution, |runtime| {
                runtime.history().immutable_commit_count()
            })
    };
    let baseline = commit_count();
    let alice_program = world
        .application
        .with_host_advancement(|phase| {
            link_program(
                &phase,
                &world,
                &alice,
                &alice_identity,
                &first_account,
                &request,
                "open",
                "alice-owner",
            )
        })
        .expect("the fixture read owns its installed host call");
    let bob_program = world
        .application
        .with_host_advancement(|phase| {
            link_program(
                &phase,
                &world,
                &bob,
                &bob_identity,
                &second_account,
                &request,
                "unrelated",
                "bob-owner",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            bob_program,
            idempotency(33, 33),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let current = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    assert_ne!(current.selected_commit(), selected.selected_commit());
    assert_eq!(commit_count(), baseline + 1);

    let outcome = world.application.compare_and_commit_application(
        alice_program,
        idempotency(34, 34),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "unchanged facts must commit on the newly admitted product basis: {outcome:?}"
    );
    assert_eq!(commit_count(), baseline + 2);
    let successor = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    assert_ne!(successor.selected_commit(), current.selected_commit());

    let readmitted = world
        .application
        .with_host_advancement(|phase| {
            link_program(
                &phase,
                &world,
                &alice,
                &alice_identity,
                &first_account,
                &request,
                "open",
                "alice-owner",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            readmitted,
            idempotency(34, 34),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::RecoveredCommitted(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
}
