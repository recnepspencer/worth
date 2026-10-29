pub use bank_domain::accounting::account_balance;
pub use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, InstitutionId, Money,
};
pub use bank_domain::proposals::{
    BankIdempotencyKey, BankOperationScopeBinding, BankProposalDenial, BankProposalEngine,
    BankSnapshot, BankSnapshotBuilder,
};
pub use bank_domain::schema::{
    AccountStatus, ApplyOpeningFunding, CreatePersonalAccount, Deposit, SendMoney, Withdraw,
};

#[path = "accounting_proposals/reversal.rs"]
pub mod reversal;

pub fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).expect("test identity is nonzero")
}

pub fn fixture() -> BankSnapshot {
    BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
        .institution(id(InstitutionId::new, 1))
        .principal(id(BankPrincipalId::new, 1))
        .principal(id(BankPrincipalId::new, 2))
        .institution_cash_account(id(AccountId::new, 100), id(InstitutionId::new, 1))
        .build()
        .expect("fixture topology is valid")
}

pub fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).expect("test key is valid")
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

pub fn create_personal(
    snapshot: &BankSnapshot,
    owner: u64,
    name: &str,
    key_value: &str,
) -> BankSnapshot {
    BankProposalEngine::prepare_create_personal_account(
        snapshot,
        binding(1),
        &key(key_value),
        &CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, owner),
            display_name: AccountName::new(name).expect("test account name is valid"),
        },
    )
    .expect("account proposal is valid")
    .proposed_snapshot()
    .clone()
}

pub fn oracle_balance(snapshot: &BankSnapshot, account: AccountId) -> i64 {
    snapshot
        .journal()
        .iter()
        .flat_map(|entry| entry.postings())
        .filter(|posting| posting.account() == account)
        .map(|posting| posting.amount().minor_units())
        .sum()
}
