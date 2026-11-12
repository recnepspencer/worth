use std::time::Duration;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyResolutionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationProjectionDenialKind;

#[test]
fn admission_from_an_equivalent_foreign_runtime_opens_no_application_door() {
    let source_world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let target_world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = source_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &source_world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = source_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            resolved_account(&host_custody, &source_world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = source_world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = source_world
        .application
        .with_host_advancement(|phase| {
            source_world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let idempotency_denial = target_world
        .application
        .with_host_advancement(|phase| {
            target_world
                .application
                .resolve_admitted_application_idempotency(&phase, &admission, idempotency(16, 16))
                .expect_err("foreign runtime admission cannot inspect target idempotency")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        idempotency_denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
    );

    let claim_denial = target_world
        .application
        .with_host_advancement(|phase| {
            target_world
                .application
                .resolve_admitted_recovery_claim_status(&phase, &admission, idempotency(16, 16))
                .unwrap_err()
        })
        .unwrap();
    assert_eq!(
        claim_denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
    );

    let read_denial = target_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            target_world
                .application
                .begin_application_read_attempt(
                    &host_custody.borrowed_request_for_test(),
                    admission,
                )
                .err()
                .expect("foreign runtime admission cannot begin a target read attempt")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        read_denial.kind(),
        WorthQueryApplicationAttemptDenialKind::ForeignApplication
    );

    let projected_admission = source_world
        .application
        .with_host_advancement(|phase| {
            source_world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let projected_denial = target_world
        .application
        .with_host_advancement(|phase| {
            target_world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &projected_admission,
                    |_, _| (),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("foreign admission cannot execute a target projection closure")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        projected_denial.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ForeignRuntime
            )
        )
    );
}

#[test]
fn cancellation_after_program_preparation_prevents_provider_commit() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
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
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "must-not-commit",
            )
        })
        .expect("the fixture read owns its installed host call");

    cancellation.cancel();
    assert!(matches!(
        world.application.compare_and_commit_application(
            program,
            idempotency(17, 17),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Cancelled
    ));
    let fresh_request = live_scope();
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &fresh_request,
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn cancelled_admission_cannot_inspect_provider_idempotency() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
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
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    cancellation.cancel();
    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .resolve_admitted_application_idempotency(&phase, &admission, idempotency(18, 18))
                .expect_err("cancelled authority cannot inspect provider idempotency")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
            ))
        )
    );
    assert_eq!(
        denial
            .authorization()
            .map(|authorization| authorization.kind()),
        Some(WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        )))
    );
}
