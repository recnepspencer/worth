use bank_domain::proposals::{
    BankInvariantApprovedProposal, BankProposalDenial, BankProposalEngine, BankProposedEffect,
};
use bank_domain::schema::{
    BankSchema, JournalIdentityField, JournalReversal, MoneyMovementResult, ReverseJournal,
    ReverseJournalMutationBinding,
};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};

use super::journal::author_journal;
use crate::bank_projection::project_journal_reversal;
use crate::operation_admission::bank_operation_scope_binding;

pub(crate) struct ReverseJournalHandler;

impl OperationHandler<BankSchema, ReverseJournalMutationBinding> for ReverseJournalHandler {
    fn decide(
        &self,
        input: &ReverseJournal,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, ReverseJournalMutationBinding>,
    ) -> HandlerResult<BankInvariantApprovedProposal, BankProposalDenial> {
        let institution = reader.scope().clone();
        let snapshot =
            match project_journal_reversal(reader.reader(), &institution, input.institution, input)
            {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                }
            };
        if let Some(journal) = snapshot.journal_entry(input.journal) {
            let matches_scope = journal.postings().iter().all(|posting| {
                snapshot
                    .account(posting.account())
                    .is_some_and(|account| account.institution() == input.institution)
            });
            if !matches_scope {
                return HandlerResult::DomainDenied(BankProposalDenial::ScopeInputMismatch);
            }
        }
        match BankProposalEngine::prepare_reverse_journal(
            &snapshot,
            bank_operation_scope_binding(reader.operation_scope_binding()),
            reader.idempotency_key(),
            input,
        ) {
            Ok(proposal) => HandlerResult::Completed(proposal),
            Err(denial) => HandlerResult::DomainDenied(denial),
        }
    }

    fn candidate_requirements(
        &self,
        _: &ReverseJournal,
        _: &BankInvariantApprovedProposal,
    ) -> ApplicationCandidateRequirements {
        ReverseJournalMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &ReverseJournal,
        decision: BankInvariantApprovedProposal,
        candidate: &mut CandidateWriter<'_, BankSchema, ReverseJournalMutationBinding>,
    ) -> HandlerResult<MoneyMovementResult, BankProposalDenial> {
        let [BankProposedEffect::ReverseJournal { original, reversal }] = decision.effects() else {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                InvalidReverseJournalCandidate,
            ));
        };
        if reversal.reversal_of() != Some(*original) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                InvalidReverseJournalCandidate,
            ));
        }
        let original = match candidate.resolve_entity(JournalIdentityField::reference(), *original)
        {
            Ok(original) => original,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        let reversal_entity =
            match author_journal(candidate, reversal, decision.proposed_snapshot()) {
                Ok(reversal) => reversal,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
        if let Err(error) = candidate.link(
            JournalReversal::reference(),
            format!("journal-reversal:{}", reversal.id().canonical_text()),
            &reversal_entity,
            &original,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(MoneyMovementResult {
            journal: reversal.id(),
        })
    }
}

#[derive(Debug)]
struct InvalidReverseJournalCandidate;

impl std::fmt::Display for InvalidReverseJournalCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("reverse-journal proposal has an invalid effect shape")
    }
}

impl std::error::Error for InvalidReverseJournalCandidate {}
