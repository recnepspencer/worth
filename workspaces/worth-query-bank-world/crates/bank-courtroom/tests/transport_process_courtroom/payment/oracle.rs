//! Independent Bank account and journal observations for the payment court.

use bank_domain::model::AccountId;
use bank_domain::queries::account_summary;
use bank_domain::reads::AccountActivityItem;
use bank_domain::schema::PostingPurpose;
use bank_server::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankReadControls};

use crate::support;

const PAYMENT_MINOR_UNITS: i64 = 900;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct PaymentAccounts {
    source_balance: i64,
    destination_balance: i64,
    source_entries: Vec<AccountActivityItem>,
    destination_entries: Vec<AccountActivityItem>,
}

pub(super) fn snapshot(
    runtime: &BankIdentityRuntime,
    approver: &BankAuthenticatedPrincipal,
    recipient: &BankAuthenticatedPrincipal,
    source: AccountId,
    destination: AccountId,
) -> PaymentAccounts {
    let controls = || BankReadControls::current(support::request_scope(), 128, 10_000).unwrap();
    let source_balance = runtime
        .query(account_summary(source))
        .as_principal(approver)
        .controls(controls())
        .execute()
        .expect("source account readback succeeds")
        .into_rows()[0]
        .current_balance()
        .minor_units();
    let destination_balance = runtime
        .query(account_summary(destination))
        .as_principal(recipient)
        .controls(controls())
        .execute()
        .expect("destination account readback succeeds")
        .into_rows()[0]
        .current_balance()
        .minor_units();
    let source_entries = runtime
        .account_activity(source)
        .as_principal(approver)
        .execute(controls())
        .expect("source activity readback succeeds")
        .rows()[0]
        .entries()
        .to_vec();
    let destination_entries = runtime
        .account_activity(destination)
        .as_principal(recipient)
        .execute(controls())
        .expect("destination activity readback succeeds")
        .rows()[0]
        .entries()
        .to_vec();
    PaymentAccounts {
        source_balance,
        destination_balance,
        source_entries,
        destination_entries,
    }
}

pub(super) fn assert_one_payment(before: &PaymentAccounts, after: &PaymentAccounts) {
    assert_eq!(
        after.source_balance,
        before.source_balance - PAYMENT_MINOR_UNITS
    );
    assert_eq!(
        after.destination_balance,
        before.destination_balance + PAYMENT_MINOR_UNITS
    );
    assert_eq!(after.source_entries.len(), before.source_entries.len() + 1);
    assert_eq!(
        after.destination_entries.len(),
        before.destination_entries.len() + 1
    );
    assert_eq!(
        &after.source_entries[..before.source_entries.len()],
        before.source_entries.as_slice()
    );
    assert_eq!(
        &after.destination_entries[..before.destination_entries.len()],
        before.destination_entries.as_slice()
    );
    let source = after.source_entries.last().unwrap();
    let destination = after.destination_entries.last().unwrap();
    assert_eq!(source.purpose(), PostingPurpose::Transfer);
    assert_eq!(destination.purpose(), PostingPurpose::Transfer);
    assert_eq!(source.amount().minor_units(), -PAYMENT_MINOR_UNITS);
    assert_eq!(destination.amount().minor_units(), PAYMENT_MINOR_UNITS);
    assert_eq!(source.journal(), destination.journal());
}
