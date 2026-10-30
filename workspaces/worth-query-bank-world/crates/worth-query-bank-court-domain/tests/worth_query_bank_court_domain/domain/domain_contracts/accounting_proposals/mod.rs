pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::accounting_proposals::*;

pub(crate) mod reversal;

#[test]
fn invariant_proposal_retains_exact_causal_snapshot_basis() {
    let basis = fixture();
    let exact_clone = basis.clone();
    let independently_built_same_version = fixture();
    assert_eq!(basis, independently_built_same_version);

    let proposal = BankProposalEngine::prepare_create_personal_account(
        &basis,
        binding(1),
        &key("basis-authority"),
        &CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, 1),
            display_name: AccountName::new("Causal basis").unwrap(),
        },
    )
    .unwrap();

    assert!(proposal.matches_basis(&basis));
    assert!(proposal.matches_basis(&exact_clone));
    assert!(!proposal.matches_basis(&independently_built_same_version));
}

#[test]
fn journal_conservation_and_balances_are_independently_recomputable() {
    let first = create_personal(&fixture(), 1, "Daily", "create-1");
    let second = create_personal(&first, 2, "Savings", "create-2");
    let source = second
        .primary_account(id(BankPrincipalId::new, 1))
        .expect("source account exists");
    let destination = second
        .primary_account(id(BankPrincipalId::new, 2))
        .expect("destination account exists");
    let funded = BankProposalEngine::prepare_opening_funding(
        &second,
        binding(2),
        &key("fund"),
        &ApplyOpeningFunding {
            institution: id(InstitutionId::new, 1),
            account: source,
            amount: Money::from_minor(10_000).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let transferred = BankProposalEngine::prepare_send_money(
        &funded,
        binding(3),
        &key("send"),
        &SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(2_500).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();

    for entry in transferred.journal() {
        let sum = entry
            .postings()
            .iter()
            .map(|posting| posting.amount().minor_units())
            .sum::<i64>();
        assert_eq!(sum, 0, "independent journal oracle");
    }
    assert_eq!(oracle_balance(&transferred, source), 7_500);
    assert_eq!(oracle_balance(&transferred, destination), 2_500);
    assert_eq!(
        account_balance(transferred.journal(), source)
            .unwrap()
            .minor_units(),
        7_500
    );
}

#[test]
fn overdraft_is_denied_before_an_approved_proposal_exists() {
    let first = create_personal(&fixture(), 1, "Daily", "create-1");
    let second = create_personal(&first, 2, "Savings", "create-2");
    let source = second.primary_account(id(BankPrincipalId::new, 1)).unwrap();
    let denial = BankProposalEngine::prepare_send_money(
        &second,
        binding(3),
        &key("overdraft"),
        &SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(1).unwrap(),
        },
    )
    .err()
    .expect("overdraft must deny");
    assert_eq!(denial, BankProposalDenial::InsufficientFunds(source));
}

#[test]
fn account_creation_is_unfunded_and_duplicate_or_wrong_institution_denies() {
    let initial = fixture();
    let created = BankProposalEngine::prepare_create_personal_account(
        &initial,
        binding(1),
        &key("create"),
        &CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, 1),
            display_name: AccountName::new("Daily").unwrap(),
        },
    )
    .unwrap();
    assert!(created.proposed_snapshot().journal().is_empty());
    assert_eq!(
        BankProposalEngine::prepare_create_personal_account(
            created.proposed_snapshot(),
            binding(1),
            &key("duplicate"),
            &CreatePersonalAccount {
                institution: id(InstitutionId::new, 1),
                owner: id(BankPrincipalId::new, 1),
                display_name: AccountName::new("Second").unwrap(),
            },
        )
        .err(),
        Some(BankProposalDenial::DuplicatePersonalAccount)
    );
    assert_eq!(
        BankProposalEngine::prepare_create_personal_account(
            &initial,
            binding(1),
            &key("wrong-institution"),
            &CreatePersonalAccount {
                institution: id(InstitutionId::new, 99),
                owner: id(BankPrincipalId::new, 1),
                display_name: AccountName::new("Wrong bank").unwrap(),
            },
        )
        .err(),
        Some(BankProposalDenial::UnknownInstitution)
    );
}

#[test]
fn stable_recipient_identity_must_resolve_to_a_real_primary_account() {
    let opened = create_personal(&fixture(), 1, "Daily", "create-1");
    let source = opened.primary_account(id(BankPrincipalId::new, 1)).unwrap();
    let denial = BankProposalEngine::prepare_send_money(
        &opened,
        binding(3),
        &key("unknown-recipient"),
        &SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(1).unwrap(),
        },
    )
    .err()
    .unwrap();
    assert_eq!(denial, BankProposalDenial::UnknownRecipient);
}

#[test]
fn opening_funding_is_one_time_and_movement_respects_account_status() {
    let opened = create_personal(&fixture(), 1, "Daily", "create-1");
    let source = opened.primary_account(id(BankPrincipalId::new, 1)).unwrap();
    let funding = ApplyOpeningFunding {
        institution: id(InstitutionId::new, 1),
        account: source,
        amount: Money::from_minor(5_000).unwrap(),
    };
    let funded =
        BankProposalEngine::prepare_opening_funding(&opened, binding(2), &key("fund"), &funding)
            .unwrap()
            .proposed_snapshot()
            .clone();
    assert_eq!(
        BankProposalEngine::prepare_opening_funding(
            &funded,
            binding(2),
            &key("fund-again"),
            &funding,
        )
        .err(),
        Some(BankProposalDenial::AccountAlreadyFunded(source))
    );

    for status in [AccountStatus::Frozen, AccountStatus::Closed] {
        let blocked = BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
            .institution(id(InstitutionId::new, 1))
            .principal(id(BankPrincipalId::new, 1))
            .institution_cash_account(id(AccountId::new, 100), id(InstitutionId::new, 1))
            .personal_account(
                id(AccountId::new, 101),
                id(InstitutionId::new, 1),
                id(BankPrincipalId::new, 1),
                AccountName::new("Blocked").unwrap(),
                status,
            )
            .build()
            .unwrap();
        let denial = BankProposalEngine::prepare_deposit(
            &blocked,
            binding(2),
            &key("blocked"),
            &Deposit {
                institution: id(InstitutionId::new, 1),
                account: id(AccountId::new, 101),
                amount: Money::from_minor(1).unwrap(),
            },
        )
        .err()
        .unwrap();
        assert_eq!(
            denial,
            BankProposalDenial::AccountStatus {
                account: id(AccountId::new, 101),
                status,
            }
        );
    }
}

#[test]
fn deposit_and_withdrawal_conserve_money_with_journal_only_mutation_effects() {
    let opened = create_personal(&fixture(), 1, "Daily", "create-1");
    let account = opened.primary_account(id(BankPrincipalId::new, 1)).unwrap();
    let deposited = BankProposalEngine::prepare_deposit(
        &opened,
        binding(2),
        &key("deposit"),
        &Deposit {
            institution: id(InstitutionId::new, 1),
            account,
            amount: Money::from_minor(7_000).unwrap(),
        },
    )
    .unwrap();
    assert_eq!(deposited.effects().len(), 1);
    let after_deposit = deposited.proposed_snapshot().clone();
    let withdrawn = BankProposalEngine::prepare_withdrawal(
        &after_deposit,
        binding(2),
        &key("withdraw"),
        &Withdraw {
            institution: id(InstitutionId::new, 1),
            account,
            amount: Money::from_minor(2_000).unwrap(),
        },
    )
    .unwrap();
    assert_eq!(withdrawn.effects().len(), 1);
    assert_eq!(
        oracle_balance(withdrawn.proposed_snapshot(), account),
        5_000
    );
}

#[test]
fn created_identity_is_retry_stable_and_disjoint_from_fixtures_and_other_keys() {
    let fixture = BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
        .institution(id(InstitutionId::new, 1))
        .principal(id(BankPrincipalId::new, 1))
        .institution_cash_account(id(AccountId::new, u64::MAX), id(InstitutionId::new, 1))
        .build()
        .expect("fixture and operation identities occupy disjoint namespaces");
    let input = CreatePersonalAccount {
        institution: id(InstitutionId::new, 1),
        owner: id(BankPrincipalId::new, 1),
        display_name: AccountName::new("Operation-created").unwrap(),
    };
    let first = BankProposalEngine::prepare_create_personal_account(
        &fixture,
        binding(8),
        &key("stable"),
        &input,
    )
    .unwrap();
    let retry = BankProposalEngine::prepare_create_personal_account(
        &fixture,
        binding(8),
        &key("stable"),
        &input,
    )
    .unwrap();
    let independent = BankProposalEngine::prepare_create_personal_account(
        &fixture,
        binding(8),
        &key("independent"),
        &input,
    )
    .unwrap();
    let created = first
        .proposed_snapshot()
        .primary_account(input.owner)
        .unwrap();
    assert_eq!(
        retry.proposed_snapshot().primary_account(input.owner),
        Some(created)
    );
    assert_ne!(
        independent.proposed_snapshot().primary_account(input.owner),
        Some(created)
    );
    assert_ne!(created, id(AccountId::new, u64::MAX));
}
