//! Real request interruption at native preparation, before any publication.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_execution::ExecutionAllocationPolicy;
use std::time::Duration;
use std::time::Instant;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[test]
fn native_preparation_cancellation_leaves_head_and_key_available_for_fresh_retry() {
    let world = installed_authorization_world(true);
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(30)),
        source.token(),
    );
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
    let head = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "native-prepared",
            )
        })
        .expect("the fixture read owns its installed host call");
    // Cancel the actual retained control at the preparation port; the next
    // request-paid index charge detects it before publication.
    world.faults.cancel_next_native_preparation(source);
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(95, 95),
        ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Cancelled),
        "the request-paid seam retains cancellation: {outcome:?}"
    );
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                &head,
                world
                    .selected_product(&(phase).test_host_request_for_test(),)
                    .product()
                    .selected_commit()
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        (*world.application.primary_provider_for_test())
            .published_application_commit_count_for_test(),
        0
    );
    let fresh = live_scope();
    let unchanged = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &fresh,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(unchanged.entity_id_for_test(), account.entity_id_for_test());
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &fresh)
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &unchanged,
                &fresh,
                "native-prepared",
            )
        })
        .expect("the fixture read owns its installed host call");
    let outcome = world.application.compare_and_commit_application(
        retry,
        idempotency(95, 95),
        ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the cancelled preparation must not consume its idempotency key: {outcome:?}"
    );
    assert_eq!(
        (*world.application.primary_provider_for_test())
            .published_application_commit_count_for_test(),
        1
    );
    let landed = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "native-prepared",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(landed.entity_id_for_test(), account.entity_id_for_test());
}
