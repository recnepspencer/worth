use std::collections::BTreeSet;

use crate::proposals::{BankProposalDenial, BankSnapshot};
use crate::schema::AccountKind;

use super::account_balance;

pub(crate) struct BankInvariantWitness {
    _private: (),
}

pub(crate) fn validate_proposed_snapshot(
    basis: &BankSnapshot,
    proposed: &BankSnapshot,
) -> Result<BankInvariantWitness, BankProposalDenial> {
    validate_proposed_snapshot_for_accounts(
        basis,
        proposed,
        proposed.accounts().map(|account| account.id()),
    )
}

pub(crate) fn validate_proposed_decision_snapshot(
    basis: &BankSnapshot,
    proposed: &BankSnapshot,
    required_balance_accounts: impl IntoIterator<Item = crate::model::AccountId>,
    starting_balances: std::collections::BTreeMap<
        crate::model::AccountId,
        crate::model::SignedMoney<crate::model::USD>,
    >,
) -> Result<BankInvariantWitness, BankProposalDenial> {
    validate_proposed_snapshot_for_accounts(basis, proposed, std::iter::empty())?;
    let appended = proposed
        .journal()
        .get(basis.journal().len()..)
        .ok_or(BankProposalDenial::SnapshotInvariantViolated)?;
    for account_id in required_balance_accounts {
        let account = proposed
            .account(account_id)
            .ok_or(BankProposalDenial::SnapshotInvariantViolated)?;
        let starting = starting_balances
            .get(&account_id)
            .copied()
            .ok_or(BankProposalDenial::SnapshotInvariantViolated)?;
        let delta = account_balance(appended, account_id)
            .map_err(|_| BankProposalDenial::ArithmeticOverflow)?;
        let balance = starting
            .minor_units()
            .checked_add(delta.minor_units())
            .ok_or(BankProposalDenial::ArithmeticOverflow)?;
        if matches!(
            account.kind(),
            AccountKind::Personal | AccountKind::Business
        ) && balance < 0
        {
            return Err(BankProposalDenial::InsufficientFunds(account_id));
        }
    }
    Ok(BankInvariantWitness { _private: () })
}

fn validate_proposed_snapshot_for_accounts(
    basis: &BankSnapshot,
    proposed: &BankSnapshot,
    required_balance_accounts: impl IntoIterator<Item = crate::model::AccountId>,
) -> Result<BankInvariantWitness, BankProposalDenial> {
    if !basis.has_valid_topology()
        || !proposed.has_valid_topology()
        || !proposed.journal().starts_with(basis.journal())
    {
        return Err(BankProposalDenial::SnapshotInvariantViolated);
    }

    let mut journal_ids = BTreeSet::new();
    let mut posting_ids = BTreeSet::new();
    for entry in proposed.journal() {
        if entry.postings().len() < 2 {
            return Err(BankProposalDenial::JournalHasTooFewPostings);
        }
        if !journal_ids.insert(entry.id()) {
            return Err(BankProposalDenial::SnapshotInvariantViolated);
        }
        let sum = entry.postings().iter().try_fold(0_i64, |sum, posting| {
            if proposed.account(posting.account()).is_none() || !posting_ids.insert(posting.id()) {
                return None;
            }
            sum.checked_add(posting.amount().minor_units())
        });
        if sum != Some(0) {
            return Err(match sum {
                None => BankProposalDenial::ArithmeticOverflow,
                Some(_) => BankProposalDenial::JournalIsUnbalanced,
            });
        }
    }

    for account_id in required_balance_accounts {
        let account = proposed
            .account(account_id)
            .ok_or(BankProposalDenial::SnapshotInvariantViolated)?;
        if matches!(
            account.kind(),
            AccountKind::Personal | AccountKind::Business
        ) && account_balance(proposed.journal(), account_id)
            .map_err(|_| BankProposalDenial::ArithmeticOverflow)?
            .minor_units()
            < 0
        {
            return Err(BankProposalDenial::InsufficientFunds(account_id));
        }
    }

    Ok(BankInvariantWitness { _private: () })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{BankJournalEntry, BankPosting};
    use crate::model::{
        AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, InstitutionId,
        JournalEntryId, PostingId, SignedMoney,
    };
    use crate::proposals::BankSnapshotBuilder;
    use crate::schema::{AccountStatus, PostingPurpose};

    #[test]
    fn malformed_journal_shapes_fail_the_invariant_oracle() {
        let basis = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
            .institution(InstitutionId::new(1).unwrap())
            .principal(BankPrincipalId::new(1).unwrap())
            .personal_account(
                AccountId::new(1).unwrap(),
                InstitutionId::new(1).unwrap(),
                BankPrincipalId::new(1).unwrap(),
                AccountName::new("Invariant target").unwrap(),
                AccountStatus::Open,
            )
            .build()
            .unwrap();

        let mut partial = basis.clone();
        partial.append_projected_journal(BankJournalEntry::new(
            JournalEntryId::new(1).unwrap(),
            PostingPurpose::Deposit,
            vec![BankPosting::new(
                PostingId::new(1).unwrap(),
                AccountId::new(1).unwrap(),
                SignedMoney::from_minor(10),
            )],
            None,
        ));
        assert!(matches!(
            validate_proposed_snapshot(&basis, &partial),
            Err(BankProposalDenial::JournalHasTooFewPostings)
        ));

        let mut unbalanced = basis.clone();
        unbalanced.append_projected_journal(BankJournalEntry::new(
            JournalEntryId::new(2).unwrap(),
            PostingPurpose::Deposit,
            vec![
                BankPosting::new(
                    PostingId::new(2).unwrap(),
                    AccountId::new(1).unwrap(),
                    SignedMoney::from_minor(10),
                ),
                BankPosting::new(
                    PostingId::new(3).unwrap(),
                    AccountId::new(1).unwrap(),
                    SignedMoney::from_minor(-9),
                ),
            ],
            None,
        ));
        assert!(matches!(
            validate_proposed_snapshot(&basis, &unbalanced),
            Err(BankProposalDenial::JournalIsUnbalanced)
        ));
    }
}
