//! Real Query publication cancellation after native transaction staging.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind;
use std::time::Duration;
use std::time::Instant;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[test]
fn omitted_work_budget_keeps_live_cancellation_through_native_validation() {
    let world = installed_authorization_world(true);
    let installed = world
        .application
        .installed_schema()
        .installed_operation(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountOperation::reference())
        .unwrap();
    assert_eq!(installed.contracts().execution_strategy().unwrap().envelope()
        .optional_scale_ceiling(worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleAxis::WorkItems), None);
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
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "after-staging",
            )
        })
        .expect("the fixture read owns its installed host call");
    // This observer cancels the actual admitted request after push_batch succeeds.
    // It neither injects a failure nor bypasses the provider's validator.
    world.faults.cancel_next_staged_validation(source);
    let outcome = world.application.compare_and_commit_application(
        rejected,
        idempotency(94, 94),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("post-staging cancellation must deny the real publication: {outcome:?}");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        denial.invariant_execution_failure().unwrap().kind(),
        WorthQueryInvariantExecutionDenialKind::RequestInterrupted(
            worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled
        )
    );
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
                "after-staging",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(
        matches!(
            world.application.compare_and_commit_application(
                retry,
                idempotency(94, 94),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation
            ),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "cancelled attempt must not consume its key"
    );
    let landed = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "after-staging",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(landed.entity_id_for_test(), account.entity_id_for_test());
}
