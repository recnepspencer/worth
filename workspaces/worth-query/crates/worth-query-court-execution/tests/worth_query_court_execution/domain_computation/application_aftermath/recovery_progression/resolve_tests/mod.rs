//! Foreign admitted-read denial with a production-derived positive twin.

use worth_query_execution::facade::domain_computation::resolve_recovery_handle;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedIdempotencyRead;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryEffectAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandle;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::application_aftermath::recovery_handle::binding::WorthQueryRecoveryHandleBindingAxisProbe;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandleDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyResolution;

pub fn probe_handle(
    idempotency: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryRecoveryHandle {
    WorthQueryRecoveryHandle::axis_probe_for_test(
        WorthQueryRecoveryHandleBindingAxisProbe::real()
            .idempotency(idempotency)
            .finish(),
    )
}

#[test]
fn foreign_admitted_idempotency_read_denies_distinctly() {
    let binding_a = WorthQueryApplicationIdempotencyBinding::new_for_test([0xA1; 32], [0xA2; 32]);
    let binding_b = WorthQueryApplicationIdempotencyBinding::new_for_test([0xB1; 32], [0xB2; 32]);
    let handle_b = probe_handle(binding_b);
    let registry = handle_b.registry_arc_for_test();
    let authority_b = WorthQueryRecoveryEffectAuthority::mint_for_test(
        handle_b.runtime_authority_for_test(),
        handle_b.authority_identity_for_test(),
        &handle_b,
    );
    let read_a = WorthQueryAdmittedIdempotencyRead::mint_for_test(
        binding_a,
        WorthQueryApplicationIdempotencyResolution::Unseen,
    );

    let denied = super::local_transition::in_phase(|phase| {
        resolve_recovery_handle(&phase, handle_b, &authority_b, read_a)
    })
    .expect_err("foreign read must deny");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::ForeignIdempotencyRead
    );
    registry.assert_no_live_handles_for_test();
}

#[test]
fn matching_admitted_idempotency_read_resolves() {
    let binding = WorthQueryApplicationIdempotencyBinding::new_for_test([0xC1; 32], [0xC2; 32]);
    let handle = probe_handle(binding);
    let registry = handle.registry_arc_for_test();
    let authority = WorthQueryRecoveryEffectAuthority::mint_for_test(
        handle.runtime_authority_for_test(),
        handle.authority_identity_for_test(),
        &handle,
    );
    let read = WorthQueryAdmittedIdempotencyRead::mint_for_test(
        binding,
        WorthQueryApplicationIdempotencyResolution::Unseen,
    );

    let resolution = super::local_transition::in_phase(|phase| {
        resolve_recovery_handle(&phase, handle, &authority, read)
    })
    .expect("matching read resolves");
    assert!(matches!(
        resolution,
        WorthQueryApplicationIdempotencyResolution::Unseen
    ));
    registry.assert_no_live_handles_for_test();
}
