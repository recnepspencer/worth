pub use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, BusinessId, CustomerRole,
    InstitutionId, Money,
};
pub use bank_domain::proposals::{
    BankIdempotencyKey, BankOperationScopeBinding, BankProposalDenial, BankProposalEngine,
    BankSnapshot, BankSnapshotBuilder,
};
pub use bank_domain::schema::{
    initiate_business_payment_application_idempotency, ApplyOpeningFunding, ApprovePayment,
    CreateBusinessAccount, CreatePersonalAccount, GrantAccountAuthorization,
    InitiateBusinessPayment, PaymentStatus, RejectPayment, RevokeAccountAuthorization,
    MAX_PAYMENT_APPROVAL_GRANTEES,
};

pub fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).expect("test identity is nonzero")
}

pub fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}

pub fn binding(value: u8) -> BankOperationScopeBinding {
    BankOperationScopeBinding::new(
        1,
        bank_domain::proposals::BankOperationScopeSchemaBinding::new(1, 1, [2; 32], [3; 32]),
        "test-operation-authority",
        bank_domain::proposals::BankOperationScopeEntityBinding::new(0, 1, 1),
        bank_domain::proposals::BankOperationScopeEntityBinding::new(0, u64::from(value), 1),
    )
}

pub fn fixture() -> BankSnapshot {
    (1..=12)
        .fold(
            BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
                .institution(id(InstitutionId::new, 1)),
            |builder, principal| builder.principal(id(BankPrincipalId::new, principal)),
        )
        .business(id(BusinessId::new, 1))
        .institution_cash_account(id(AccountId::new, 100), id(InstitutionId::new, 1))
        .build()
        .unwrap()
}

pub fn prepared_business_world() -> BankSnapshot {
    let personal = BankProposalEngine::prepare_create_personal_account(
        &fixture(),
        binding(1),
        &key("personal"),
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
        &personal,
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
    let account = business.business_account(id(BusinessId::new, 1)).unwrap();
    BankProposalEngine::prepare_opening_funding(
        &business,
        binding(3),
        &key("fund-business"),
        &ApplyOpeningFunding {
            institution: id(InstitutionId::new, 1),
            account,
            amount: Money::from_minor(20_000).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

pub fn with_authorizations(
    world: BankSnapshot,
    grants: impl IntoIterator<Item = (u64, CustomerRole)>,
) -> BankSnapshot {
    let account = world.business_account(id(BusinessId::new, 1)).unwrap();
    grants.into_iter().fold(world, |world, (principal, role)| {
        BankProposalEngine::prepare_grant_account_authorization(
            &world,
            binding(u8::try_from(principal).unwrap()),
            &key(&format!("grant-{principal}")),
            &GrantAccountAuthorization {
                account,
                principal: id(BankPrincipalId::new, principal),
                role,
            },
        )
        .unwrap()
        .proposed_snapshot()
        .clone()
    })
}

pub fn decide_initiation(
    world: &BankSnapshot,
) -> Result<bank_domain::schema::InitiateBusinessPaymentDecision, BankProposalDenial> {
    let input = InitiateBusinessPayment {
        business: id(BusinessId::new, 1),
        from: world.business_account(id(BusinessId::new, 1)).unwrap(),
        recipient: id(BankPrincipalId::new, 2),
        amount: Money::from_minor(5_000).unwrap(),
    };
    BankProposalEngine::decide_initiate_business_payment_from_application(
        world,
        initiate_business_payment_application_idempotency(binding(40), &key("initiate"), &input),
        id(BankPrincipalId::new, 1),
        &input,
    )
}
