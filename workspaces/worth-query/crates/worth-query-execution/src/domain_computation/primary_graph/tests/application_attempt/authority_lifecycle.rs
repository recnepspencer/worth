use std::time::{Duration, Instant};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyResolutionDenialKind,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationProjectionDenialKind,
};

#[test]
fn admission_from_an_equivalent_foreign_runtime_opens_no_application_door() {
    let source_world = installed_authorization_world(true);
    let target_world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&source_world, &request);
    let account = resolved_account(&source_world, "open", &request);
    let operation = source_world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = source_world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();

    let idempotency_denial = target_world
        .application
        .resolve_admitted_application_idempotency(&admission, idempotency(16, 16))
        .expect_err("foreign runtime admission cannot inspect target idempotency");
    assert_eq!(
        idempotency_denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
    );

    let read_denial = target_world
        .application
        .begin_application_read_attempt(admission)
        .err()
        .expect("foreign runtime admission cannot begin a target read attempt");
    assert_eq!(
        read_denial.kind(),
        WorthQueryApplicationAttemptDenialKind::ForeignApplication
    );

    let projected_admission = source_world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let projected_denial = target_world
        .invariant
        .project_admitted_operation(&projected_admission, |_, _| ())
        .err()
        .expect("foreign admission cannot execute a target projection closure");
    assert_eq!(
        projected_denial.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::ForeignRuntime
        )
    );
}

#[test]
fn cancellation_after_program_preparation_prevents_provider_commit() {
    let world = installed_authorization_world(true);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program(&world, &principal, &account, &request, "must-not-commit");

    cancellation.cancel();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(17, 17)),
        WorthQueryApplicationCommitOutcome::Cancelled
    ));
    let fresh_request = live_scope();
    let _still_open = resolved_account(&world, "open", &fresh_request);
}

#[test]
fn cancelled_admission_cannot_inspect_provider_idempotency() {
    let world = installed_authorization_world(true);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();

    cancellation.cancel();
    let denial = world
        .application
        .resolve_admitted_application_idempotency(&admission, idempotency(18, 18))
        .expect_err("cancelled authority cannot inspect provider idempotency");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization
    );
    assert_eq!(
        denial
            .authorization()
            .map(|authorization| authorization.kind()),
        Some(WorthQueryOperationAuthorizationDenialKind::Cancelled)
    );
}
