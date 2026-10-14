//! Constructs lawful fixture snapshots through the real Bank proposal owner.
use super::*;
pub(super) fn create_personal(
    snapshot: BankSnapshot,
    owner: BankPrincipalId,
    name: &str,
    operation: &str,
) -> BankSnapshot {
    BankProposalEngine::prepare_create_personal_account(
        &snapshot,
        binding(1),
        &key(operation),
        &CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner,
            display_name: AccountName::new(name).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

pub(super) fn create_business(
    snapshot: BankSnapshot,
    institution: InstitutionId,
    business: BusinessId,
) -> BankSnapshot {
    BankProposalEngine::prepare_create_business_account(
        &snapshot,
        binding(2),
        &key("business"),
        &CreateBusinessAccount {
            institution,
            business,
            display_name: AccountName::new("Operations").unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

pub(super) fn fund(
    snapshot: BankSnapshot,
    institution: InstitutionId,
    account: AccountId,
    amount: i64,
    operation: &str,
) -> BankSnapshot {
    BankProposalEngine::prepare_opening_funding(
        &snapshot,
        binding(3),
        &key(operation),
        &ApplyOpeningFunding {
            institution,
            account,
            amount: Money::from_minor(amount).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

pub(super) fn send(
    snapshot: BankSnapshot,
    from: AccountId,
    recipient: BankPrincipalId,
) -> BankSnapshot {
    BankProposalEngine::prepare_send_money(
        &snapshot,
        binding(4),
        &key("personal-send"),
        &SendMoney {
            from,
            recipient,
            amount: Money::from_minor(2_500).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

pub(super) fn grant(
    snapshot: BankSnapshot,
    account: AccountId,
    principal: BankPrincipalId,
    role: CustomerRole,
    operation: &str,
) -> BankSnapshot {
    BankProposalEngine::prepare_grant_account_authorization(
        &snapshot,
        binding(5),
        &key(operation),
        &GrantAccountAuthorization {
            account,
            principal,
            role,
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
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

pub(super) fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}
