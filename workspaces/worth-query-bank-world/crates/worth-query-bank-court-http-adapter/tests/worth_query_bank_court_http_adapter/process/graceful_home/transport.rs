//! Original Bank wire meaning and independent consequences survive actual Host close/reopen.
use super::{fixture::*, rejection::Reject};
use std::sync::atomic::Ordering;
use std::time::Duration;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
#[test]
fn completed_bank_rail_custody_reopens_without_another_contact_or_handler() {
    let world = World::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = checkpoint_capture::ExecutionRequest::leased(&lease);
    let first = world.open(execution, true);
    let input = notice(8701);
    let receipt = first.emit(input.clone(), 8701);
    assert_eq!(
        receipt.external_dispatch().unwrap().posture().kind(),
        WorthQueryExternalDispatchPostureKind::Completed
    );
    let original = receipt.commit_reference().clone();
    let correlation = first.transport.attempts()[0].clone();
    assert_eq!(first.transport.completed_effect_count(), 1);
    let consequence = first.transport.completed_notice(&correlation).unwrap();
    assert_eq!(
        (
            consequence.estate(),
            consequence.notice(),
            consequence.subject()
        ),
        (
            input.estate().get(),
            input.notice().get(),
            input.subject().get()
        )
    );
    let old = first
        .home
        .runtime()
        .runtime()
        .with_host_advancement(|phase| {
            first
                .home
                .runtime()
                .runtime()
                .mint_recovery_handle(&phase, &receipt)
        })
        .unwrap()
        .unwrap();
    drop(receipt);
    first.close(&execution);
    let second = world.open(execution, false);
    let handle = second.recover(input.clone(), 8701);
    assert_eq!(handle.binding().commit_reference(), &original);
    drop(old);
    second
        .admitted(input, 8701, |phase, admission, _, _| {
            second
                .home
                .runtime()
                .runtime()
                .settle_recorded_recovery_completion(phase, handle, admission)
        })
        .unwrap();
    assert!(second.transport.production_dispatches().is_empty());
    assert_eq!(second.transport.completed_effect_count(), 1);
    assert_eq!(world.handlers.load(Ordering::SeqCst), 1);
    second.close(&execution);
    world.finish();
}
#[test]
fn lost_bank_response_safe_retries_the_same_original_after_graceful_reopen() {
    let world = World::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = checkpoint_capture::ExecutionRequest::leased(&lease);
    let first = world.open(execution, true);
    first
        .transport
        .under(FaultScript::CommitThenLoseResponse, Duration::from_secs(5));
    let input = notice(8711);
    let receipt = first.emit(input.clone(), 8711);
    assert_eq!(
        receipt.external_dispatch().unwrap().posture().kind(),
        WorthQueryExternalDispatchPostureKind::Unresolved
    );
    let original = receipt.commit_reference().clone();
    let outbound = first.transport.production_dispatches()[0].clone();
    assert_eq!(
        first.transport.ledger_status(&outbound.correlation),
        LedgerStatus::Completed
    );
    assert_eq!(first.transport.completed_effect_count(), 1);
    drop(receipt);
    first.close(&execution);
    let second = world.open(execution, false);
    let handle = second.recover(input.clone(), 8711);
    assert_eq!(handle.binding().commit_reference(), &original);
    second
        .transport
        .under(FaultScript::Succeed, Duration::from_secs(5));
    second.admitted(input, 8711, |phase, admission, _, _| {
        let runtime = second.home.runtime().runtime();
        let authority = runtime
            .admit_recovery_effect_authority(&handle, admission)
            .unwrap();
        let performed = runtime
            .redispatch_admitted_external_effect(phase, &handle, &authority, admission)
            .unwrap();
        let completed = safe_retry_recovery_handle(phase, handle, &authority, performed).unwrap();
        assert_eq!(
            completed.outbox_durability(),
            WorthQueryDispatchOutboxDurabilityPosture::DurableHome
        );
    });
    let repeated = second.transport.production_dispatches();
    assert_eq!(repeated.len(), 1);
    assert_eq!(repeated[0].correlation, outbound.correlation);
    assert_eq!(repeated[0].payload, outbound.payload);
    assert_eq!(second.transport.admission_count(), 1);
    assert_eq!(second.transport.completed_effect_count(), 1);
    assert_eq!(world.handlers.load(Ordering::SeqCst), 1);
    second.close(&execution);
    world.finish();
}
#[test]
fn duplicate_bank_acknowledgement_keeps_its_durable_pending_identity_after_reopen() {
    let world = World::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = checkpoint_capture::ExecutionRequest::leased(&lease);
    let first = world.open(execution, true);
    first.transport.under(
        FaultScript::DuplicateAcknowledgement,
        Duration::from_secs(5),
    );
    let input = notice(8721);
    let receipt = first.emit(input.clone(), 8721);
    assert_eq!(
        receipt.external_dispatch().unwrap().posture().kind(),
        WorthQueryExternalDispatchPostureKind::Unresolved
    );
    assert_eq!(
        receipt
            .external_dispatch()
            .unwrap()
            .posture()
            .classification()
            .unwrap()
            .fault(),
        ExternalRailTransportFault::DuplicatedAcknowledgement
    );
    let remote = first.transport.attempts()[0].clone();
    assert_eq!(
        first.transport.ledger_status(&remote),
        LedgerStatus::Acknowledged
    );
    let correlation = *receipt.dispatch_outbox().unwrap().correlation().bytes();
    let recorded = first
        .home
        .runtime()
        .runtime()
        .durable_dispatch_custody(&correlation)
        .unwrap();
    let original = receipt.commit_reference().clone();
    drop(receipt);
    first.close(&execution);
    let second = world.open(execution, false);
    assert_eq!(
        second
            .home
            .runtime()
            .runtime()
            .durable_dispatch_custody(&correlation)
            .unwrap(),
        recorded
    );
    second.admitted(input, 8721, |phase, admission, binding, _| {
        let read = second
            .home
            .runtime()
            .runtime()
            .resolve_admitted_application_idempotency(phase, admission, binding)
            .unwrap();
        let WorthQueryApplicationIdempotencyResolution::RecoveredCommitted(receipt) =
            read.into_resolution()
        else {
            panic!("original keyed occurrence must be readmitted")
        };
        assert_eq!(receipt.commit_reference(), &original);
    });
    assert!(second.transport.production_dispatches().is_empty());
    assert_eq!(second.transport.completed_effect_count(), 0);
    assert_eq!(world.handlers.load(Ordering::SeqCst), 1);
    second.close(&execution);
    world.finish();
}
#[test]
fn bank_semantic_rejection_reopens_as_proven_no_effect_without_a_remote_admission() {
    let world = World::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = checkpoint_capture::ExecutionRequest::leased(&lease);
    let first = world.open(execution, true);
    let input = notice(8731);
    let scope = request_scope();
    let runtime = first.home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let outcome = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(Reject(action(input)))
        .without_source()
        .idempotency(&8731)
        .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("local emission commits before the external owner rejects its unsupported effect")
    };
    assert_eq!(
        receipt.external_dispatch().unwrap().posture().kind(),
        WorthQueryExternalDispatchPostureKind::ProvenNoEffect
    );
    let correlation = *receipt.dispatch_outbox().unwrap().correlation().bytes();
    let recorded = runtime
        .runtime()
        .durable_dispatch_custody(&correlation)
        .unwrap();
    assert_eq!(first.transport.production_dispatches().len(), 1);
    assert_eq!(first.transport.admission_count(), 0);
    assert_eq!(first.transport.completed_effect_count(), 0);
    drop(receipt);
    first.close(&execution);
    let second = world.open(execution, false);
    assert_eq!(
        second
            .home
            .runtime()
            .runtime()
            .durable_dispatch_custody(&correlation)
            .unwrap(),
        recorded
    );
    assert!(second.transport.production_dispatches().is_empty());
    assert_eq!(second.transport.admission_count(), 0);
    assert_eq!(second.transport.completed_effect_count(), 0);
    assert_eq!(world.handlers.load(Ordering::SeqCst), 1);
    second.close(&execution);
    world.finish();
}
