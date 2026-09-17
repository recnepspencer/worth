use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, BusinessId, InstitutionId, Money,
};
use bank_domain::proposals::{
    BankIdempotencyKey, BankOperationScopeBinding, BankProposalEngine, BankSnapshot,
    BankSnapshotBuilder,
};
use bank_domain::schema::{ApplyOpeningFunding, CreatePersonalAccount};

pub(super) fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).expect("test identity is nonzero")
}

pub(super) fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}

pub(super) fn descriptive_binding(value: u8) -> BankOperationScopeBinding {
    BankOperationScopeBinding::new(
        1,
        bank_domain::proposals::BankOperationScopeSchemaBinding::new(1, 1, [2; 32], [3; 32]),
        "test-operation-authority",
        bank_domain::proposals::BankOperationScopeEntityBinding::new(0, 1, 1),
        bank_domain::proposals::BankOperationScopeEntityBinding::new(0, u64::from(value), 1),
    )
}

pub(super) fn funded_personal_world() -> BankSnapshot {
    funded_personal_world_with_unrelated_journals(0)
}

pub(super) fn funded_independent_sender_world() -> BankSnapshot {
    let mut snapshot = BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
        .institution(id(InstitutionId::new, 1))
        .principal(id(BankPrincipalId::new, 1))
        .principal(id(BankPrincipalId::new, 2))
        .principal(id(BankPrincipalId::new, 3))
        .principal(id(BankPrincipalId::new, 4))
        .institution_cash_account(id(AccountId::new, 100), id(InstitutionId::new, 1))
        .build()
        .unwrap();
    for principal in 1..=4 {
        let name = format!("Independent {principal}");
        snapshot = BankProposalEngine::prepare_create_personal_account(
            &snapshot,
            descriptive_binding(5),
            &key(&format!("independent-create-{principal}")),
            &CreatePersonalAccount {
                institution: id(InstitutionId::new, 1),
                owner: id(BankPrincipalId::new, principal),
                display_name: AccountName::new(&name).unwrap(),
            },
        )
        .unwrap()
        .proposed_snapshot()
        .clone();
    }
    for principal in [1, 2] {
        let account = snapshot
            .primary_account(id(BankPrincipalId::new, principal))
            .unwrap();
        snapshot = BankProposalEngine::prepare_opening_funding(
            &snapshot,
            descriptive_binding(6),
            &key(&format!("independent-fund-{principal}")),
            &ApplyOpeningFunding {
                institution: id(InstitutionId::new, 1),
                account,
                amount: Money::from_minor(10_000).unwrap(),
            },
        )
        .unwrap()
        .proposed_snapshot()
        .clone();
    }
    snapshot
}

pub(super) fn funded_personal_world_with_unrelated_journals(
    unrelated_journals: usize,
) -> BankSnapshot {
    let mut builder = BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
        .institution(id(InstitutionId::new, 1))
        .principal(id(BankPrincipalId::new, 1))
        .principal(id(BankPrincipalId::new, 2))
        .principal(id(BankPrincipalId::new, 3))
        .business(id(BusinessId::new, 1))
        .institution_cash_account(id(AccountId::new, 100), id(InstitutionId::new, 1));
    for ordinal in 0..unrelated_journals {
        builder = builder.principal(unrelated_principal(ordinal));
    }
    let empty = builder.build().unwrap();
    let first = BankProposalEngine::prepare_create_personal_account(
        &empty,
        descriptive_binding(1),
        &key("fixture-create-1"),
        &CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, 1),
            display_name: AccountName::new("Daily").unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let second = BankProposalEngine::prepare_create_personal_account(
        &first,
        descriptive_binding(1),
        &key("fixture-create-2"),
        &CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, 2),
            display_name: AccountName::new("Recipient").unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let source = second.primary_account(id(BankPrincipalId::new, 1)).unwrap();
    let funded = BankProposalEngine::prepare_opening_funding(
        &second,
        descriptive_binding(2),
        &key("fixture-fund"),
        &ApplyOpeningFunding {
            institution: id(InstitutionId::new, 1),
            account: source,
            amount: Money::from_minor(10_000).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    add_unrelated_funded_accounts(funded, unrelated_journals)
}

fn add_unrelated_funded_accounts(
    mut snapshot: BankSnapshot,
    unrelated_accounts: usize,
) -> BankSnapshot {
    for ordinal in 0..unrelated_accounts {
        let principal = unrelated_principal(ordinal);
        let account_name = format!("Unrelated {ordinal}");
        snapshot = BankProposalEngine::prepare_create_personal_account(
            &snapshot,
            descriptive_binding(3),
            &key(&format!("unrelated-create-{ordinal}")),
            &CreatePersonalAccount {
                institution: id(InstitutionId::new, 1),
                owner: principal,
                display_name: AccountName::new(&account_name).unwrap(),
            },
        )
        .unwrap()
        .proposed_snapshot()
        .clone();
        let account = snapshot.primary_account(principal).unwrap();
        snapshot = BankProposalEngine::prepare_opening_funding(
            &snapshot,
            descriptive_binding(4),
            &key(&format!("unrelated-fund-{ordinal}")),
            &ApplyOpeningFunding {
                institution: id(InstitutionId::new, 1),
                account,
                amount: Money::from_minor(1).unwrap(),
            },
        )
        .unwrap()
        .proposed_snapshot()
        .clone();
    }
    snapshot
}

pub(super) fn unrelated_principal(ordinal: usize) -> BankPrincipalId {
    id(
        BankPrincipalId::new,
        1_000 + u64::try_from(ordinal).unwrap(),
    )
}
