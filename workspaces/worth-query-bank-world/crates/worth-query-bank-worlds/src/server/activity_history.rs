pub use crate::facade::server::fixture;
#[path = "activity_history/historical.rs"]
pub mod historical;
pub use crate::facade::server::support;

pub use std::num::NonZeroUsize;

pub use bank_domain::model::Money;
pub use bank_domain::proposals::BankIdempotencyKey;
pub use bank_domain::schema::{Deposit, RevokeAccountAuthorization};
pub use bank_server::{
    mutations, queries, BankApplicationQueryAdmissionDenialKind, BankApplicationQueryDenial,
    BankMutationControls, BankReadControls,
};
pub use fixture::{ordinary_read_world, OWNER, TELLER, VIEWER};
pub use support::request_scope;
pub use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryResumeControls;

pub fn page_controls(request: &WorthQueryRequestScope, page_width: usize) -> BankReadControls {
    BankReadControls::current(request.clone(), page_width).unwrap()
}

pub fn resume_controls<'a>(
    request: &'a WorthQueryRequestScope,
    page_width: usize,
) -> WorthQueryApplicationQueryResumeControls<'a> {
    WorthQueryApplicationQueryResumeControls::new(NonZeroUsize::new(page_width).unwrap(), request)
}

pub fn authorized_user_id(
    fixture: &fixture::OrdinaryReadFixture,
    owner: &bank_server::BankAuthenticatedPrincipal,
    principal: usize,
) -> bank_domain::model::AccountAuthorizationId {
    let users = fixture
        .world
        .runtime
        .query(queries::account_authorized_users(fixture.personal_account))
        .as_principal(owner)
        .controls(BankReadControls::current(request_scope(), 16).unwrap())
        .execute()
        .expect("owner should read authorized users");
    let [users] = users.rows() else {
        panic!("authorized users query must return one account row")
    };
    users
        .users()
        .iter()
        .find(|user| user.principal() == fixture::principal_id(principal))
        .expect("fixture authorization should exist")
        .authorization()
}
