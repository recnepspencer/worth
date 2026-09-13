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
use bank_server::{mutations, queries, BankMutationControls, BankMutationStatus, BankReadControls};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedApplicationCommitKind;
use worth_query_host::facade::publication::domain_computation::WorthQueryPublishedApplicationCommitAttemptReleasePosture;

use assertions::{assert_committed, assert_emitting_commit};
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

#[test]
fn public_consumer_executes_every_typed_mutation_family() {
    let fixture = ordinary_read_world("ordinary-mutations", 0);
    let owner = fixture.authenticate(OWNER);
    let recipient = fixture.authenticate(RECIPIENT);
    let approver = fixture.authenticate(APPROVER);
    let teller = fixture.authenticate(TELLER);
    let stranger = fixture.authenticate(STRANGER);
    let discovery = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&stranger)
        .controls(read_controls())
        .execute()
        .expect("the prospective account owner must be discoverable");
    assert!(discovery.rows().is_empty());

    assert_committed(execute!(
        fixture,
        teller,
        mutations::create_personal_account(CreatePersonalAccount {
            institution: fixture.institution,
            owner: principal_id(STRANGER),
            display_name: AccountName::new("Stranger account").unwrap(),
        }),
        "create-personal",
    ));
    let created_accounts = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&stranger)
        .controls(read_controls())
        .execute()
        .expect("the created account must be query-visible");
    let created_personal = created_accounts.rows()[0].id();
    assert_committed(execute!(
        fixture,
        teller,
        mutations::create_business_account(CreateBusinessAccount {
            institution: fixture.institution,
            business: id(BusinessId::new, 2),
            display_name: AccountName::new("Second business").unwrap(),
        }),
        "create-business",
    ));
    assert_emitting_commit(execute!(
        fixture,
        teller,
        mutations::apply_opening_funding(ApplyOpeningFunding {
            institution: fixture.institution,
            account: created_personal,
            amount: Money::from_minor(3_000).unwrap(),
        }),
        "opening-funding",
    ));
    assert_emitting_commit(execute!(
        fixture,
        teller,
        mutations::deposit(Deposit {
            institution: fixture.institution,
            account: fixture.recipient_account,
            amount: Money::from_minor(500).unwrap(),
        }),
        "deposit",
    ));
    assert_emitting_commit(execute!(
        fixture,
        teller,
        mutations::withdraw(Withdraw {
            institution: fixture.institution,
            account: fixture.recipient_account,
            amount: Money::from_minor(100).unwrap(),
        }),
        "withdraw",
    ));
    let send = SendMoney {
        from: fixture.personal_account,
        recipient: principal_id(RECIPIENT),
        amount: Money::from_minor(250).unwrap(),
    };
    assert_emitting_commit(execute!(
        fixture,
        owner,
        mutations::send_money(send.clone()),
        "send",
    ));
    let retry = execute!(fixture, owner, mutations::send_money(send), "send");
    let BankMutationStatus::AlreadyCommitted(receipt) = retry.status() else {
        panic!("equivalent public retry must recover the commit");
    };
    let publication = receipt.publication().inspect();
    assert_eq!(
        publication.kind(),
        WorthQueryPublishedApplicationCommitKind::Recovered
    );
    assert_eq!(
        publication.attempt_release(),
        WorthQueryPublishedApplicationCommitAttemptReleasePosture::NotAttempted
    );

    let initiation = execute!(
        fixture,
        owner,
        mutations::initiate_business_payment(InitiateBusinessPayment {
            business: id(BusinessId::new, 1),
            from: fixture.business_account,
            recipient: principal_id(RECIPIENT),
            amount: Money::from_minor(300).unwrap(),
        }),
        "initiate",
    );
    assert!(matches!(
        initiation.status(),
        BankMutationStatus::Committed(_)
    ));
    assert!(initiation.continuation().is_some());
    let available = account_activity(&fixture, &approver, fixture.business_account)
        .into_iter()
        .map(|item| item.amount().minor_units())
        .sum::<i64>();
    assert!(available >= 900);
    let approval = execute!(
        fixture,
        approver,
        mutations::approve_payment(ApprovePayment {
            payment: fixture.payment,
            approver: principal_id(APPROVER),
        }),
        "approve",
    );
    assert!(
        matches!(approval.status(), BankMutationStatus::Committed(_)),
        "approval={approval:?}, business={:?}, created={created_personal:?}, recipient={:?}, personal={:?}",
        fixture.business_account,
        fixture.recipient_account,
        fixture.personal_account,
    );
    assert_emitting_commit(approval);
    let pending = pending_payments(&fixture, &approver);
    assert!(pending
        .iter()
        .all(|payment| payment.id() != fixture.payment));
    let pending = pending
        .iter()
        .find(|payment| payment.id() != fixture.payment)
        .expect("the newly initiated payment remains pending");
    assert_committed(execute!(
        fixture,
        approver,
        mutations::reject_payment(RejectPayment {
            payment: pending.id(),
            rejecting_principal: principal_id(APPROVER),
        }),
        "reject",
    ));

    assert_committed(execute!(
        fixture,
        owner,
        mutations::grant_account_access(GrantAccountAuthorization {
            account: fixture.personal_account,
            principal: principal_id(STRANGER),
            role: CustomerRole::Viewer,
        }),
        "grant",
    ));
    let authorization = authorized_users(&fixture, &owner)
        .into_iter()
        .find(|user| user.principal() == principal_id(STRANGER))
        .expect("granted authorization must be query-visible")
        .authorization();
    assert_committed(execute!(
        fixture,
        owner,
        mutations::revoke_account_access(RevokeAccountAuthorization {
            account: fixture.personal_account,
            authorization,
        }),
        "revoke",
    ));

    let journal = account_activity(&fixture, &recipient, fixture.recipient_account)
        .into_iter()
        .find(|item| item.purpose() == PostingPurpose::Deposit)
        .expect("deposit journal must be query-visible")
        .journal();
    assert_emitting_commit(execute!(
        fixture,
        teller,
        mutations::reverse_journal(ReverseJournal {
            institution: id(InstitutionId::new, 1),
            journal,
            reason: ReversalReason::OperatorCorrection,
        }),
        "reverse",
    ));
}

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
    assert!(matches!(cancelled.status(), BankMutationStatus::Cancelled));
    assert_eq!(cancelled.metadata().projection_work(), None);

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
    assert!(matches!(
        expired.status(),
        BankMutationStatus::DeadlineExceeded
    ));

    let denied = execute!(
        fixture,
        stranger,
        mutations::send_money(send.clone()),
        "denied",
    );
    assert!(matches!(denied.status(), BankMutationStatus::Denied(_)));

    assert_emitting_commit(execute!(
        fixture,
        owner,
        mutations::send_money(send),
        "drift",
    ));
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
        drift.status(),
        BankMutationStatus::Denied(bank_server::BankMutationDenial::IdempotencyIntentDrift)
    ));
    assert!(drift.metadata().provider_work_units() > 0);
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
