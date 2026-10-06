//! A performed re-dispatch remains affine to its exact recovery handle.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use crate::domain_computation::application_aftermath::external_effect::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};
use crate::domain_computation::application_aftermath::recovery_handle::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBindingAxisProbe,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
use crate::domain_computation::application_aftermath::recovery_progression::{
    safe_retry_recovery_handle, WorthQueryPerformedExternalRedispatch,
    WorthQueryRecoveryEffectAuthority,
};
use crate::domain_computation::primary_graph::{
    recoverable_application_world,
    tests::{
        application_attempt::{authenticated_principal, resolved_account},
        fixture::{
            live_scope, Account, AuthorizationWorld, ExactStatusRetentionInput,
            ExactStatusRetentionOperation, IdentityExecutionSchema,
        },
    },
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitReceipt,
    WorthQueryExternalRedispatchDenial,
};

type RecoveryAdmission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

struct CompletingTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for CompletingTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::AcqRel);
        WorthQueryExternalTransportOutcome::Completed
    }
}

type RealHandle = (
    AuthorizationWorld,
    WorthQueryApplicationCommitReceipt,
    WorthQueryRecoveryHandle,
    RecoveryAdmission,
);

fn real_handle(seed: u8, label: &str) -> RealHandle {
    real_handle_under(seed, label, live_scope())
}

/// A production recovery handle and a current admission under `request`.
fn real_handle_under(seed: u8, label: &str, request: WorthQueryRequestScope) -> RealHandle {
    let (world, receipt) = recoverable_application_world(seed, label);
    let handle = world
        .application
        .mint_recovery_handle(&receipt)
        .expect("the production receipt admits a recovery handle");
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, label, &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .expect("the committed product admits the current recovery operation");
    (world, receipt, handle, admission)
}

fn authority(
    world: &AuthorizationWorld,
    handle: &WorthQueryRecoveryHandle,
    admission: &RecoveryAdmission,
) -> WorthQueryRecoveryEffectAuthority {
    world
        .application
        .admit_recovery_effect_authority(handle, admission)
        .expect("current operation truth admits recovery effect authority")
}

fn performed_redispatch(
    world: &AuthorizationWorld,
    handle: &WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryEffectAuthority,
    admission: &RecoveryAdmission,
) -> WorthQueryPerformedExternalRedispatch {
    world
        .application
        .redispatch_admitted_external_effect(handle, authority, admission)
        .expect("the public redispatch route performs the exact bound outbox")
}

#[test]
fn redispatch_performed_for_handle_a_cannot_safe_retry_handle_b() {
    let (world_a, receipt_a, handle_a, admission_a) = real_handle(211, "notify-death-a");
    let (world_b, receipt_b, handle_b, admission_b) = real_handle(212, "notify-death-b");
    assert_ne!(
        receipt_a.dispatch_outbox().unwrap().correlation(),
        receipt_b.dispatch_outbox().unwrap().correlation()
    );
    let transport_a = Arc::new(CompletingTransport(AtomicUsize::new(0)));
    world_a
        .application
        .install_external_effect_transport(transport_a.clone())
        .unwrap();
    let transport_b = Arc::new(CompletingTransport(AtomicUsize::new(0)));
    world_b
        .application
        .install_external_effect_transport(transport_b)
        .unwrap();
    let authority_a = authority(&world_a, &handle_a, &admission_a);
    let authority_b = authority(&world_b, &handle_b, &admission_b);
    let redispatch_a = performed_redispatch(&world_a, &handle_a, &authority_a, &admission_a);
    assert_eq!(transport_a.0.load(Ordering::Acquire), 1);
    assert_eq!(
        handle_a.binding().committed_product_publication(),
        receipt_a.committed_product_publication(),
        "the public route starts from the recovery handle's exact performed World terminal",
    );
    assert_eq!(
        redispatch_a.dispatch().correlation(),
        receipt_a.dispatch_outbox().unwrap().correlation(),
    );

    let denied = safe_retry_recovery_handle(handle_b, &authority_b, redispatch_a)
        .expect_err("a proof performed for handle A cannot retire handle B");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::CorrelationMismatch
    );

    let redispatch_a = performed_redispatch(&world_a, &handle_a, &authority_a, &admission_a);
    safe_retry_recovery_handle(handle_a, &authority_a, redispatch_a)
        .expect("the exact handle admits its own performed re-dispatch");
}

#[test]
fn redispatch_without_an_installed_transport_names_the_missing_transport() {
    let (world, _receipt, handle, admission) = real_handle(214, "notify-death-no-transport");
    let authority = authority(&world, &handle, &admission);
    let denied = world
        .application
        .redispatch_admitted_external_effect(&handle, &authority, &admission)
        .expect_err("no installed transport means no physical attempt");
    assert_eq!(
        denied,
        WorthQueryExternalRedispatchDenial::TransportNotInstalled
    );
    assert_eq!(
        WorthQueryRecoveryHandleDenial::from(denied).kind(),
        WorthQueryRecoveryHandleDenialKind::TransportNotInstalled
    );
}

#[test]
fn safe_retry_denies_when_the_handle_carries_no_co_committed_outbox() {
    let (world, _receipt, source, admission) = real_handle(213, "notify-death-source");
    world
        .application
        .install_external_effect_transport(Arc::new(CompletingTransport(AtomicUsize::new(0))))
        .unwrap();
    let source_authority = authority(&world, &source, &admission);
    let redispatch = performed_redispatch(&world, &source, &source_authority, &admission);
    drop(source);
    let outboxless = WorthQueryRecoveryHandle::axis_probe(
        WorthQueryRecoveryHandleBindingAxisProbe::real()
            .without_dispatch_outbox()
            .finish(),
    );
    let authority = WorthQueryRecoveryEffectAuthority::mint(
        outboxless.runtime_authority(),
        outboxless.authority_identity(),
    );
    let denied = safe_retry_recovery_handle(outboxless, &authority, redispatch)
        .expect_err("no bound outbox means no proof can match");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::CorrelationMismatch
    );
}
/// Fresh effect authority, the admitted request, and the admission's owner are
/// separate checks, and each refusal names its own cause.
#[test]
fn redispatch_names_fresh_authority_and_current_admission_failures_apart() {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let (world, _receipt, handle, admission) =
        real_handle_under(215, "notify-death-lapsed", request);
    let (other, _other_receipt, other_handle, other_admission) =
        real_handle(216, "notify-death-other");
    let own_authority = authority(&world, &handle, &admission);
    let other_authority = authority(&other, &other_handle, &other_admission);
    let redispatch = |authority, admission| {
        world
            .application
            .redispatch_admitted_external_effect(&handle, authority, admission)
            .expect_err("each presented failure refuses before transport")
    };

    assert_eq!(
        redispatch(&other_authority, &admission),
        WorthQueryExternalRedispatchDenial::FreshAuthority(
            WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied
        ),
        "another handle's authority is a fresh-authority failure"
    );
    assert_eq!(
        redispatch(&own_authority, &other_admission),
        WorthQueryExternalRedispatchDenial::ForeignAdmission,
        "another runtime's admission is not this runtime's to redispatch"
    );
    cancellation.cancel();
    let lapsed = redispatch(&own_authority, &admission);
    assert_eq!(
        lapsed,
        WorthQueryExternalRedispatchDenial::AdmissionCancelled,
        "a cancelled request is a current-admission failure"
    );
    assert_eq!(
        WorthQueryRecoveryHandleDenial::from(lapsed).kind(),
        WorthQueryRecoveryHandleDenialKind::AdmissionCancelled
    );
}
