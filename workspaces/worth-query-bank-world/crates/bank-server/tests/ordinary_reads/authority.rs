use std::time::{Duration, Instant};

use bank_server::{queries, BankApplicationQueryDenial, BankReadControls};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionDenialKind;

use super::fixture::{ordinary_read_world, APPROVER, AUDITOR, OWNER, STRANGER, TELLER, VIEWER};
use crate::support::request_scope;

#[test]
fn account_visibility_does_not_imply_account_access_management() {
    let fixture = ordinary_read_world("read-account-authority", 0);
    let viewer = fixture.authenticate(VIEWER);
    let owner = fixture.authenticate(OWNER);
    let stranger = fixture.authenticate(STRANGER);

    assert!(fixture
        .world
        .runtime
        .query(queries::account_detail(fixture.personal_account))
        .as_principal(&viewer)
        .controls(controls(8))
        .execute()
        .is_ok());
    assert!(fixture
        .world
        .runtime
        .query(queries::account_authorized_users(fixture.personal_account))
        .as_principal(&viewer)
        .controls(controls(8))
        .execute()
        .is_err());
    let users_result = fixture
        .world
        .runtime
        .query(queries::account_authorized_users(fixture.personal_account))
        .as_principal(&owner)
        .controls(controls(8))
        .execute()
        .expect("owner should read account authorizations");
    let [users] = users_result.rows() else {
        panic!("authorized users query must return one account row");
    };
    assert!(users
        .users()
        .iter()
        .any(|user| user.principal().get() == u64::try_from(VIEWER).unwrap() + 1));
    assert!(users_result
        .receipt()
        .inspect()
        .terminal_resources_released());
    assert!(fixture
        .world
        .runtime
        .query(queries::account_detail(fixture.personal_account))
        .as_principal(&stranger)
        .controls(controls(8))
        .execute()
        .is_err());
    assert!(fixture
        .world
        .runtime
        .query(queries::account_detail(fixture.recipient_account))
        .as_principal(&owner)
        .controls(controls(8))
        .execute()
        .is_err());
}

#[test]
fn payment_and_audit_reads_preserve_distinct_authority_paths() {
    let fixture = ordinary_read_world("read-payment-authority", 0);
    let approver = fixture.authenticate(APPROVER);
    let owner = fixture.authenticate(OWNER);
    let auditor = fixture.authenticate(AUDITOR);
    let teller = fixture.authenticate(TELLER);

    let pending = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(&approver)
        .controls(controls(8))
        .execute()
        .expect("approver should discover approval-required payments");
    assert_eq!(pending.rows(), [payment_summary(&fixture)]);
    assert!(pending.receipt().inspect().terminal_resources_released());
    let owner_pending = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(&owner)
        .controls(controls(8))
        .execute()
        .expect("non-approver visibility should yield an empty result");
    assert!(owner_pending.rows().is_empty());
    let payment = fixture
        .world
        .runtime
        .query(queries::payment(fixture.payment))
        .as_principal(&owner)
        .controls(controls(1))
        .execute()
        .expect("payment participant should read payment detail");
    assert_eq!(payment.rows(), [payment_summary(&fixture)]);
    assert!(payment.receipt().inspect().terminal_resources_released());

    let audit = fixture
        .world
        .runtime
        .query(queries::institution_audit(fixture.institution))
        .as_principal(&auditor)
        .controls(controls(16))
        .execute()
        .expect("an institution auditor should execute the installed audit query");
    assert_eq!(audit.rows()[0].institution(), fixture.institution);
    assert!(audit.rows()[0]
        .accounts()
        .iter()
        .any(|account| !account.entries().is_empty()));
    assert!(audit.receipt().inspect().terminal_resources_released());
    assert!(matches!(
        fixture
            .world
            .runtime
            .query(queries::institution_audit(fixture.institution))
            .as_principal(&teller)
            .controls(controls(16))
            .execute(),
        Err(BankApplicationQueryDenial::Admission(_))
    ));
}

#[test]
fn cancellation_and_deadline_are_typed_before_projection() {
    let fixture = ordinary_read_world("read-interruption", 0);
    let owner = fixture.authenticate(OWNER);
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let cancelled = BankReadControls::current(
        WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(60),
            cancellation.token(),
        ),
        8,
        10_000,
    )
    .unwrap();
    assert!(matches!(
        fixture
            .world
            .runtime
            .query(queries::accounts())
            .as_principal(&owner)
            .controls(cancelled)
            .execute(),
        Err(BankApplicationQueryDenial::PrincipalResolution(
            WorthQueryPrincipalResolutionDenialKind::Cancelled
        ))
    ));

    let deadline_source = WorthQueryCancellationSource::new();
    let expired = BankReadControls::current(
        WorthQueryRequestScope::new(Instant::now(), deadline_source.token()),
        8,
        10_000,
    )
    .unwrap();
    assert!(matches!(
        fixture
            .world
            .runtime
            .query(queries::accounts())
            .as_principal(&owner)
            .controls(expired)
            .execute(),
        Err(BankApplicationQueryDenial::PrincipalResolution(
            WorthQueryPrincipalResolutionDenialKind::DeadlineExceeded
        ))
    ));
}

fn payment_summary(
    fixture: &super::fixture::OrdinaryReadFixture,
) -> bank_domain::reads::PaymentSummary {
    let approver = fixture.authenticate(APPROVER);
    let result = fixture
        .world
        .runtime
        .query(queries::payment(fixture.payment))
        .as_principal(&approver)
        .controls(controls(1))
        .execute()
        .expect("approver should read payment detail");
    let [payment] = result.rows() else {
        panic!("payment detail must return exactly one row")
    };
    *payment
}

fn controls(maximum_results: usize) -> BankReadControls {
    BankReadControls::current(request_scope(), maximum_results, 10_000).unwrap()
}
