use bank_server::BankRecoveryIdempotencyResolution;

use super::{disburse, fixture::disbursement_world, idempotency};
use crate::support::request_scope;

#[test]
fn disbursement_recovery_uses_its_own_admitted_operation() {
    let fixture = disbursement_world("disbursement-recovery-authority", 1_000);
    let specialist = fixture.authenticate_actor();
    let action = fixture.action(250);
    let outcome = {
        let closing_actor = &specialist;
        let closing_scope = crate::support::request_scope();
        let closing_request = (&fixture)
            .world
            .runtime
            .request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = disburse(&fixture, &specialist, action, idempotency(91))
        else {
            panic!("native source completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        bank_server::BankCommitReceipt::from_closed_source(
            done.close_with_evidence(&closing_request)
                .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}")),
        )
    };
    let receipt = outcome;
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
    let outcome = {
        let closing_actor = &specialist;
        let closing_scope = crate::support::request_scope();
        let closing_request = (&fixture)
            .world
            .runtime
            .request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = disburse(&fixture, &specialist, action, idempotency(92))
        else {
            panic!("native source completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        bank_server::BankCommitReceipt::from_closed_source(
            done.close_with_evidence(&closing_request)
                .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}")),
        )
    };
    let receipt = outcome;
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
