//! Recovery's outer handle custody and interrupted admission lifetime.

use crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use crate::domain_computation::primary_graph::tests::application_attempt::idempotency;
use crate::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use std::time::Duration;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

use crate::domain_computation::primary_graph::tests::application_attempt::*;
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::ExactStatusRetentionInput;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation;
use worth_query_execution::facade::domain_computation::WorthQueryExternalRedispatchDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandle;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandleDenialKind;
pub type RecoveryAdmission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;
pub fn committed_handle(
    world: &AuthorizationWorld,
) -> (
    WorthQueryRecoveryHandle,
    RecoveryAdmission,
    WorthQueryCancellationSource,
) {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (principal, account) = world
        .application
        .with_host_advancement(|phase| {
            (
                authenticated_principal(&phase.test_host_request_for_test(), world, &request),
                resolved_account(&phase.test_host_request_for_test(), world, "open", &request),
            )
        })
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .unwrap();
    let (program, admission) = world
        .application
        .with_host_advancement(|phase| {
            let program = crate::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program(
                &phase.test_host_request_for_test(),
                world,
                &principal,
                &account,
                &request,
                "closed",
                crate::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth::Narrow,
            );
            let admission = world
                .selected_product(&phase.test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            (program, admission)
        })
        .unwrap();
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(181, 182),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the genuine recovery source commits: {outcome:?}");
    };
    assert!(receipt.dispatch_outbox().is_some());
    let handle = world.application.mint_recovery_handle(&receipt).unwrap();
    (handle, admission, cancellation)
}
