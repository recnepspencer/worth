pub use std::num::NonZeroUsize;

pub use bank_domain::model::{CustomerRole, Money};
pub use bank_domain::proposals::BankIdempotencyKey;
pub use bank_domain::schema::{Deposit, GrantAccountAuthorization, RevokeAccountAuthorization};
pub use bank_server::{
    mutations, BankApplicationQueryAdmissionDenialKind, BankApplicationQueryDenial,
    BankMutationControls,
};

pub use super::fixture::{
    ordinary_read_world, OrdinaryReadFixture, OWNER, STRANGER, TELLER, VIEWER,
};
pub use super::support::request_scope;

pub fn commit_deposit(
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
