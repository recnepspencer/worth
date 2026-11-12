//! Claim generation, expiry and Drop settlement retain one original durable obligation.
use super::*;
use WorthQueryRecoveryClaimStatus as Status;
#[test]
fn a_live_claim_reissues_the_same_deadline_after_reopen_and_fences_the_old_handle() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(execution, true, &[Outcome::LostResponse]);
    let receipt = emit(&home, 0x8501);
    let old = mint(&home, &receipt);
    let expiry = old.binding().expires_at_unix_ms();
    assert_eq!(
        home.runtime()
            .runtime()
            .recovery_claim_status(&receipt)
            .unwrap(),
        Status::Live
    );
    assert_eq!(rail.contacts().len(), 1);
    drop(receipt);
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    assert_eq!(status(&home, 0x8501), Status::Live);
    let denied = admitted(&home, 0x8501, |_, admission, _, _| {
        home.runtime()
            .runtime()
            .admit_recovery_effect_authority(&old, admission)
    })
    .unwrap_err();
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::StaleIncarnation
    );
    drop(old);
    let handle = recover(&home, 0x8501);
    assert_eq!(handle.binding().expires_at_unix_ms(), expiry);
    let denied = admitted(&home, 0x8501, |phase, admission, binding, input| {
        home.runtime()
            .runtime()
            .mint_admitted_recovery_handle(phase, admission, binding, input)
    })
    .unwrap_err();
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::RecoveryAlreadyMinted
    );
    admitted(&home, 0x8501, |phase, admission, _, _| {
        let authority = home
            .runtime()
            .runtime()
            .admit_recovery_effect_authority(&handle, admission)
            .unwrap();
        reconcile_recovery_handle(phase, handle, &authority).unwrap();
    });
    assert_eq!(status(&home, 0x8501), Status::Consumed);
    assert!(rail.contacts().is_empty());
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    assert_eq!(status(&home, 0x8501), Status::Consumed);
    let denied = admitted(&home, 0x8501, |phase, admission, binding, input| {
        home.runtime()
            .runtime()
            .mint_admitted_recovery_handle(phase, admission, binding, input)
    })
    .unwrap_err();
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::RecoveryAlreadyMinted
    );
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    close(home, &execution);
}
#[test]
fn a_dropped_home_handle_keeps_its_claim_until_funded_disposal_is_acknowledged() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, _) = owners.open(execution, true, &[Outcome::LostResponse]);
    let receipt = emit(&home, 0x8502);
    let handle = mint(&home, &receipt);
    drop(handle);
    assert_eq!(
        home.runtime()
            .runtime()
            .recovery_claim_status(&receipt)
            .unwrap(),
        Status::Live
    );
    home.runtime()
        .runtime()
        .with_host_advancement(|phase| home.runtime().runtime().maintain_recovery_claims(&phase))
        .unwrap()
        .unwrap();
    assert_eq!(
        home.runtime()
            .runtime()
            .recovery_claim_status(&receipt)
            .unwrap(),
        Status::Disposed
    );
    drop(receipt);
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    assert_eq!(status(&home, 0x8502), Status::Disposed);
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    close(home, &execution);
}
#[test]
fn a_recovered_claim_expires_at_its_original_owner_deadline_and_stays_expired() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, _) = owners.open(execution, true, &[Outcome::LostResponse]);
    let receipt = emit(&home, 0x8503);
    let old = mint(&home, &receipt);
    let expiry = old.binding().expires_at_unix_ms().unwrap();
    drop(receipt);
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    let handle = recover(&home, 0x8503);
    assert_eq!(handle.binding().expires_at_unix_ms(), Some(expiry));
    drop(old);
    owners.now.store(expiry / 1000 + 1, Ordering::SeqCst);
    let WorthQueryRecoveryExpiryEvaluation::Expired(decision) = home
        .runtime()
        .runtime()
        .evaluate_recovery_expiry(&handle)
        .unwrap()
    else {
        panic!("actual owner clock passes original deadline")
    };
    home.runtime()
        .runtime()
        .with_host_advancement(|phase| expire_recovery_handle(&phase, handle, &decision))
        .unwrap()
        .unwrap();
    assert_eq!(status(&home, 0x8503), Status::Expired);
    assert!(rail.contacts().is_empty());
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    assert_eq!(status(&home, 0x8503), Status::Expired);
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    close(home, &execution);
}

#[test]
fn a_relinquished_claim_records_its_release_before_a_new_generation_is_admitted() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, _) = owners.open(
        execution,
        true,
        &[Outcome::LostResponse, Outcome::LostResponse],
    );
    let left = emit(&home, 0x8531);
    let left_handle = mint(&home, &left);
    let old_expiry = left_handle.binding().expires_at_unix_ms();
    let right = emit(&home, 0x8532);
    let right_handle = mint(&home, &right);
    let foreign = admitted(&home, 0x8532, |_, admission, _, _| {
        home.runtime()
            .runtime()
            .admit_recovery_effect_authority(&right_handle, admission)
    })
    .unwrap();
    let denial = home
        .runtime()
        .runtime()
        .with_host_advancement(|phase| compensate_recovery_handle(&phase, left_handle, &foreign))
        .unwrap()
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied
    );
    assert_eq!(
        home.runtime()
            .runtime()
            .recovery_claim_status(&left)
            .unwrap(),
        Status::Live
    );
    home.runtime()
        .runtime()
        .with_host_advancement(|phase| home.runtime().runtime().maintain_recovery_claims(&phase))
        .unwrap()
        .unwrap();
    assert_eq!(
        home.runtime()
            .runtime()
            .recovery_claim_status(&left)
            .unwrap(),
        Status::Unclaimed
    );
    owners.now.fetch_add(3, Ordering::SeqCst);
    drop((left, right));
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    assert_eq!(status(&home, 0x8531), Status::Unclaimed);
    let handle = recover(&home, 0x8531);
    assert!(handle.binding().expires_at_unix_ms() > old_expiry);
    admitted(&home, 0x8531, |phase, admission, _, _| {
        let authority = home
            .runtime()
            .runtime()
            .admit_recovery_effect_authority(&handle, admission)
            .unwrap();
        reconcile_recovery_handle(phase, handle, &authority).unwrap();
    });
    drop(right_handle);
    let right = recover(&home, 0x8532);
    admitted(&home, 0x8532, |phase, admission, _, _| {
        let authority = home
            .runtime()
            .runtime()
            .admit_recovery_effect_authority(&right, admission)
            .unwrap();
        dispose_recovery_handle(phase, right, &authority).unwrap();
    });
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 2);
    close(home, &execution);
}
