use bank_domain::schema::{AccountStatus, AccountingRevision, Status};

use super::*;

#[test]
fn ordinary_send_money_carries_typed_preconditions_through_retry_receipts() {
    let fixture = ordinary_read_world("ordinary-preconditions", 0);
    let owner = fixture.authenticate(OWNER);
    let before = account_summary(&fixture, &owner, fixture.personal_account);
    let send = SendMoney {
        from: fixture.personal_account,
        recipient: principal_id(RECIPIENT),
        amount: Money::from_minor(250).unwrap(),
    };

    let mismatch = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send.clone()))
        .as_principal(&owner)
        .expect_version(
            AccountingRevision::reference(),
            before.accounting_revision(),
        )
        .unwrap()
        .expect_fact(Status::reference(), AccountStatus::Closed)
        .unwrap()
        .controls(BankMutationControls::new(
            request_scope(),
            key("typed-mismatch"),
        ))
        .execute();
    assert!(matches!(
        mismatch.status(),
        BankMutationStatus::Denied(bank_server::BankMutationDenial::Preparation(
            bank_server::BankCommitPreparationDenial::Application {
                kind: bank_server::BankApplicationAttemptDenialKind::MutationPreconditionMismatch,
                ..
            }
        ))
    ));
    assert_eq!(
        account_summary(&fixture, &owner, fixture.personal_account)
            .current_balance()
            .minor_units(),
        before.current_balance().minor_units()
    );

    let committed = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send.clone()))
        .as_principal(&owner)
        .expect_version(
            AccountingRevision::reference(),
            before.accounting_revision(),
        )
        .unwrap()
        .expect_fact(Status::reference(), before.status())
        .unwrap()
        .controls(BankMutationControls::new(
            request_scope(),
            key("typed-send"),
        ))
        .execute();
    let BankMutationStatus::Committed(receipt) = committed.status() else {
        panic!("typed send must commit: {committed:?}");
    };
    assert_eq!(receipt.expected_version_count(), 1);
    assert_eq!(receipt.expected_fact_count(), 1);
    assert_warm_canonical_work_is_zero(receipt.canonical_work());

    let retried = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send.clone()))
        .as_principal(&owner)
        .expect_version(
            AccountingRevision::reference(),
            before.accounting_revision(),
        )
        .unwrap()
        .expect_fact(Status::reference(), before.status())
        .unwrap()
        .controls(BankMutationControls::new(
            request_scope(),
            key("typed-send"),
        ))
        .execute();
    let BankMutationStatus::AlreadyCommitted(recovered) = retried.status() else {
        panic!("lost-response retry must recover the commit: {retried:?}");
    };
    assert_eq!(recovered.canonical_work(), receipt.canonical_work());
    assert_warm_canonical_work_is_zero(recovered.canonical_work());

    let after = account_summary(&fixture, &owner, fixture.personal_account);
    assert_eq!(
        after.current_balance().minor_units(),
        before.current_balance().minor_units() - send.amount.minor_units()
    );

    let intent_drift = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send))
        .as_principal(&owner)
        .expect_version(
            AccountingRevision::reference(),
            before.accounting_revision().next().unwrap(),
        )
        .unwrap()
        .expect_fact(Status::reference(), before.status())
        .unwrap()
        .controls(BankMutationControls::new(
            request_scope(),
            key("typed-send"),
        ))
        .execute();
    assert!(
        matches!(
            intent_drift.status(),
            BankMutationStatus::Denied(bank_server::BankMutationDenial::IdempotencyIntentDrift)
        ),
        "changed precondition outcome: {intent_drift:?}"
    );
    assert_eq!(
        account_summary(&fixture, &owner, fixture.personal_account)
            .current_balance()
            .minor_units(),
        after.current_balance().minor_units()
    );
}

fn assert_warm_canonical_work_is_zero(phases: bank_server::BankCommitCanonicalWorkPhases) {
    let zero = bank_server::BankCommitCanonicalWorkEvidence::default();
    assert!(phases.installation().basis_preparations() > 0);
    assert!(phases.installation().canonical_encoded_bytes() > 0);
    assert!(phases.admission().basis_preparations() > 0);
    assert!(phases.admission().canonical_encoded_bytes() > 0);
    assert_eq!(phases.execution(), zero);
    assert_eq!(phases.provider_commit(), zero);
    assert_eq!(phases.projection(), zero);
    assert_eq!(phases.live_delivery(), zero);
    assert_eq!(phases.retry_resolution(), zero);
    assert_eq!(phases.recovery_inspection(), zero);
    assert_eq!(phases.publication(), zero);
}

fn account_summary(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
    account: bank_domain::model::AccountId,
) -> bank_domain::reads::AccountSummary {
    fixture
        .world
        .runtime
        .query(queries::account_summary(account))
        .as_principal(principal)
        .controls(read_controls())
        .execute()
        .expect("account summary must be query-visible")
        .into_rows()
        .into_iter()
        .next()
        .expect("the requested account must be visible")
}
