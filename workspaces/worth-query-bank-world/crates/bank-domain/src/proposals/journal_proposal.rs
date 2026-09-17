use crate::accounting::{BankJournalEntry, BankPosting};
use crate::model::{AccountId, JournalEntryId, Money, PostingId, SignedMoney, USD};
use crate::schema::{AccountStatus, PostingPurpose};

use super::{BankIdempotencyKeyIdentity, BankProposalDenial, BankSnapshot};

pub(crate) fn append_balanced_transfer(
    snapshot: &mut BankSnapshot,
    debit: AccountId,
    credit: AccountId,
    amount: Money<USD>,
    purpose: PostingPurpose,
    reversal_of: Option<JournalEntryId>,
    identity: BankIdempotencyKeyIdentity,
) -> Result<BankJournalEntry, BankProposalDenial> {
    if debit == credit {
        return Err(BankProposalDenial::SelfTransfer);
    }
    let identity = identity.bytes();
    let journal_id = JournalEntryId::from_operation(identity, 0);
    let debit_posting = BankPosting::new(
        PostingId::from_operation(identity, 0),
        debit,
        SignedMoney::from_minor(
            amount
                .minor_units()
                .checked_neg()
                .ok_or(BankProposalDenial::ArithmeticOverflow)?,
        ),
    );
    let credit_posting = BankPosting::new(
        PostingId::from_operation(identity, 1),
        credit,
        SignedMoney::from(amount),
    );
    let entry = BankJournalEntry::new(
        journal_id,
        purpose,
        vec![debit_posting, credit_posting],
        reversal_of,
    );
    ensure_open(snapshot, debit)?;
    ensure_open(snapshot, credit)?;
    snapshot.append_proposed_journal(entry.clone())?;
    Ok(entry)
}

pub(crate) fn ensure_open(
    snapshot: &BankSnapshot,
    account_id: AccountId,
) -> Result<(), BankProposalDenial> {
    let account = snapshot
        .account(account_id)
        .ok_or(BankProposalDenial::UnknownAccount(account_id))?;
    if account.status() != AccountStatus::Open {
        return Err(BankProposalDenial::AccountStatus {
            account: account_id,
            status: account.status(),
        });
    }
    Ok(())
}
