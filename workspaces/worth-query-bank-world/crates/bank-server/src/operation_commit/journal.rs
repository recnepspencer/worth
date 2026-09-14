use std::collections::BTreeMap;

use bank_domain::accounting::BankJournalEntry;
use bank_domain::model::{AccountId, AccountJournalRevision};
use bank_domain::schema::*;
use worth_query_host::facade::declaration::application_schema::OperationEmits;
use worth_query_host::facade::domain::{
    OperationCreates, OperationLinks, OperationReads, OperationWrites,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationReadAttempt,
    WorthQueryProjectedApplicationMutation,
};

use super::{entity_key, BankCommitPreparationDenial};
use crate::graph_bootstrap::{journal_key, posting_key};

pub(crate) fn resolve_journal_accounts<Operation, Input, Scope>(
    reads: &mut WorthQueryApplicationReadAttempt<
        BankSchema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    journal: &BankJournalEntry,
) -> Result<TouchedAccounts, BankCommitPreparationDenial>
where
    AccountIdentity: OperationReads<Operation>,
    AccountingRevision: OperationReads<Operation>,
{
    let mut posting_counts = BTreeMap::<AccountId, u64>::new();
    for posting in journal.postings() {
        let count = posting_counts.entry(posting.account()).or_default();
        *count = count
            .checked_add(1)
            .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
    }
    let mut accounts = BTreeMap::new();
    for (account_id, posting_count) in posting_counts {
        let identity = reads.resolve_entity(AccountIdentity::reference(), account_id)?;
        let current = reads.observe_field(&identity, AccountingRevision::reference())?;
        let first_sequence = current
            .get()
            .checked_add(1)
            .map(AccountJournalRevision::from_posting_count)
            .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
        let revision = current
            .get()
            .checked_add(posting_count)
            .map(AccountJournalRevision::from_posting_count)
            .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
        accounts.insert(
            account_id,
            TouchedAccount {
                identity,
                revision,
                next_sequence: first_sequence,
                remaining_postings: posting_count,
            },
        );
    }
    Ok(TouchedAccounts(accounts))
}

pub(crate) fn lower_journal<Operation, Input, Scope>(
    effects: &mut WorthQueryApplicationEffectProgramBuilder<BankSchema, Operation, Input, Scope>,
    journal: &BankJournalEntry,
    mut accounts: TouchedAccounts,
) -> Result<WorthQueryApplicationEffectEntity<BankSchema, JournalEntry>, BankCommitPreparationDenial>
where
    JournalEntry: OperationCreates<Operation>,
    Posting: OperationCreates<Operation>,
    JournalIdentityField: OperationWrites<Operation>,
    JournalPurpose: OperationWrites<Operation>,
    PostingIdentityField: OperationWrites<Operation>,
    PostingAmount: OperationWrites<Operation>,
    PostingAccountSequence: OperationWrites<Operation>,
    Purpose: OperationWrites<Operation>,
    AccountingRevision: OperationWrites<Operation>,
    JournalPosting: OperationLinks<Operation>,
    PostingAccount: OperationLinks<Operation>,
    AccountActivityEffect: OperationEmits<Operation>,
{
    let context = accounts
        .0
        .values()
        .next()
        .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
    let context = effects.existing_entity(&context.identity)?;
    let journal_entity = effects.create_entity_in_context(
        &context,
        JournalEntry::reference(),
        entity_key(journal_key(journal.id()))?,
    )?;
    effects.initialize_field(
        &journal_entity,
        JournalIdentityField::reference(),
        journal.id(),
    )?;
    effects.initialize_field(
        &journal_entity,
        JournalPurpose::reference(),
        journal.purpose(),
    )?;
    for posting in journal.postings() {
        lower_posting(
            effects,
            &context,
            &journal_entity,
            journal,
            posting,
            &mut accounts,
        )?;
    }
    for (_, account) in accounts.0 {
        if account.remaining_postings != 0 || account.next_sequence != account.revision {
            return Err(BankCommitPreparationDenial::InvalidProposalShape);
        }
        let target = effects.existing_entity(&account.identity)?;
        effects.write_field(&target, AccountingRevision::reference(), account.revision)?;
    }
    Ok(journal_entity)
}

fn lower_posting<Operation, Input, Scope>(
    effects: &mut WorthQueryApplicationEffectProgramBuilder<BankSchema, Operation, Input, Scope>,
    context: &WorthQueryApplicationEffectEntity<BankSchema, Account>,
    journal_entity: &WorthQueryApplicationEffectEntity<BankSchema, JournalEntry>,
    journal: &BankJournalEntry,
    posting: &bank_domain::accounting::BankPosting,
    accounts: &mut TouchedAccounts,
) -> Result<(), BankCommitPreparationDenial>
where
    Posting: OperationCreates<Operation>,
    PostingIdentityField: OperationWrites<Operation>,
    PostingAmount: OperationWrites<Operation>,
    PostingAccountSequence: OperationWrites<Operation>,
    Purpose: OperationWrites<Operation>,
    JournalPosting: OperationLinks<Operation>,
    PostingAccount: OperationLinks<Operation>,
    AccountActivityEffect: OperationEmits<Operation>,
{
    let posting_entity = effects.create_entity_in_context(
        context,
        Posting::reference(),
        entity_key(posting_key(posting.id()))?,
    )?;
    effects.initialize_field(
        &posting_entity,
        PostingIdentityField::reference(),
        posting.id(),
    )?;
    effects.initialize_field(
        &posting_entity,
        PostingAmount::reference(),
        posting.amount(),
    )?;
    let account = accounts
        .0
        .get_mut(&posting.account())
        .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
    let account_sequence = account.next_sequence;
    account.remaining_postings = account
        .remaining_postings
        .checked_sub(1)
        .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
    if account.remaining_postings != 0 {
        account.next_sequence = account
            .next_sequence
            .next()
            .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
    }
    effects.initialize_field(
        &posting_entity,
        PostingAccountSequence::reference(),
        account_sequence,
    )?;
    effects.initialize_field(&posting_entity, Purpose::reference(), journal.purpose())?;
    effects.link(
        JournalPosting::reference(),
        format!("journal-posting:{}", posting.id().canonical_text()),
        journal_entity,
        &posting_entity,
    )?;
    let account = effects.existing_entity(&account.identity)?;
    effects.link(
        PostingAccount::reference(),
        format!("posting-account:{}", posting.id().canonical_text()),
        &posting_entity,
        &account,
    )?;
    effects.emit(
        AccountActivityEffect::reference(),
        ActivityEvent {
            account: posting.account(),
            journal: journal.id(),
            posting: posting.id(),
            journal_sequence: account_sequence.get(),
        },
    )?;
    Ok(())
}

pub(crate) struct TouchedAccounts(BTreeMap<AccountId, TouchedAccount>);

struct TouchedAccount {
    identity: WorthQueryApplicationEntityIdentity<BankSchema, Account>,
    revision: AccountJournalRevision,
    next_sequence: AccountJournalRevision,
    remaining_postings: u64,
}
