//! Business-payment authorization fixture construction.

use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, BusinessId, CustomerRole,
    InstitutionId, Money, PaymentId,
};
use bank_domain::proposals::{
    BankIdempotencyKey, BankOperationScopeBinding, BankProposalEngine, BankSnapshot,
    BankSnapshotBuilder,
};
use bank_domain::schema::{
    ApplyOpeningFunding, CreateBusinessAccount, CreatePersonalAccount, GrantAccountAuthorization,
    InitiateBusinessPayment,
};

pub(super) fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).unwrap()
}

pub(super) fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}

pub(super) fn binding(value: u8) -> BankOperationScopeBinding {
    BankOperationScopeBinding::new(
        1,
        bank_domain::proposals::BankOperationScopeSchemaBinding::new(1, 1, [2; 32], [3; 32]),
        "test-operation-authority",
        bank_domain::proposals::BankOperationScopeEntityBinding::new(0, 1, 1),
        bank_domain::proposals::BankOperationScopeEntityBinding::new(0, u64::from(value), 1),
    )
}

pub(super) fn pending_business_payment_world() -> (BankSnapshot, PaymentId) {
    let accounts = create_business_payment_accounts(3);
    let source = accounts.business_account(id(BusinessId::new, 1)).unwrap();
    let other_source = accounts.business_account(id(BusinessId::new, 2)).unwrap();
    let funded = fund_business_account(&accounts, source);
    let initiator_access = grant_access(
        &funded,
        source,
        1,
        CustomerRole::Approver,
        "initiator-approval-role",
    );
    let approver_access = grant_access(
        &initiator_access,
        source,
        3,
        CustomerRole::Approver,
        "approver-role",
    );
    let viewer_access = grant_access(
        &approver_access,
        source,
        2,
        CustomerRole::Viewer,
        "viewer-role",
    );
    let cross_business_access = grant_access(
        &viewer_access,
        other_source,
        2,
        CustomerRole::Approver,
        "cross-business-role",
    );
    initiate_business_payment(&cross_business_access, source)
}

/// A funded business account whose initiator is principal 1 and whose
/// approvers are principals 3 onward; principal 2 is the recipient.
pub(super) fn approver_crowded_world(approvers: u64) -> (BankSnapshot, AccountId) {
    let accounts = create_business_payment_accounts(2 + approvers);
    let source = accounts.business_account(id(BusinessId::new, 1)).unwrap();
    let initiator = grant_access(
        &fund_business_account(&accounts, source),
        source,
        1,
        CustomerRole::Initiator,
        "initiator-role",
    );
    let crowded = (3..3 + approvers).fold(initiator, |snapshot, principal| {
        grant_access(
            &snapshot,
            source,
            principal,
            CustomerRole::Approver,
            &format!("approver-role-{principal}"),
        )
    });
    (crowded, source)
}

fn create_business_payment_accounts(principals: u64) -> BankSnapshot {
    let empty = (1..=principals)
        .fold(
            BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
                .institution(id(InstitutionId::new, 1)),
            |builder, principal| builder.principal(id(BankPrincipalId::new, principal)),
        )
        .business(id(BusinessId::new, 1))
        .business(id(BusinessId::new, 2))
        .institution_cash_account(id(AccountId::new, 100), id(InstitutionId::new, 1))
        .build()
        .unwrap();
    let recipient = BankProposalEngine::prepare_create_personal_account(
        &empty,
        binding(1),
        &key("recipient"),
        &CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, 2),
            display_name: AccountName::new("Recipient").unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let business = BankProposalEngine::prepare_create_business_account(
        &recipient,
        binding(2),
        &key("business"),
        &CreateBusinessAccount {
            institution: id(InstitutionId::new, 1),
            business: id(BusinessId::new, 1),
            display_name: AccountName::new("Operations").unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    BankProposalEngine::prepare_create_business_account(
        &business,
        binding(2),
        &key("second-business"),
        &CreateBusinessAccount {
            institution: id(InstitutionId::new, 1),
            business: id(BusinessId::new, 2),
            display_name: AccountName::new("Other operations").unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

fn fund_business_account(snapshot: &BankSnapshot, source: AccountId) -> BankSnapshot {
    BankProposalEngine::prepare_opening_funding(
        snapshot,
        binding(3),
        &key("fund"),
        &ApplyOpeningFunding {
            institution: id(InstitutionId::new, 1),
            account: source,
            amount: Money::from_minor(50_000).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

fn grant_access(
    snapshot: &BankSnapshot,
    account: AccountId,
    principal: u64,
    role: CustomerRole,
    key_value: &str,
) -> BankSnapshot {
    BankProposalEngine::prepare_grant_account_authorization(
        snapshot,
        binding(5),
        &key(key_value),
        &GrantAccountAuthorization {
            account,
            principal: id(BankPrincipalId::new, principal),
            role,
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

fn initiate_business_payment(
    snapshot: &BankSnapshot,
    source: AccountId,
) -> (BankSnapshot, PaymentId) {
    let pending = BankProposalEngine::prepare_initiate_business_payment(
        snapshot,
        binding(4),
        &key("initiate"),
        id(BankPrincipalId::new, 1),
        &InitiateBusinessPayment {
            business: id(BusinessId::new, 1),
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(8_000).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let payment = pending.payments().next().unwrap().id();
    (pending, payment)
}
