//! Recovery expiry is evaluated and consumed through Bank-owned phases.

use bank_external_rail::test_control::FaultScript;
use bank_server::{BankRecoveryClaimStatus, BankRecoveryDenialKind, BankRecoveryExpiryEvaluation};

use super::phase8_cross_gate::world::{
    cross_gate_world, cross_gate_world_with_authorization_time, PATIENT,
};
use crate::authorization_time::AuthorizationTimeController;

#[test]
fn current_expiry_evidence_remains_descriptive() {
    let world = cross_gate_world("expiry-lifecycle");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_notification(77);
    let handle = world.open_recovery(&receipt);
    assert_eq!(
        world
            .fixture
            .world
            .runtime
            .commit_recovery_claim_status(&receipt),
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
    let authorization_time = AuthorizationTimeController::at_epoch_seconds(2_000);
    let world = cross_gate_world_with_authorization_time(
        "expiry-terminal",
        Some(authorization_time.clone()),
    );
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_notification(78);
    let handle = world.open_recovery(&receipt);
    authorization_time.advance_to_epoch_seconds(5_601);
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
            .commit_recovery_claim_status(&receipt),
        Ok(BankRecoveryClaimStatus::Expired)
    );
    let denied = world
        .fixture
        .world
        .runtime
        .open_commit_recovery(&receipt)
        .expect_err("expired recovery remains terminal");
    assert_eq!(denied.kind(), BankRecoveryDenialKind::RecoveryAlreadyMinted);
}
