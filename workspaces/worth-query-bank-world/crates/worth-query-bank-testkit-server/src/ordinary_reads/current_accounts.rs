//! Actual Bank fixtures and receiver reads for managed current accounts.
pub use super::fixture::{
    ordinary_read_world, over_budget_discovery_world, principal_id, OrdinaryReadFixture, APPROVER,
    AUDITOR, OWNER, RECIPIENT, STRANGER, TELLER, VIEWER,
};
pub use crate::facade::support::request_scope;
pub use bank_domain::{
    model::*,
    schema::{
        CreateBusinessAccount, CreatePersonalAccount, GrantAccountAuthorization,
        RevokeAccountAuthorization,
    },
};
pub use bank_server::{
    mutations, queries, BankApplicationQueryDenial, BankAuthenticatedPrincipal,
    BankMutationControls, BankReadControls,
};
pub use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::*,
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationOneShotDenialKind,
        WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryManagedIndexDenialCause,
    },
};
pub fn controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 64).unwrap()
}
pub fn ordinary(
    fixture: &OrdinaryReadFixture,
    actor: &BankAuthenticatedPrincipal,
) -> Vec<bank_domain::reads::VisibleAccount> {
    fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(actor)
        .controls(controls())
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}
pub fn managed(
    fixture: &OrdinaryReadFixture,
    actor: &BankAuthenticatedPrincipal,
) -> Vec<bank_domain::reads::VisibleAccount> {
    fixture
        .world
        .runtime
        .query(queries::current_accounts())
        .as_principal(actor)
        .controls(controls())
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}
pub fn authorization(
    fixture: &OrdinaryReadFixture,
    owner: &BankAuthenticatedPrincipal,
    account: AccountId,
    principal: usize,
) -> AccountAuthorizationId {
    let rows = fixture
        .world
        .runtime
        .query(queries::account_authorized_users(account))
        .as_principal(owner)
        .controls(controls())
        .execute()
        .unwrap();
    rows.rows()[0]
        .users()
        .iter()
        .find(|user| user.principal() == principal_id(principal))
        .unwrap()
        .authorization()
}

mod business_creation;
pub use business_creation::business_creation_world;
