pub use crate::facade::server::authorization_time;
pub use crate::facade::server::estate_operations;
pub use crate::facade::server::fixture;
pub use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
#[path = "ordinary_mutations/preconditions.rs"]
pub mod preconditions;
#[path = "ordinary_mutations/publication.rs"]
pub mod publication;
pub use crate::facade::server::support;

pub use std::time::{Duration, Instant};

pub use bank_domain::model::{AccountName, BusinessId, CustomerRole, InstitutionId, Money};
pub use bank_domain::proposals::BankIdempotencyKey;
pub use bank_domain::schema::{
    ApplyOpeningFunding, ApprovePayment, CreateBusinessAccount, CreatePersonalAccount, Deposit,
    GrantAccountAuthorization, InitiateBusinessPayment, PostingPurpose, RejectPayment,
    ReversalReason, ReverseJournal, RevokeAccountAuthorization, SendMoney, Withdraw,
};
pub use bank_server::{mutations, queries, BankMutationControls, BankReadControls};
pub use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
pub use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
};
pub use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitTerminalKind;

pub use fixture::{
    ordinary_read_world, principal_id, APPROVER, OWNER, RECIPIENT, STRANGER, TELLER,
};
pub use support::request_scope;

#[path = "ordinary_mutations/money_families.rs"]
pub mod money_families;
#[path = "ordinary_mutations/owning_deposit.rs"]
pub mod owning_deposit;
#[path = "ordinary_mutations/program_families.rs"]
pub mod program_families;
#[path = "ordinary_mutations/recorded_correction.rs"]
pub mod recorded_correction;
#[path = "ordinary_mutations/retained_publication.rs"]
pub mod retained_publication;
#[path = "ordinary_mutations/transfer_posting_oracle.rs"]
pub mod transfer_posting_oracle;

pub fn pending_payments(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
) -> Vec<bank_domain::reads::PaymentSummary> {
    let result = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(principal)
        .controls(read_controls())
        .execute()
        .expect("pending payments must be readable");
    result.rows().to_vec()
}

pub fn authorized_users(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
) -> Vec<bank_domain::reads::AuthorizedAccountUser> {
    let result = fixture
        .world
        .runtime
        .query(queries::account_authorized_users(fixture.personal_account))
        .as_principal(principal)
        .controls(read_controls())
        .execute()
        .expect("authorized users must be readable");
    let mut rows = result.into_rows();
    assert_eq!(rows.len(), 1);
    rows.pop().unwrap().into_users()
}

pub fn account_activity(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
    account: bank_domain::model::AccountId,
) -> Vec<bank_domain::reads::AccountActivityItem> {
    let request = request_scope();
    fixture
        .world
        .runtime
        .account_activity(account)
        .as_principal(principal)
        .execute(BankReadControls::current(request, 128).unwrap())
        .expect("account activity must be readable")
        .rows()[0]
        .entries()
        .to_vec()
}

pub fn read_controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 128).unwrap()
}

pub fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}

pub fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).unwrap()
}

pub fn send_for(fixture: &fixture::OrdinaryReadFixture) -> SendMoney {
    SendMoney {
        from: fixture.personal_account,
        recipient: principal_id(RECIPIENT),
        amount: Money::from_minor(25).unwrap(),
    }
}

#[path = "ordinary_mutations/posting_lease.rs"]
pub mod posting_lease;

#[path = "ordinary_mutations/owning_accounts.rs"]
pub mod owning_accounts;
