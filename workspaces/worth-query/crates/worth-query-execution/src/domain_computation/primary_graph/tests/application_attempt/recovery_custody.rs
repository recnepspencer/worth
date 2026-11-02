//! Recovery's outer handle custody and interrupted admission lifetime.
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenialKind,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, AuthorizationWorld, ExactStatusRetentionInput, ExactStatusRetentionOperation,
    IdentityExecutionSchema,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryExternalRedispatchDenial,
    WorthQueryOperationAuthorizationDenialKind,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

pub(super) type RecoveryAdmission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

pub(super) fn committed_handle(
    world: &AuthorizationWorld,
) -> (
    WorthQueryRecoveryHandle,
    RecoveryAdmission,
    WorthQueryCancellationSource,
) {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    let (principal, account) = world
        .application
        .with_host_advancement(|phase| {
            (
                authenticated_principal(&phase.test_host_request(), world, &request),
                resolved_account(&phase.test_host_request(), world, "open", &request),
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
            let program = super::preimage_evidence::retained_status_program(
                &phase.test_host_request(),
                world,
                &principal,
                &account,
                &request,
                "closed",
                super::preimage_evidence::RetentionMutationBreadth::Narrow,
            );
            let admission = world
                .selected_product(&phase.test_host_request())
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
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the genuine recovery source commits: {outcome:?}");
    };
    assert!(receipt.dispatch_outbox().is_some());
    let handle = world.application.mint_recovery_handle(&receipt).unwrap();
    (handle, admission, cancellation)
}

#[test]
fn expired_redispatch_handle_precedes_cancelled_admission() {
    let world = installed_authorization_world(true);
    let now = std::time::SystemTime::now();
    world.authorization_time.hold(now);
    let (handle, admission, cancellation) = committed_handle(&world);
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .expect("fresh handle and genuine admission issue authority");
    cancellation.cancel();
    assert_eq!(
        admission.validate_current_authority().unwrap_err().kind(),
        WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        ))
    );
    world
        .authorization_time
        .hold(now + Duration::from_secs(3_601));
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let Err(denial) = world
                .application
                .redispatch_admitted_external_effect(&phase, &handle, &authority, &admission)
            else {
                panic!("the expired handle refuses redispatch");
            };
            denial
        })
        .unwrap();
    assert!(
        matches!(
            denial,
            WorthQueryExternalRedispatchDenial::FreshAuthority(
                WorthQueryRecoveryHandleDenialKind::Expired
            )
        ),
        "outer handle custody must win: {denial:?}"
    );
}
