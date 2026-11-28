pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::phase8_recovery_expiry::*;

#[test]
fn current_expiry_evidence_remains_descriptive() {
    let world = cross_gate_world("expiry-lifecycle");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt_closed = world.commit_notification(77);
    let receipt = receipt_closed
        .live_receipt()
        .expect("fresh source retains its full native owner");
    let handle = world.open_recovery(receipt);
    assert_eq!(
        world
            .fixture
            .world
            .runtime
            .commit_recovery_claim_status(receipt),
        Ok(BankRecoveryClaimStatus::Live)
    );
    assert!(matches!(
        world
            .fixture
            .world
            .runtime
            .evaluate_commit_recovery_expiry(&handle)
            .expect("expiry evaluation"),
        BankRecoveryExpiryEvaluation::Current
    ));
    drop(handle);
}

#[test]
fn clock_advanced_expiry_terminalizes_through_bank() {
    let authorization_time = std::sync::Arc::new(
        worth_query_host::facade::runtime::time::ControlledTime::at_unix_secs(2_000),
    );
    let world = cross_gate_world_with_authorization_time(
        "expiry-terminal",
        Some(authorization_time.clone()),
    );
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt_closed = world.commit_notification(78);
    let receipt = receipt_closed
        .live_receipt()
        .expect("fresh source retains its full native owner");
    let handle = world.open_recovery(receipt);
    authorization_time.set_wall_unix_nanos((5_601) * 1_000_000_000);
    let BankRecoveryExpiryEvaluation::Expired(decision) = world
        .fixture
        .world
        .runtime
        .evaluate_commit_recovery_expiry(&handle)
        .expect("expiry evaluation")
    else {
        panic!("advanced clock must produce expired evidence");
    };
    world
        .fixture
        .world
        .runtime
        .expire_commit_recovery(handle, decision)
        .expect("expire terminal");
    assert_eq!(
        world
            .fixture
            .world
            .runtime
            .commit_recovery_claim_status(receipt),
        Ok(BankRecoveryClaimStatus::Expired)
    );
    let denied = world
        .fixture
        .world
        .runtime
        .open_commit_recovery(receipt)
        .expect_err("expired recovery remains terminal");
    assert_eq!(denied.kind(), BankRecoveryDenialKind::RecoveryAlreadyMinted);
}
