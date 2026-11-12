//! Actual no-effect terminal publication and immutable original-input refusal.
use super::*;
use worth_query_execution::facade::application_home_integration::QueryDispatchJournalDenial;
use WorthQueryRecoveryClaimStatus as Status;
#[test]
fn a_no_effect_terminal_publication_returns_the_same_live_recovery_handle() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(execution, true, &[Outcome::LostResponse]);
    let receipt = emit(&home, 0x8521);
    let handle = mint(&home, &receipt);
    let identity = handle.opaque_wire_identity();
    home.stop_next_prepared_dispatch_publication_for_test();
    let failed = admitted(&home, 0x8521, |phase, admission, _, _| {
        let authority = home
            .runtime()
            .runtime()
            .admit_recovery_effect_authority(&handle, admission)
            .unwrap();
        dispose_recovery_handle(phase, handle, &authority)
    })
    .unwrap_err();
    let (denial, handle) = failed.into_parts();
    assert_eq!(
        denial.source(),
        Some(&WorthQueryRecoverySourceDenial::Journal(
            QueryDispatchJournalDenial::PublicationNoEffect
        ))
    );
    let handle = handle.expect("an actual no-effect returns its original live capability");
    assert_eq!(handle.opaque_wire_identity(), identity);
    assert_eq!(
        home.runtime()
            .runtime()
            .recovery_claim_status(&receipt)
            .unwrap(),
        Status::Live
    );
    admitted(&home, 0x8521, |phase, admission, _, _| {
        let authority = home
            .runtime()
            .runtime()
            .admit_recovery_effect_authority(&handle, admission)
            .unwrap();
        dispose_recovery_handle(phase, handle, &authority).unwrap();
    });
    assert_eq!(
        home.runtime()
            .runtime()
            .recovery_claim_status(&receipt)
            .unwrap(),
        Status::Disposed
    );
    assert_eq!(rail.contacts().len(), 1);
    drop(receipt);
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    assert_eq!(status(&home, 0x8521), Status::Disposed);
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    close(home, &execution);
}
#[test]
fn recovered_claim_admission_refuses_changed_original_input_without_repeating_the_handler() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, _) = owners.open(execution, true, &[Outcome::LostResponse]);
    let receipt = emit(&home, 0x8522);
    let old = mint(&home, &receipt);
    drop(receipt);
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    let denial = admitted_input(&home, 0x8522, 0x8523, |phase, admission, binding, input| {
        home.runtime()
            .runtime()
            .mint_admitted_recovery_handle(phase, admission, binding, input)
    })
    .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryRecoveryHandleDenialKind::GovernedInputMismatch
    );
    assert_eq!(status(&home, 0x8522), Status::Live);
    let denial = admitted(&home, 0x8522, |phase, admission, binding, _| {
        let substituted =
            worth_query_decl::facade::application_operation::ApplicationEncodedInput::<
                worth_query_testkit_host::facade::host::dispatch_home::notify::BalanceValueBinding,
            >::encode(0x8523)
            .unwrap();
        home.runtime().runtime().mint_admitted_recovery_handle(
            phase,
            admission,
            binding,
            &substituted,
        )
    })
    .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryRecoveryHandleDenialKind::GovernedInputMismatch
    );
    assert_eq!(status(&home, 0x8522), Status::Live);
    let handle = recover(&home, 0x8522);
    drop((old, handle));
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    close(home, &execution);
}
