mod support;

use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, EmployeeAssignmentId,
    EmployeeRole, InstitutionId,
};
use bank_domain::proposals::{BankIdempotencyKey, BankSnapshotBuilder};
use bank_domain::queries;
use bank_domain::schema::{
    Account, CreatePersonalAccount, CreatePersonalAccountMutationBinding,
    CREATE_PERSONAL_ACCOUNT_OUTPUT_ACCOUNT,
};
use bank_server::{BankEmployeeAssignmentSeed, BankPrincipalSeed, BankWorldSeed};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
};

use support::{block_on, request_scope, runtime, CausalCredential, DynamicIdentity};

#[test]
fn public_request_creates_reads_and_idempotently_recovers_a_personal_account() {
    assert_public_creation("Everyday account");
    assert_public_creation(&"a".repeat(AccountName::MAX_BYTES));
    assert!(AccountName::new("a".repeat(AccountName::MAX_BYTES + 1)).is_err());
}

fn assert_public_creation(display_name: &str) {
    let actor_identity = DynamicIdentity::new("public-create-personal-account");
    let institution = id(InstitutionId::new, 1);
    let actor_id = id(BankPrincipalId::new, 1);
    let snapshot = BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
        .institution(institution)
        .principal(actor_id)
        .institution_cash_account(id(AccountId::new, 100), institution)
        .build()
        .expect("the account-creation world should be valid");
    let world = runtime(
        BankWorldSeed::new(snapshot)
            .principal(BankPrincipalSeed::enabled(
                actor_id,
                actor_identity.external(),
            ))
            .employee(BankEmployeeAssignmentSeed::new(
                id(EmployeeAssignmentId::new, 1),
                institution,
                actor_id,
                EmployeeRole::Teller,
            )),
    );
    let scope = request_scope();
    let actor = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(&actor_identity),
        &scope,
    ))
    .expect("the teller should authenticate");
    let request = world.runtime.request(&actor, &scope);
    let key = BankIdempotencyKey::new("public-create-personal-account")
        .expect("the client key should be valid");
    let input = CreatePersonalAccount {
        institution,
        owner: actor_id,
        display_name: AccountName::new(display_name).expect("the name should be valid"),
    };
    let output_role = WorthQueryApplicationOutputRole::<
        CreatePersonalAccountMutationBinding,
        Account,
        WorthQueryCreateOutput,
    >::new(CREATE_PERSONAL_ACCOUNT_OUTPUT_ACCOUNT);

    let first = request
        .mutate(input.clone())
        .idempotency(&key)
        .execute()
        .expect("the public mutation request should execute");
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: mut first_receipt,
        result,
    } = first
    else {
        panic!("the first mutation must publish an actual World commit");
    };
    let account = result.account;
    assert!(account.canonical_text().starts_with("operation:"));
    assert_eq!(account.canonical_text().len(), 76);
    let created_entity = first_receipt
        .output_correspondence()
        .entity(output_role)
        .expect("the declared created-account output must resolve")
        .entity_id();
    assert!(first_receipt
        .take_performed_relational_product_change()
        .is_some());

    let detail = request
        .query(queries::account_detail(account))
        .execute()
        .expect("the new account should be readable on the same request");
    let [detail] = detail.rows() else {
        panic!("account detail must return the created account");
    };
    assert_eq!(detail.summary().id(), account);
    assert_eq!(detail.summary().display_name(), &input.display_name);
    assert_eq!(detail.institution(), institution);
    assert_eq!(detail.personal_owner(), Some(actor_id));

    let retry = request
        .mutate(input.clone())
        .idempotency(&key)
        .execute()
        .expect("the identical retry should execute");
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(mut retry_receipt) = retry else {
        panic!("the identical retry must recover the prior commit");
    };
    let recovered_entity = retry_receipt
        .output_correspondence()
        .entity(output_role)
        .expect("the recovered output correspondence must remain complete")
        .entity_id();
    assert_eq!(recovered_entity, created_entity);
    assert!(retry_receipt.is_same_authoritative_commit(&first_receipt));
    assert!(retry_receipt
        .take_performed_relational_product_change()
        .is_none());

    let changed = request
        .mutate(CreatePersonalAccount {
            display_name: AccountName::new("Changed account").expect("the name should be valid"),
            ..input
        })
        .idempotency(&key)
        .execute()
        .expect("the changed retry should resolve idempotency");
    assert!(matches!(
        changed,
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
    ));
}

fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).expect("the fixture identity should be valid")
}
