use bank_server::{BankMutationCommitOutcome, BankRecoveryIdempotencyResolution};

use super::{disburse, fixture::disbursement_world, idempotency};
use crate::support::request_scope;

#[test]
fn disbursement_recovery_uses_its_own_admitted_operation() {
    let fixture = disbursement_world("disbursement-recovery-authority", 1_000);
    let specialist = fixture.authenticate_actor();
    let action = fixture.action(250);
    let outcome = disburse(&fixture, &specialist, action, idempotency(91))
        .expect("the disbursement should commit through the production entry");
    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the disbursement must commit: {outcome:?}");
    };
    let handle = fixture
        .world
        .runtime
        .open_commit_recovery(&receipt)
        .expect("the committed receipt must open its exact recovery handle");
    fixture
        .world
        .runtime
        .inspect_commit_recovery(&handle, &specialist, action, &request_scope())
        .expect("inspection uses the disbursement capability");
    let resolution = fixture
        .world
        .runtime
        .resolve_commit_recovery(handle, &specialist, action, &request_scope())
        .expect("resolution must use the disbursement operation rather than NotifyDeath");
    assert!(matches!(
        resolution,
        BankRecoveryIdempotencyResolution::AlreadyCommitted
    ));
}

#[test]
fn disbursement_compensation_uses_its_own_admitted_operation() {
    let fixture = disbursement_world("disbursement-compensation-authority", 1_000);
    let specialist = fixture.authenticate_actor();
    let action = fixture.action(250);
    let outcome = disburse(&fixture, &specialist, action, idempotency(92))
        .expect("the disbursement should commit through the production entry");
    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the disbursement must commit: {outcome:?}");
    };
    let handle = fixture
        .world
        .runtime
        .open_commit_recovery(&receipt)
        .expect("the committed receipt must open its exact recovery handle");
    let transition = fixture
        .world
        .runtime
        .compensate_commit_recovery(handle, &specialist, action, &request_scope())
        .expect("compensation must admit the disbursement operation");
    assert_eq!(transition.installed_operation(), "DisburseEstateOperation");
}
