use std::collections::BTreeMap;

use bank_domain::accounting::BankJournalEntry;
use bank_domain::model::{AccountId, AccountJournalRevision};
use bank_domain::proposals::BankSnapshot;
use bank_domain::schema::{
    Account, AccountActivityEffect, AccountIdentity, AccountingRevision, ActivityEvent, BankSchema,
    JournalEntry, JournalIdentityField, JournalPosting, JournalPurpose, Posting, PostingAccount,
    PostingAccountSequence, PostingAmount, PostingIdentityField, Purpose,
};
use worth_query_host::facade::declaration::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationEffectMarkerIdentity, OperationEmits},
};
use worth_query_host::facade::domain::{
    OperationCreates, OperationLinks, OperationReads, OperationWrites,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, HandlerExecutionDenial, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEntityKey,
};

use crate::graph_bootstrap::{journal_key, posting_key};

pub(super) fn author_journal<Binding>(
    candidate: &mut CandidateWriter<'_, BankSchema, Binding>,
    journal: &BankJournalEntry,
    proposed: &BankSnapshot,
) -> Result<WorthQueryApplicationEffectEntity<BankSchema, JournalEntry>, HandlerExecutionDenial>
where
    Binding: ApplicationMutationBinding<BankSchema>,
    AccountIdentity: OperationReads<Binding::Operation>,
    JournalEntry: OperationCreates<Binding::Operation>,
    Posting: OperationCreates<Binding::Operation>,
    JournalIdentityField: OperationWrites<Binding::Operation>,
    JournalPurpose: OperationWrites<Binding::Operation>,
    PostingIdentityField: OperationWrites<Binding::Operation>,
    PostingAmount: OperationWrites<Binding::Operation>,
    PostingAccountSequence: OperationWrites<Binding::Operation>,
    Purpose: OperationWrites<Binding::Operation>,
    AccountingRevision: OperationWrites<Binding::Operation>,
    JournalPosting: OperationLinks<Binding::Operation>,
    PostingAccount: OperationLinks<Binding::Operation>,
    AccountActivityEffect:
        ApplicationEffectMarkerIdentity<BankSchema> + OperationEmits<Binding::Operation>,
{
    let mut sequences = posting_sequences(journal, proposed)?;
    let context_id = journal
        .postings()
        .first()
        .ok_or_else(|| HandlerExecutionDenial::new(InvalidJournalCandidate))?
        .account();
    let context = resolve_account::<Binding>(candidate, context_id)?;
    let journal_entity = candidate
        .create_entity_in_context(
            &context,
            JournalEntry::reference(),
            entity_key(journal_key(journal.id()))?,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &journal_entity,
            JournalIdentityField::reference(),
            journal.id(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &journal_entity,
            JournalPurpose::reference(),
            journal.purpose(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    for posting in journal.postings() {
        author_posting::<Binding>(
            candidate,
            &context,
            &journal_entity,
            journal,
            posting,
            &mut sequences,
        )?;
    }
    for (account_id, sequence) in sequences {
        if sequence.remaining != 0 || sequence.next != sequence.final_revision {
            return Err(HandlerExecutionDenial::new(InvalidJournalCandidate));
        }
        let account = resolve_account::<Binding>(candidate, account_id)?;
        candidate
            .write_field(
                &account,
                AccountingRevision::reference(),
                sequence.final_revision,
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(journal_entity)
}

fn author_posting<Binding>(
    candidate: &mut CandidateWriter<'_, BankSchema, Binding>,
    context: &WorthQueryApplicationEffectEntity<BankSchema, Account>,
    journal_entity: &WorthQueryApplicationEffectEntity<BankSchema, JournalEntry>,
    journal: &BankJournalEntry,
    posting: &bank_domain::accounting::BankPosting,
    sequences: &mut BTreeMap<AccountId, PostingSequence>,
) -> Result<(), HandlerExecutionDenial>
where
    Binding: ApplicationMutationBinding<BankSchema>,
    AccountIdentity: OperationReads<Binding::Operation>,
    Posting: OperationCreates<Binding::Operation>,
    PostingIdentityField: OperationWrites<Binding::Operation>,
    PostingAmount: OperationWrites<Binding::Operation>,
    PostingAccountSequence: OperationWrites<Binding::Operation>,
    Purpose: OperationWrites<Binding::Operation>,
    JournalPosting: OperationLinks<Binding::Operation>,
    PostingAccount: OperationLinks<Binding::Operation>,
    AccountActivityEffect:
        ApplicationEffectMarkerIdentity<BankSchema> + OperationEmits<Binding::Operation>,
{
    let posting_entity = candidate
        .create_entity_in_context(
            context,
            Posting::reference(),
            entity_key(posting_key(posting.id()))?,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &posting_entity,
            PostingIdentityField::reference(),
            posting.id(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &posting_entity,
            PostingAmount::reference(),
            posting.amount(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    let sequence = sequences
        .get_mut(&posting.account())
        .ok_or_else(|| HandlerExecutionDenial::new(InvalidJournalCandidate))?;
    let account_sequence = sequence.next;
    sequence.remaining = sequence
        .remaining
        .checked_sub(1)
        .ok_or_else(|| HandlerExecutionDenial::new(InvalidJournalCandidate))?;
    if sequence.remaining != 0 {
        sequence.next = sequence
            .next
            .next()
            .ok_or_else(|| HandlerExecutionDenial::new(InvalidJournalCandidate))?;
    }
    candidate
        .initialize_field(
            &posting_entity,
            PostingAccountSequence::reference(),
            account_sequence,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(&posting_entity, Purpose::reference(), journal.purpose())
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            JournalPosting::reference(),
            format!("journal-posting:{}", posting.id().canonical_text()),
            journal_entity,
            &posting_entity,
        )
        .map_err(HandlerExecutionDenial::new)?;
    let account = resolve_account::<Binding>(candidate, posting.account())?;
    candidate
        .link(
            PostingAccount::reference(),
            format!("posting-account:{}", posting.id().canonical_text()),
            &posting_entity,
            &account,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .emit(
            AccountActivityEffect::reference(),
            ActivityEvent {
                account: posting.account(),
                journal: journal.id(),
                posting: posting.id(),
                journal_sequence: account_sequence.get(),
            },
        )
        .map_err(HandlerExecutionDenial::new)
}

fn posting_sequences(
    journal: &BankJournalEntry,
    proposed: &BankSnapshot,
) -> Result<BTreeMap<AccountId, PostingSequence>, HandlerExecutionDenial> {
    let mut counts = BTreeMap::<AccountId, u64>::new();
    for posting in journal.postings() {
        let count = counts.entry(posting.account()).or_default();
        *count = count
            .checked_add(1)
            .ok_or_else(|| HandlerExecutionDenial::new(InvalidJournalCandidate))?;
    }
    counts
        .into_iter()
        .map(|(account, count)| {
            let final_revision = proposed
                .account_journal_revision(account)
                .ok_or_else(|| HandlerExecutionDenial::new(InvalidJournalCandidate))?;
            let previous = final_revision
                .get()
                .checked_sub(count)
                .map(AccountJournalRevision::from_posting_count)
                .ok_or_else(|| HandlerExecutionDenial::new(InvalidJournalCandidate))?;
            let next = previous
                .next()
                .ok_or_else(|| HandlerExecutionDenial::new(InvalidJournalCandidate))?;
            Ok((
                account,
                PostingSequence {
                    next,
                    final_revision,
                    remaining: count,
                },
            ))
        })
        .collect()
}

fn resolve_account<Binding>(
    candidate: &CandidateWriter<'_, BankSchema, Binding>,
    account: AccountId,
) -> Result<WorthQueryApplicationEffectEntity<BankSchema, Account>, HandlerExecutionDenial>
where
    Binding: ApplicationMutationBinding<BankSchema>,
    AccountIdentity: OperationReads<Binding::Operation>,
{
    candidate
        .resolve_entity(AccountIdentity::reference(), account)
        .map_err(HandlerExecutionDenial::new)
}

fn entity_key<Entity>(
    key: String,
) -> Result<WorthQueryApplicationEntityKey<BankSchema, Entity>, HandlerExecutionDenial> {
    WorthQueryApplicationEntityKey::new(key).map_err(HandlerExecutionDenial::new)
}

struct PostingSequence {
    next: AccountJournalRevision,
    final_revision: AccountJournalRevision,
    remaining: u64,
}

#[derive(Debug)]
struct InvalidJournalCandidate;

impl std::fmt::Display for InvalidJournalCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("approved payment produced an invalid journal candidate")
    }
}

impl std::error::Error for InvalidJournalCandidate {}
