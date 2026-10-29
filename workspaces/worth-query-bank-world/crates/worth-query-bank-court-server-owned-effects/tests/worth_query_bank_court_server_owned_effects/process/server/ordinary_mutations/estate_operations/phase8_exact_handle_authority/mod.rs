pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::phase8_exact_handle_authority::*;

#[test]
fn cloned_receipt_cannot_mint_a_second_recovery_handle() {
    let world = cross_gate_world("same-runtime-receipt-mint-claim");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_notification(120);
    let copied_receipt = receipt.clone();
    let handle = world.open_recovery(&receipt);
    let denied = world
        .fixture
        .world
        .runtime
        .open_commit_recovery(&copied_receipt)
        .expect_err("one authoritative commit can open only one handle");
    assert_eq!(denied.kind(), BankRecoveryDenialKind::RecoveryAlreadyMinted);
    drop(handle);
}

#[test]
fn a_receipt_committed_by_another_runtime_cannot_mint_a_handle_here() {
    let committing = cross_gate_world("cross-runtime-receipt-mint-committing");
    let bystander = cross_gate_world("cross-runtime-receipt-mint-bystander");
    committing
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = committing.commit_notification(127);
    let denied = bystander
        .fixture
        .world
        .runtime
        .open_commit_recovery(&receipt)
        .expect_err("a runtime cannot open recovery for a foreign commit");
    assert_eq!(denied.kind(), BankRecoveryDenialKind::ForeignRuntime);
    drop(committing.open_recovery(&receipt));
}

#[test]
fn a_completed_bank_transition_spends_the_recovery_permanently() {
    let world = cross_gate_world("completed-transition-spends-recovery");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_notification(130);
    let handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    world
        .fixture
        .world
        .runtime
        .reconcile_commit_recovery(
            handle,
            &specialist,
            world.specialist_action(),
            &request_scope(),
        )
        .expect("Bank-owned recovery transition admits");
    let denied = world
        .fixture
        .world
        .runtime
        .open_commit_recovery(&receipt)
        .expect_err("an exercised recovery cannot be reopened");
    assert_eq!(denied.kind(), BankRecoveryDenialKind::RecoveryAlreadyMinted);
}

#[test]
fn expiry_decision_is_consumed_only_by_the_bank_transition() {
    let authorization_time = AuthorizationTimeController::at_epoch_seconds(2_000);
    let world = cross_gate_world_with_authorization_time(
        "bank-owned-expiry-transition",
        Some(authorization_time.clone()),
    );
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_notification(125);
    let handle = world.open_recovery(&receipt);
    authorization_time.advance_to_epoch_seconds(5_601);
    let evaluation = world
        .fixture
        .world
        .runtime
        .evaluate_commit_recovery_expiry(&handle)
        .expect("expiry evaluation");
    let BankRecoveryExpiryEvaluation::Expired(decision) = evaluation else {
        panic!("advanced clock must expire the Bank handle");
    };
    world
        .fixture
        .world
        .runtime
        .expire_commit_recovery(handle, decision)
        .expect("Bank consumes its opaque expiry decision");
}
