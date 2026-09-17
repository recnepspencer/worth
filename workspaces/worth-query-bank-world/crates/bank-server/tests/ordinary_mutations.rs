#[path = "ordinary_mutations/assertions.rs"]
mod assertions;
#[path = "ordinary_mutations/authorization_time.rs"]
mod authorization_time;
#[path = "ordinary_mutations/estate_operations.rs"]
mod estate_operations;
#[allow(
    dead_code,
    reason = "the shared read fixture has discovery-only helpers exercised by its owning test binary"
)]
#[path = "ordinary_reads/fixture.rs"]
mod fixture;
#[path = "ordinary_mutations/preconditions.rs"]
mod preconditions;
#[path = "ordinary_mutations/publication.rs"]
mod publication;
mod support;

use std::time::{Duration, Instant};

use bank_domain::model::{AccountName, BusinessId, CustomerRole, InstitutionId, Money};
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{
    ApplyOpeningFunding, ApprovePayment, CreateBusinessAccount, CreatePersonalAccount, Deposit,
    GrantAccountAuthorization, InitiateBusinessPayment, PostingPurpose, RejectPayment,
    ReversalReason, ReverseJournal, RevokeAccountAuthorization, SendMoney, Withdraw,
};
use bank_server::{mutations, queries, BankMutationControls, BankReadControls};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitTerminalKind, WorthQueryPrincipalResolutionDenialKind,
};

use assertions::assert_program_committed;
use fixture::{ordinary_read_world, principal_id, APPROVER, OWNER, RECIPIENT, STRANGER, TELLER};
use support::request_scope;

macro_rules! execute {
    ($fixture:expr, $principal:expr, $mutation:expr, $idempotency:expr $(,)?) => {
        $fixture
            .world
            .runtime
            .mutate($mutation)
            .as_principal(&$principal)
            .controls(BankMutationControls::new(
                request_scope(),
                key($idempotency),
            ))
            .execute()
    };
}

#[path = "ordinary_mutations/program_families.rs"]
mod program_families;
#[path = "ordinary_mutations/retained_publication.rs"]
mod retained_publication;
#[path = "ordinary_mutations/transfer_posting_oracle.rs"]
mod transfer_posting_oracle;

#[test]
fn public_mutation_controls_preserve_interruptions_permissions_and_intent_drift() {
    let fixture = ordinary_read_world("ordinary-mutation-controls", 0);
    let owner = fixture.authenticate(OWNER);
    let stranger = fixture.authenticate(STRANGER);
    let send = SendMoney {
        from: fixture.personal_account,
        recipient: principal_id(RECIPIENT),
        amount: Money::from_minor(25).unwrap(),
    };

    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let cancelled = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send.clone()))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            WorthQueryRequestScope::new(
                Instant::now() + Duration::from_secs(60),
                cancellation.token(),
            ),
            key("cancelled"),
        ))
        .execute();
    assert!(
        matches!(
            cancelled,
            Err(WorthQueryApplicationRequestMutationDenial::PrincipalResolution(ref denial))
                if denial.kind() == WorthQueryPrincipalResolutionDenialKind::Cancelled
        ),
        "unexpected cancelled outcome: {cancelled:?}"
    );

    let deadline = WorthQueryCancellationSource::new();
    let expired = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send.clone()))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            WorthQueryRequestScope::new(Instant::now() - Duration::from_secs(1), deadline.token()),
            key("expired"),
        ))
        .execute();
    assert!(
        matches!(
            expired,
            Err(WorthQueryApplicationRequestMutationDenial::PrincipalResolution(ref denial))
                if denial.kind() == WorthQueryPrincipalResolutionDenialKind::DeadlineExceeded
        ),
        "unexpected deadline outcome: {expired:?}"
    );

    let denied = execute!(
        fixture,
        stranger,
        mutations::send_money(send.clone()),
        "denied",
    );
    assert!(matches!(
        denied,
        Err(ref denial)
            if denial.kind()
                == worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::Authorization
    ));

    assert_program_committed::<SendMoney>(
        execute!(fixture, owner, mutations::send_money(send), "drift"),
        true,
    );
    let drift = execute!(
        fixture,
        owner,
        mutations::send_money(SendMoney {
            amount: Money::from_minor(26).unwrap(),
            ..send_for(&fixture)
        }),
        "drift",
    );
    assert!(matches!(
        drift,
        Ok(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
    ));
}

fn pending_payments(
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

fn authorized_users(
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

fn account_activity(
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
        .execute(BankReadControls::current(request, 128, 8_192).unwrap())
        .expect("account activity must be readable")
        .rows()[0]
        .entries()
        .to_vec()
}

fn read_controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 128, 10_000).unwrap()
}

fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}

fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).unwrap()
}

fn send_for(fixture: &fixture::OrdinaryReadFixture) -> SendMoney {
    SendMoney {
        from: fixture.personal_account,
        recipient: principal_id(RECIPIENT),
        amount: Money::from_minor(25).unwrap(),
    }
}
