use crate::accounting::validate_proposed_decision_snapshot;
use crate::accounting::validate_proposed_snapshot;

use super::{
    BankIdempotencyClaim, BankInvariantApprovedProposal, BankProposalDenial, BankProposedEffect,
    BankSnapshot,
};

pub(crate) fn complete_proposal(
    basis: &BankSnapshot,
    proposed: BankSnapshot,
    idempotency: BankIdempotencyClaim,
    effects: Vec<BankProposedEffect>,
) -> Result<BankInvariantApprovedProposal, BankProposalDenial> {
    validate_effect_replay(basis, &proposed, &effects)?;
    let witness = validate_proposed_snapshot(basis, &proposed)?;
    Ok(BankInvariantApprovedProposal::new(
        basis.retain_basis(),
        idempotency,
        effects,
        proposed,
        witness,
    ))
}

pub(crate) fn complete_decision_proposal(
    basis: BankSnapshot,
    required_balance_accounts: std::collections::BTreeSet<crate::model::AccountId>,
    starting_balances: std::collections::BTreeMap<
        crate::model::AccountId,
        crate::model::SignedMoney<crate::model::USD>,
    >,
    proposed: BankSnapshot,
    idempotency: BankIdempotencyClaim,
    effects: Vec<BankProposedEffect>,
) -> Result<BankInvariantApprovedProposal, BankProposalDenial> {
    validate_effect_replay(&basis, &proposed, &effects)?;
    let witness = validate_proposed_decision_snapshot(
        &basis,
        &proposed,
        required_balance_accounts,
        starting_balances,
    )?;
    Ok(BankInvariantApprovedProposal::new(
        basis.retain_basis(),
        idempotency,
        effects,
        proposed,
        witness,
    ))
}

fn validate_effect_replay(
    basis: &BankSnapshot,
    proposed: &BankSnapshot,
    effects: &[BankProposedEffect],
) -> Result<(), BankProposalDenial> {
    let mut replayed = basis.clone();
    for effect in effects {
        match effect {
            BankProposedEffect::CreateAccount(account) => {
                if replayed.account(account.id()).is_some() {
                    return Err(BankProposalDenial::SnapshotInvariantViolated);
                }
                replayed.insert_account(account.clone());
            }
            BankProposedEffect::AppendJournal(entry) => {
                ensure_new_journal_identity(&replayed, entry)?;
                replayed.append_proposed_journal(entry.clone())?;
            }
            BankProposedEffect::ReverseJournal { original, reversal } => {
                ensure_new_journal_identity(&replayed, reversal)?;
                replayed.append_proposed_journal(reversal.clone())?;
                replayed.mark_reversed(*original);
            }
            BankProposedEffect::CreatePayment(payment) => {
                if replayed.payment(payment.id()).is_some() {
                    return Err(BankProposalDenial::SnapshotInvariantViolated);
                }
                replayed.insert_payment(payment.clone());
            }
            BankProposedEffect::UpdatePayment {
                payment,
                replacement,
            } => {
                if replacement.id() != *payment || replayed.payment(*payment).is_none() {
                    return Err(BankProposalDenial::SnapshotInvariantViolated);
                }
                replayed.replace_payment(replacement.clone());
            }
            BankProposedEffect::GrantAuthorization(authorization) => {
                if replayed.authorization(authorization.id()).is_some() {
                    return Err(BankProposalDenial::SnapshotInvariantViolated);
                }
                replayed.insert_authorization(*authorization);
            }
            BankProposedEffect::RevokeAuthorization(authorization) => {
                if replayed.remove_authorization(authorization.id()) != Some(*authorization) {
                    return Err(BankProposalDenial::SnapshotInvariantViolated);
                }
            }
        }
    }
    if replayed == *proposed {
        Ok(())
    } else {
        Err(BankProposalDenial::SnapshotInvariantViolated)
    }
}

fn ensure_new_journal_identity(
    snapshot: &BankSnapshot,
    entry: &crate::accounting::BankJournalEntry,
) -> Result<(), BankProposalDenial> {
    if snapshot.journal_entry(entry.id()).is_some() {
        return Err(BankProposalDenial::SnapshotInvariantViolated);
    }
    let mut posting_ids = std::collections::BTreeSet::new();
    for posting in entry.postings() {
        let already_exists = snapshot
            .journal()
            .iter()
            .flat_map(crate::accounting::BankJournalEntry::postings)
            .any(|candidate| candidate.id() == posting.id());
        if already_exists || !posting_ids.insert(posting.id()) {
            return Err(BankProposalDenial::SnapshotInvariantViolated);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::BankAccount;
    use crate::model::{AccountName, BankPrincipalId, BankSnapshotVersion, InstitutionId};
    use crate::proposals::BankSnapshotBuilder;

    #[test]
    fn omitted_effect_cannot_certify_a_changed_snapshot() {
        let basis = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
            .institution(InstitutionId::new(1).unwrap())
            .principal(BankPrincipalId::new(1).unwrap())
            .build()
            .unwrap();
        let mut proposed = basis.clone();
        let account = BankAccount::personal(
            crate::model::AccountId::from_operation([2; 32], 0),
            InstitutionId::new(1).unwrap(),
            BankPrincipalId::new(1).unwrap(),
            AccountName::new("Unreported account").unwrap(),
        );
        proposed.insert_account(account);

        assert!(matches!(
            complete_proposal(
                &basis,
                proposed,
                BankIdempotencyClaim::derive(
                    super::super::BankOperationScopeBinding::new(
                        1,
                        super::super::BankOperationScopeSchemaBinding::new(1, 1, [1; 32], [2; 32],),
                        "operation-authority",
                        super::super::BankOperationScopeEntityBinding::new(0, 1, 1),
                        super::super::BankOperationScopeEntityBinding::new(0, 2, 1),
                    ),
                    &super::super::BankIdempotencyKey::new("missing-effect").unwrap(),
                    super::super::CanonicalProposalPayload::new("test"),
                ),
                Vec::new(),
            ),
            Err(BankProposalDenial::SnapshotInvariantViolated)
        ));
    }
}
