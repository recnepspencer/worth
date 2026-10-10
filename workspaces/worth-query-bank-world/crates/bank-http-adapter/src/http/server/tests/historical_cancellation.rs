//! The Bank historical read retains master's HTTP cancellation and remedy.
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use bank_domain::model::{AccountId, InstitutionId, Money};
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::Deposit;
use bank_server::{mutations, BankMutationControls};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

use super::super::super::protocol::{BankHttpDenialKind, BankHttpNextAction};
use super::fixture::application;

#[tokio::test]
async fn historical_account_activity_cancellation_preserves_http_class_and_remedy() {
    let account = AccountId::new(100).unwrap();
    let application = application(account);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let principal = application.authenticate(&scope).await;
    let committed = application
        .runtime
        .mutate(mutations::deposit(Deposit {
            institution: InstitutionId::new(1).unwrap(),
            account,
            amount: Money::from_minor(1).unwrap(),
        }))
        .as_principal(&principal)
        .controls(BankMutationControls::new(
            scope.clone(),
            BankIdempotencyKey::new("historical-http-cancellation").unwrap(),
        ))
        .execute()
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = committed else {
        panic!("fixture deposit must commit: {committed:?}");
    };
    let read = || {
        application
            .runtime
            .account_activity(account)
            .as_principal(&principal)
            .historical(
                &receipt,
                NonZeroUsize::new(16).unwrap(),
                NonZeroUsize::new(20_000).unwrap(),
                &scope,
            )
    };
    assert!(
        read().is_ok(),
        "the exact commit is readable before cancellation"
    );
    cancellation.cancel();
    let denial = read()
        .err()
        .expect("the canceled historical endpoint must refuse");
    let http = super::super::query_denial::query_denial(denial);
    // Master maps the canceled retained-basis selection to Cancelled / Retry.
    assert_eq!(http.kind, BankHttpDenialKind::Cancelled);
    assert_eq!(http.next_action, BankHttpNextAction::Retry);
}
