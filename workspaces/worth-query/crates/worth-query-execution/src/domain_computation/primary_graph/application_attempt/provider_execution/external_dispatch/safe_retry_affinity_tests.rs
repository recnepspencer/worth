//! A performed re-dispatch remains affine to its exact recovery handle.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use crate::domain_computation::application_aftermath::external_effect::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};
use crate::domain_computation::application_aftermath::recovery_handle::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBindingAxisProbe,
    WorthQueryRecoveryHandleDenialKind,
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

fn real_handle(
    seed: u8,
    label: &str,
) -> (
    AuthorizationWorld,
    WorthQueryApplicationCommitReceipt,
    WorthQueryRecoveryHandle,
    RecoveryAdmission,
) {
    let (world, receipt) = recoverable_application_world(seed, label);
    let handle = world
        .application
        .mint_recovery_handle(&receipt)
        .expect("the production receipt admits a recovery handle");
    let request = live_scope();
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
