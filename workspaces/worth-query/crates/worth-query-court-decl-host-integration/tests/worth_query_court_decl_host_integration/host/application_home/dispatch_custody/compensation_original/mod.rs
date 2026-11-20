//! Original-only supplier declaration, honest admission and retained input across Home reopen.
mod home;
mod operation;
mod program;
mod schema;
use home::*;
use operation::{Notice, Supplier};
use std::sync::atomic::Ordering;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenialKind as MutationDenial,
    },
    primary_graph::*,
    product::WorthQueryDurableStateAdmissionDenial,
};
use worth_query_testkit_host::facade::host::{
    application_runtime_surface as identity, checkpoint_capture,
};

#[test]
fn declared_external_original_recovers_after_graceful_reopen() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let first = owners.open(execution, true, Supplier::Complete(owners.handlers.clone()));
    let receipt = emit(&first, 901);
    let coordinates = coordinates(&receipt);
    let callback = Callback::from_original(&receipt);
    let original = receipt.commit_reference().clone();
    let world = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let old = admitted(&first, 901, |phase, _, _, _| {
        first
            .runtime()
            .runtime()
            .mint_recovery_handle(phase, &receipt)
    })
    .unwrap();
    assert!(matches!(
        first
            .runtime()
            .expire_optional_correction_material(&execution, &coordinates),
        Err(WorthQueryCorrectionRetentionDenial::Material(
            QueryCorrectionMaterialDenial::MaterialPinned
        ))
    ));
    assert_eq!(owners.contacts.load(Ordering::SeqCst), 1);
    assert_eq!(
        *owners.payloads.lock().unwrap(),
        [901_u64.to_be_bytes().to_vec()]
    );
    drop(receipt);
    close(first, &execution);
    let second = owners.open(
        execution,
        false,
        Supplier::Complete(owners.handlers.clone()),
    );
    let recovered = recover(&second, 901).unwrap();
    assert_eq!(recovered.binding().commit_reference(), &original);
    let recovered_world = recovered
        .binding()
        .committed_product_publication()
        .composite_commit();
    assert_eq!(
        recovered_world.owner_identity().durable(),
        world.owner_identity().durable()
    );
    assert_eq!(recovered_world.ordinal(), world.ordinal());
    assert_ne!(recovered_world.owner_identity(), world.owner_identity());
    let identity = recovered.opaque_wire_identity();
    let refused = admitted(&second, 901, |phase, admission, _, _| {
        second
            .runtime()
            .runtime()
            .settle_recorded_recovery_completion(phase, recovered, admission)
    })
    .unwrap_err();
    assert_eq!(
        refused.kind(),
        WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture
    );
    let (_, retained) = refused.into_parts();
    let retained = retained.expect("refusal must return the original live claim");
    assert_eq!(retained.opaque_wire_identity(), identity);
    callback.complete(&second);
    admitted(&second, 901, |phase, admission, _, _| {
        second
            .runtime()
            .runtime()
            .settle_recorded_recovery_completion(phase, retained, admission)
    })
    .unwrap();
    assert_eq!(owners.contacts.load(Ordering::SeqCst), 1);
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    drop(old);
    close(second, &execution);
}

#[test]
fn missing_original_dispatch_custody_requirement_refuses_before_handler() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let first = owners.open(
        execution,
        true,
        Supplier::MissingDispatch(owners.handlers.clone()),
    );
    let before = first.runtime().runtime().current_world();
    let (_, scope) = super::super::pending_support::scope();
    let runtime = first.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let denied = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(Notice(902))
        .without_source()
        .idempotency(&902)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    assert!(matches!(
        denied.kind(),
        MutationDenial::DurableStateAdmission(
            WorthQueryDurableStateAdmissionDenial::UnsupportedCorrectionCompensation
        )
    ));
    assert_eq!(runtime.runtime().current_world(), before);
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 0);
    assert_eq!(owners.contacts.load(Ordering::SeqCst), 0);
    drop((denied, principal));
    close(first, &execution);
    let second = owners.open(
        execution,
        false,
        Supplier::Complete(owners.handlers.clone()),
    );
    admitted(&second, 902, |phase, admission, binding, _| {
        assert_eq!(
            second
                .runtime()
                .runtime()
                .resolve_admitted_recovery_claim_status(phase, admission, binding)
                .unwrap(),
            None
        );
    });
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 0);
    assert_eq!(owners.contacts.load(Ordering::SeqCst), 0);
    close(second, &execution);
}

#[test]
fn recovery_requires_its_retained_original_input() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let first = owners.open(execution, true, Supplier::Complete(owners.handlers.clone()));
    let receipt = emit(&first, 903);
    let coordinates = coordinates(&receipt);
    // Separate unclaimed original: no mandatory recovery pin is discarded to expire it.
    first
        .runtime()
        .expire_optional_correction_material(&execution, &coordinates)
        .unwrap();
    drop(receipt);
    close(first, &execution);
    let second = owners.open(
        execution,
        false,
        Supplier::Complete(owners.handlers.clone()),
    );
    let before = second.runtime().runtime().current_world();
    let denied = recover(&second, 903).unwrap_err();
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::RetainedMaterialUnavailable
    );
    assert!(matches!(
        denied.source(),
        Some(WorthQueryRecoverySourceDenial::Material(
            QueryCorrectionMaterialDenial::RetainedMaterialExpired
        ))
    ));
    assert_eq!(second.runtime().runtime().current_world(), before);
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    assert_eq!(owners.contacts.load(Ordering::SeqCst), 1);
    close(second, &execution);
}
