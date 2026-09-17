use crate::accounting::{BankJournalEntry, BankPosting};
use crate::model::SignedMoney;
use crate::proposals::{
    complete_proposal, ensure_open, BankIdempotencyClaim, BankIdempotencyKey,
    BankInvariantApprovedProposal, BankOperationScopeBinding, BankProposalDenial,
    BankProposedEffect, BankSnapshot, CanonicalProposalPayload,
};
use crate::schema::{PostingPurpose, ReversalReason, ReverseJournal};

use super::BankProposalEngine;

impl BankProposalEngine {
    pub fn prepare_reverse_journal(
        snapshot: &BankSnapshot,
        binding: BankOperationScopeBinding,
        key: &BankIdempotencyKey,
        input: &ReverseJournal,
    ) -> Result<BankInvariantApprovedProposal, BankProposalDenial> {
        if !snapshot.is_known_institution(input.institution) {
            return Err(BankProposalDenial::UnknownInstitution);
        }
        if snapshot.is_reversed(input.journal) {
            return Err(BankProposalDenial::JournalAlreadyReversed(input.journal));
        }
        let original = snapshot
            .journal_entry(input.journal)
            .cloned()
            .ok_or(BankProposalDenial::UnknownJournal(input.journal))?;
        if !original.postings().iter().all(|posting| {
            snapshot
                .account(posting.account())
                .is_some_and(|account| account.institution() == input.institution)
        }) {
            return Err(BankProposalDenial::ScopeInputMismatch);
        }
        let payload = CanonicalProposalPayload::new("reverse-journal")
            .u64("institution", input.institution.get())
            .text("journal", &input.journal.canonical_text())
            .byte("reason", reversal_reason_tag(input.reason));
        let intent = BankIdempotencyClaim::derive(binding, key, payload);

        let mut proposed = snapshot.clone();
        let identity = intent.key().bytes();
        let journal_id = crate::model::JournalEntryId::from_operation(identity, 0);
        let mut postings = Vec::with_capacity(original.postings().len());
        for (ordinal, original_posting) in original.postings().iter().enumerate() {
            ensure_open(&proposed, original_posting.account())?;
            let reversed_amount = original_posting
                .amount()
                .minor_units()
                .checked_neg()
                .ok_or(BankProposalDenial::ArithmeticOverflow)?;
            postings.push(BankPosting::new(
                crate::model::PostingId::from_operation(
                    identity,
                    u32::try_from(ordinal).map_err(|_| BankProposalDenial::IdentityExhausted)?,
                ),
                original_posting.account(),
                SignedMoney::from_minor(reversed_amount),
            ));
        }
        let reversal = BankJournalEntry::new(
            journal_id,
            PostingPurpose::Reversal,
            postings,
            Some(input.journal),
        );
        proposed.append_proposed_journal(reversal.clone())?;
        proposed.mark_reversed(input.journal);
        let effects = vec![BankProposedEffect::ReverseJournal {
            original: input.journal,
            reversal,
        }];
        complete_proposal(snapshot, proposed, intent, effects)
    }
}

const fn reversal_reason_tag(reason: ReversalReason) -> u8 {
    match reason {
        ReversalReason::Duplicate => 1,
        ReversalReason::OperatorCorrection => 2,
        ReversalReason::ExternalReturn => 3,
    }
}
