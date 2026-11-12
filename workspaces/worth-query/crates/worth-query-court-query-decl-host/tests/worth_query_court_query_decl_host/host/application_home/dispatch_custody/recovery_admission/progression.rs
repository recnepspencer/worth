//! Recovered recovery grants actual current transitions without repeating a completed effect.
use super::*;
use WorthQueryRecoveryClaimStatus as Status;
#[test]
fn a_recovered_uncertain_dispatch_safe_retries_once_and_records_completed_claim_custody() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, _) = owners.open(execution, true, &[Outcome::LostResponse]);
    let receipt = emit(&home, 0x8511);
    let old = mint(&home, &receipt);
    drop(receipt);
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[Outcome::Completed]);
    let handle = recover(&home, 0x8511);
    drop(old);
    admitted(&home, 0x8511, |phase, admission, _, _| {
        let identity = handle.opaque_wire_identity();
        let denied = home
            .runtime()
            .runtime()
            .settle_recorded_recovery_completion(phase, handle, admission)
            .unwrap_err();
        assert_eq!(
            denied.kind(),
            WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture
        );
        let (_, handle) = denied.into_parts();
        let handle = handle.expect("unresolved settlement retains the same live claim");
        assert_eq!(handle.opaque_wire_identity(), identity);
        assert!(rail.contacts().is_empty());
        let authority = home
            .runtime()
            .runtime()
            .admit_recovery_effect_authority(&handle, admission)
            .unwrap();
        let redispatch = home
            .runtime()
            .runtime()
            .redispatch_admitted_external_effect(phase, &handle, &authority, admission)
            .unwrap();
        let completed = safe_retry_recovery_handle(phase, handle, &authority, redispatch).unwrap();
        assert_eq!(
            completed.outbox_durability(),
            WorthQueryDispatchOutboxDurabilityPosture::DurableHome
        );
    });
    assert_eq!(rail.contacts().len(), 1);
    assert_recorded_before_contact(&home, &rail.contacts());
    assert_eq!(status(&home, 0x8511), Status::Completed);
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    assert_eq!(status(&home, 0x8511), Status::Completed);
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    close(home, &execution);
}
#[test]
fn recovered_compensation_and_original_resolution_consume_only_their_own_claims() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, _) = owners.open(
        execution,
        true,
        &[Outcome::LostResponse, Outcome::Completed],
    );
    let left = emit(&home, 0x8512);
    let left_handle = mint(&home, &left);
    let right = emit(&home, 0x8513);
    let right_handle = mint(&home, &right);
    drop((left, right));
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    let left = recover(&home, 0x8512);
    let right = recover(&home, 0x8513);
    drop((left_handle, right_handle));
    admitted(&home, 0x8512, |phase, admission, _, _| {
        let authority = home
            .runtime()
            .runtime()
            .admit_recovery_effect_authority(&left, admission)
            .unwrap();
        compensate_recovery_handle(phase, left, &authority).unwrap();
    });
    admitted(&home, 0x8513, |phase, admission, binding, _| {
        let authority = home
            .runtime()
            .runtime()
            .admit_recovery_effect_authority(&right, admission)
            .unwrap();
        let read = home
            .runtime()
            .runtime()
            .resolve_admitted_application_idempotency(phase, admission, binding)
            .unwrap();
        assert!(matches!(
            resolve_recovery_handle(phase, right, &authority, read).unwrap(),
            WorthQueryApplicationIdempotencyResolution::RecoveredCommitted(_)
        ));
    });
    for key in [0x8512, 0x8513] {
        assert_eq!(status(&home, key), Status::Consumed);
    }
    assert!(rail.contacts().is_empty());
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    for key in [0x8512, 0x8513] {
        assert_eq!(status(&home, key), Status::Consumed);
    }
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 2);
    close(home, &execution);
}

#[test]
fn a_recorded_completion_settles_a_recovered_live_claim_without_contacting_the_rail() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(execution, true, &[Outcome::Completed]);
    let receipt = emit(&home, 0x8514);
    assert_eq!(rail.contacts().len(), 1);
    let old = mint(&home, &receipt);
    drop(receipt);
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    let handle = recover(&home, 0x8514);
    drop(old);
    let original = handle
        .binding()
        .committed_product_publication()
        .composite_commit()
        .clone();
    admitted(&home, 0x8514, |phase, admission, _, _| {
        let settled = home
            .runtime()
            .runtime()
            .settle_recorded_recovery_completion(phase, handle, admission)
            .unwrap();
        assert_eq!(
            settled.committed_product_publication().composite_commit(),
            &original
        );
        assert!(settled
            .committed_product_publication()
            .composite_commit()
            .owner_identity()
            .durable()
            .is_some());
        assert_eq!(
            settled.provider_posture_kind(),
            Some(WorthQueryExternalDispatchPostureKind::Completed)
        );
    });
    assert_eq!(status(&home, 0x8514), Status::Completed);
    assert!(rail.contacts().is_empty());
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    assert_eq!(status(&home, 0x8514), Status::Completed);
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    close(home, &execution);
}
