//! Foreign admitted-read denial with a production-derived positive twin.

use super::{
    resolve_recovery_handle, WorthQueryAdmittedIdempotencyRead, WorthQueryRecoveryEffectAuthority,
};
use crate::domain_computation::application_aftermath::recovery_handle::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBindingAxisProbe,
    WorthQueryRecoveryHandleDenialKind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
};

fn probe_handle(idempotency: WorthQueryApplicationIdempotencyBinding) -> WorthQueryRecoveryHandle {
    WorthQueryRecoveryHandle::axis_probe(
        WorthQueryRecoveryHandleBindingAxisProbe::real()
            .idempotency(idempotency)
            .finish(),
    )
}

#[test]
fn foreign_admitted_idempotency_read_denies_distinctly() {
    let binding_a = WorthQueryApplicationIdempotencyBinding::new([0xA1; 32], [0xA2; 32]);
    let binding_b = WorthQueryApplicationIdempotencyBinding::new([0xB1; 32], [0xB2; 32]);
    let handle_b = probe_handle(binding_b);
    let registry = handle_b.registry_arc();
    let authority_b = WorthQueryRecoveryEffectAuthority::mint(
        handle_b.runtime_authority(),
        handle_b.authority_identity(),
    );
    let read_a = WorthQueryAdmittedIdempotencyRead::mint(
        binding_a,
        WorthQueryApplicationIdempotencyResolution::Unseen,
    );

    let denied = resolve_recovery_handle(handle_b, &authority_b, read_a)
        .expect_err("foreign read must deny");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::ForeignIdempotencyRead
    );
    registry.assert_no_live_handles();
}

#[test]
fn matching_admitted_idempotency_read_resolves() {
    let binding = WorthQueryApplicationIdempotencyBinding::new([0xC1; 32], [0xC2; 32]);
    let handle = probe_handle(binding);
    let registry = handle.registry_arc();
    let authority = WorthQueryRecoveryEffectAuthority::mint(
        handle.runtime_authority(),
        handle.authority_identity(),
    );
    let read = WorthQueryAdmittedIdempotencyRead::mint(
        binding,
        WorthQueryApplicationIdempotencyResolution::Unseen,
    );

    let resolution =
        resolve_recovery_handle(handle, &authority, read).expect("matching read resolves");
    assert_eq!(
        resolution,
        WorthQueryApplicationIdempotencyResolution::Unseen
    );
    registry.assert_no_live_handles();
}
