pub use std::time::{Duration, Instant};

pub use bank_server::{queries, BankApplicationQueryDenial, BankReadControls};
pub use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
pub use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};

pub use super::fixture::{ordinary_read_world, APPROVER, AUDITOR, OWNER, STRANGER, TELLER, VIEWER};
pub use crate::facade::server::support::request_scope;

pub fn payment_summary(
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

pub fn controls(maximum_results: usize) -> BankReadControls {
    BankReadControls::current(request_scope(), maximum_results).unwrap()
}
