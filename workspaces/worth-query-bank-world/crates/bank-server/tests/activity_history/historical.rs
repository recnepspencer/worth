use std::num::NonZeroUsize;

use bank_domain::model::{CustomerRole, Money};
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{Deposit, GrantAccountAuthorization, RevokeAccountAuthorization};
use bank_server::{
    mutations, BankApplicationQueryAdmissionDenialKind, BankApplicationQueryDenial,
    BankMutationControls,
};

use super::fixture::{ordinary_read_world, OrdinaryReadFixture, OWNER, STRANGER, TELLER, VIEWER};
use super::support::request_scope;

#[test]
fn account_activity_reads_one_real_prior_bank_commit() {
    let fixture = ordinary_read_world("historical-account-activity", 0);
    let owner = fixture.authenticate(OWNER);
    let teller = fixture.authenticate(TELLER);
    let historical_commit = commit_deposit(&fixture, &teller, 1, "historical-first");
    commit_deposit(&fixture, &teller, 2, "historical-second");

    let historical_request = request_scope();
    let historical = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .historical(
            &historical_commit,
            NonZeroUsize::new(16).unwrap(),
            NonZeroUsize::new(2_048).unwrap(),
            &historical_request,
        )
        .expect("the retained bank commit must remain queryable");
    let current = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .execute(bank_server::BankReadControls::current(request_scope(), 16).unwrap())
        .expect("the current account activity must execute");

    assert_eq!(historical.rows()[0].entries().len(), 3);
    assert_eq!(current.rows()[0].entries().len(), 4);
    assert_eq!(historical.rows()[0].entries()[2].amount().minor_units(), 1);
    let receipt = historical.receipt();
    assert_eq!(receipt.inspect().result_count(), 1);
    assert!(receipt.inspect().terminal_resources_released());
}

#[test]
fn foreign_bank_commit_receipt_cannot_select_local_history() {
    let source = ordinary_read_world("historical-receipt-source", 0);
    let target = ordinary_read_world("historical-receipt-target", 0);
    let source_teller = source.authenticate(TELLER);
    let target_owner = target.authenticate(OWNER);
    let target_teller = target.authenticate(TELLER);
    let foreign_commit = commit_deposit(&source, &source_teller, 1, "foreign-history");
    let _local_commit = commit_deposit(&target, &target_teller, 1, "local-history");
    let request = request_scope();

    let denial = target
        .world
        .runtime
        .account_activity(target.personal_account)
        .as_principal(&target_owner)
        .historical(
            &foreign_commit,
            NonZeroUsize::new(16).unwrap(),
            NonZeroUsize::new(2_048).unwrap(),
            &request,
        )
        .err()
        .expect("a foreign bank commit receipt must not select local history");

    assert!(matches!(
        denial,
        BankApplicationQueryDenial::HistoricalCommitUnavailable
            | BankApplicationQueryDenial::ProductSelection(_)
    ));
}

fn commit_deposit(
    fixture: &OrdinaryReadFixture,
    teller: &bank_server::BankAuthenticatedPrincipal,
    minor_units: i64,
    idempotency_key: &str,
) -> worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt {
    let outcome = fixture
        .world
        .runtime
        .mutate(mutations::deposit(Deposit {
            institution: fixture.institution,
            account: fixture.personal_account,
            amount: Money::from_minor(minor_units).unwrap(),
        }))
        .as_principal(teller)
        .controls(BankMutationControls::new(
            request_scope(),
            BankIdempotencyKey::new(idempotency_key).unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("deposit construction: {cause:?}"));
    let scope = request_scope();
    let request = fixture.world.runtime.request(teller, &scope);
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
        completed,
    ) = outcome.advance(&request, Some(NonZeroUsize::new(20_000).unwrap()))
    else {
        panic!("native retained deposit completion")
    };
    let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
        receipt,
        ..
    } = completed.evidence()
    else {
        panic!("fresh deposit evidence")
    };
    let receipt = receipt.clone();
    assert!(completed.close(&request).is_ok());

    receipt
}

#[test]
fn historical_read_rechecks_current_account_authorization() {
    let fixture = ordinary_read_world("historical-current-authorization", 0);
    let owner = fixture.authenticate(OWNER);
    let viewer = fixture.authenticate(VIEWER);
    let teller = fixture.authenticate(TELLER);
    let commit = commit_deposit(&fixture, &teller, 1, "historical-before-revocation");
    let read = |request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope| {
        fixture
            .world
            .runtime
            .account_activity(fixture.personal_account)
            .as_principal(&viewer)
            .historical(
                &commit,
                NonZeroUsize::new(16).unwrap(),

                NonZeroUsize::new(2_048).unwrap(), request,
            )
    };
    assert!(read(&request_scope()).is_ok());

    let authorization = super::authorized_user_id(&fixture, &owner, VIEWER);
    let revoked_scope = crate::support::request_scope();
    let revoked_request = fixture.world.runtime.request(&owner, &revoked_scope);
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::revoke_account_access(
            RevokeAccountAuthorization {
                account: fixture.personal_account,
                authorization,
            },
        ))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            request_scope(),
            BankIdempotencyKey::new("historical-revoke-viewer").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("declared source action: {cause:?}"));
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
        revoked,
    ) = operation.advance(&revoked_request)
    else {
        panic!("fresh source completion")
    };
    assert!(matches!(
        revoked.evidence(),
        worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
    ));
    revoked
        .close(&revoked_request)
        .unwrap_or_else(|(_, cause)| panic!("lawful source close: {cause:?}"));
    let denied = read(&request_scope());
    assert!(matches!(
        denied,
        Err(BankApplicationQueryDenial::Admission(denial))
            if matches!(
                denial.kind(),
                BankApplicationQueryAdmissionDenialKind::Authorization(_)
            )
    ));
}

#[test]
fn newly_granted_reader_can_read_an_earlier_commit() {
    let fixture = ordinary_read_world("historical-new-viewer", 0);
    let owner = fixture.authenticate(OWNER);
    let stranger = fixture.authenticate(STRANGER);
    let teller = fixture.authenticate(TELLER);
    let commit = commit_deposit(&fixture, &teller, 1, "historical-before-grant");

    let granted_scope = crate::support::request_scope();
    let granted_request = fixture.world.runtime.request(&owner, &granted_scope);
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::grant_account_access(GrantAccountAuthorization {
            account: fixture.personal_account,
            principal: super::fixture::principal_id(STRANGER),
            role: CustomerRole::Viewer,
        }))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            request_scope(),
            BankIdempotencyKey::new("historical-grant-new-viewer").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("declared source action: {cause:?}"));
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
        granted,
    ) = operation.advance(&granted_request)
    else {
        panic!("fresh source completion")
    };
    assert!(matches!(
        granted.evidence(),
        worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
    ));
    granted
        .close(&granted_request)
        .unwrap_or_else(|(_, cause)| panic!("lawful source close: {cause:?}"));

    let request = request_scope();
    let historical = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&stranger)
        .historical(
            &commit,
            NonZeroUsize::new(16).unwrap(),
            NonZeroUsize::new(2_048).unwrap(),
            &request,
        )
        .expect("current authorization should admit the retained rows");
    assert_eq!(historical.rows()[0].entries().len(), 3);
    assert_eq!(historical.rows()[0].entries()[2].amount().minor_units(), 1);
}
