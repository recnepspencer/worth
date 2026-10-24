use std::collections::BTreeMap;

use bank_domain::model::{AccountId, AccountJournalRevision};
use bank_domain::proposals::BankSnapshot;
use bank_domain::schema::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntitySeed, WorthQueryApplicationRelationSeed,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};

use super::{account_key, entity_key, journal_key, posting_key};

pub(super) fn bind_journal(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let mut account_sequences = BTreeMap::<AccountId, AccountJournalRevision>::new();
    for entry in snapshot.journal() {
        let seed = WorthQueryApplicationEntitySeed::new(
            JournalEntry::reference(),
            entity_key(journal_key(entry.id())),
        )
        .field(JournalIdentityField::reference(), entry.id())
        .field(JournalPurpose::reference(), entry.purpose());
        let context = entry
            .postings()
            .first()
            .and_then(|posting| snapshot.account(posting.account()));
        let seed = match context {
            Some(account) => seed.field(
                JournalInstitutionContext::reference(),
                account.institution(),
            ),
            None => seed,
        };
        graph.bind_entity(seed)?;
        for posting in entry.postings() {
            let sequence = account_sequences.entry(posting.account()).or_default();
            *sequence = sequence
                .next()
                .expect("an in-memory fixture cannot contain u64::MAX postings");
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Posting::reference(),
                    entity_key(posting_key(posting.id())),
                )
                .field(PostingIdentityField::reference(), posting.id())
                .field(PostingAmount::reference(), posting.amount())
                .field(PostingAccountSequence::reference(), *sequence)
                .field(Purpose::reference(), entry.purpose()),
            )?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                JournalPosting::reference(),
                format!("journal-posting:{}", posting.id().canonical_text()),
                entity_key(journal_key(entry.id())),
                entity_key(posting_key(posting.id())),
            ))?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                PostingAccount::reference(),
                format!("posting-account:{}", posting.id().canonical_text()),
                entity_key(posting_key(posting.id())),
                entity_key(account_key(posting.account())),
            ))?;
        }
        if let Some(original) = entry.reversal_of() {
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                JournalReversal::reference(),
                format!("journal-reversal:{}", entry.id().canonical_text()),
                entity_key(journal_key(entry.id())),
                entity_key(journal_key(original)),
            ))?;
        }
    }
    Ok(())
}
