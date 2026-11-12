//! Real signed Bank callback bytes complete the original after Host reopen and remain deduplicated.
use super::fixture::*;
use std::sync::atomic::Ordering;
use std::time::Duration;
use worth_query_host::facade::primary_graph::*;
#[test]
fn authentic_bank_callback_after_reopen_completes_the_original_and_deduplicates_another_reopen() {
    let mut world = World::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = checkpoint_capture::ExecutionRequest::leased(&lease);
    let first = world.open(execution, true);
    first
        .transport
        .under(FaultScript::CommitThenLoseResponse, Duration::from_secs(5));
    let input = notice(8741);
    let receipt = first.emit(input.clone(), 8741);
    assert_eq!(
        receipt.external_dispatch().unwrap().posture().kind(),
        WorthQueryExternalDispatchPostureKind::Unresolved
    );
    let correlation = *receipt.dispatch_outbox().unwrap().correlation().bytes();
    let original = receipt.commit_reference().clone();
    drop(receipt);
    first.close(&execution);
    let second = world.open(execution, false);
    let recovered = second.recover(input.clone(), 8741);
    let identity = recovered.opaque_wire_identity();
    let denied = second
        .admitted(input.clone(), 8741, |phase, admission, _, _| {
            second
                .home
                .runtime()
                .runtime()
                .settle_recorded_recovery_completion(phase, recovered, admission)
        })
        .unwrap_err();
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture
    );
    let (_, recovered) = denied.into_parts();
    let recovered = recovered.expect("no actual canonical terminal preserves original live claim");
    assert_eq!(recovered.opaque_wire_identity(), identity);
    let body = world.callback.receive(&world.capture_loop.blocking);
    let mut altered = body.clone();
    altered[130] ^= 1;
    assert!(matches!(
        second.home.runtime().runtime().receive_inbound_occurrence(
            &second.verifier,
            &altered,
            &request_scope()
        ),
        Err(WorthQueryInboundAdmissionDenial::Verification(
            WorthQueryInboundVerificationDenial::AuthenticationFailed
        ))
    ));
    let received = second
        .home
        .runtime()
        .runtime()
        .receive_inbound_occurrence(&second.verifier, &body, &request_scope())
        .unwrap();
    assert_eq!(
        received.posture(),
        WorthQueryInboundReceiptPosture::Performed
    );
    let message = *received.message_identity();
    drop(received);
    second.maintain();
    let terminal = second
        .home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&second.verifier, correlation)
        .unwrap();
    assert_eq!(recovered.binding().commit_reference(), &original);
    second
        .admitted(input, 8741, |phase, admission, _, _| {
            second
                .home
                .runtime()
                .runtime()
                .settle_recorded_recovery_completion(phase, recovered, admission)
        })
        .unwrap();
    assert!(second.transport.production_dispatches().is_empty());
    assert_eq!(second.transport.completed_effect_count(), 1);
    second.close(&execution);
    let third = world.open(execution, false);
    let duplicate = third
        .home
        .runtime()
        .runtime()
        .receive_inbound_occurrence(&third.verifier, &body, &request_scope())
        .unwrap();
    assert_eq!(
        duplicate.posture(),
        WorthQueryInboundReceiptPosture::AlreadyCompleted
    );
    assert_eq!(duplicate.message_identity(), &message);
    drop(duplicate);
    third.maintain();
    let repeated = third
        .home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&third.verifier, correlation)
        .unwrap();
    // Current opener fences change; full Home/World/ordinal DATA coordinates remain original.
    assert_eq!(
        (
            repeated
                .original_world_commit()
                .owner_identity()
                .durable()
                .unwrap(),
            repeated.original_world_commit().ordinal()
        ),
        (
            terminal
                .original_world_commit()
                .owner_identity()
                .durable()
                .unwrap(),
            terminal.original_world_commit().ordinal()
        ),
    );
    assert_eq!(
        (
            repeated
                .completion_world_commit()
                .owner_identity()
                .durable()
                .unwrap(),
            repeated.completion_world_commit().ordinal()
        ),
        (
            terminal
                .completion_world_commit()
                .owner_identity()
                .durable()
                .unwrap(),
            terminal.completion_world_commit().ordinal()
        ),
    );
    assert_eq!(
        (
            repeated
                .completion_attempt()
                .owner_identity()
                .durable()
                .unwrap(),
            repeated.completion_attempt().ordinal()
        ),
        (
            terminal
                .completion_attempt()
                .owner_identity()
                .durable()
                .unwrap(),
            terminal.completion_attempt().ordinal()
        ),
    );
    assert_ne!(
        repeated.original_world_commit().owner_identity(),
        terminal.original_world_commit().owner_identity()
    );
    assert!(third.transport.production_dispatches().is_empty());
    assert_eq!(third.transport.admission_count(), 1);
    assert_eq!(third.transport.completed_effect_count(), 1);
    assert_eq!(world.handlers.load(Ordering::SeqCst), 1);
    third.close(&execution);
    world.finish();
}
#[test]
fn a_bank_completion_after_transport_timeout_is_authenticated_on_recovered_custody() {
    let mut world = World::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = checkpoint_capture::ExecutionRequest::leased(&lease);
    let first = world.open(execution, true);
    first.transport.under(
        FaultScript::CompleteAfterDelay { delay_millis: 250 },
        Duration::from_millis(10),
    );
    let input = notice(8751);
    let receipt = first.emit(input, 8751);
    assert_eq!(
        receipt.external_dispatch().unwrap().posture().kind(),
        WorthQueryExternalDispatchPostureKind::Unresolved
    );
    let correlation = *receipt.dispatch_outbox().unwrap().correlation().bytes();
    drop(receipt);
    first.close(&execution);
    let second = world.open(execution, false);
    let body = world.callback.receive(&world.capture_loop.blocking);
    let accepted = second
        .home
        .runtime()
        .runtime()
        .receive_inbound_occurrence(&second.verifier, &body, &request_scope())
        .unwrap();
    assert_eq!(
        accepted.posture(),
        WorthQueryInboundReceiptPosture::Performed
    );
    drop(accepted);
    second.maintain();
    assert!(second
        .home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&second.verifier, correlation)
        .is_some());
    assert!(second.transport.production_dispatches().is_empty());
    assert_eq!(second.transport.admission_count(), 1);
    assert_eq!(second.transport.completed_effect_count(), 1);
    assert_eq!(world.handlers.load(Ordering::SeqCst), 1);
    second.close(&execution);
    world.finish();
}
