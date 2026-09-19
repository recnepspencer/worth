use bank_domain::{
    estate::EstateAction,
    proposals::{
        BankIdempotencyClaim, BankInvariantApprovedProposal, BankProposalEngine, BankProposedEffect,
    },
    schema::{
        BankSchema, DisburseEstateMutationBinding, EstateMutationDenial, EstateMutationResult,
    },
};
use worth_query_host::facade::{
    declaration::application_operation::{
        ApplicationCandidateRequirements, ApplicationMutationBinding,
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    },
};

use super::journal::author_journal;

pub(crate) struct DisburseEstateHandler;

impl OperationHandler<BankSchema, DisburseEstateMutationBinding> for DisburseEstateHandler {
    fn decide(
        &self,
        input: &EstateAction,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, DisburseEstateMutationBinding>,
    ) -> HandlerResult<BankInvariantApprovedProposal, EstateMutationDenial> {
        let EstateAction::DisburseEstate(disbursement) = input else {
            return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant);
        };
        let estate = reader.scope().clone();
        let decision = match crate::bank_projection::project_estate_disbursement(
            reader.reader(),
            &estate,
            disbursement,
        ) {
            Ok(decision) => decision,
            Err(denial) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
            }
        };
        let (snapshot, _, _) = decision.into_parts();
        let idempotency = BankIdempotencyClaim::from_application_binding(
            DisburseEstateMutationBinding::idempotency_key_identity(reader.idempotency_key()),
            DisburseEstateMutationBinding::input_identity(input),
        );
        match BankProposalEngine::prepare_estate_disbursement_from_decision(
            snapshot,
            idempotency,
            disbursement,
        ) {
            Ok(proposal) => HandlerResult::Completed(proposal),
            Err(denial) => HandlerResult::DomainDenied(EstateMutationDenial::Proposal(denial)),
        }
    }

    fn candidate_requirements(
        &self,
        _: &EstateAction,
        _: &BankInvariantApprovedProposal,
    ) -> ApplicationCandidateRequirements {
        DisburseEstateMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &EstateAction,
        decision: BankInvariantApprovedProposal,
        candidate: &mut CandidateWriter<'_, BankSchema, DisburseEstateMutationBinding>,
    ) -> HandlerResult<EstateMutationResult, EstateMutationDenial> {
        let EstateAction::DisburseEstate(disbursement) = input else {
            return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant);
        };
        let [BankProposedEffect::AppendJournal(journal)] = decision.effects() else {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                InvalidEstateDisbursementCandidate,
            ));
        };
        if let Err(denial) = author_journal(candidate, journal, decision.proposed_snapshot()) {
            return HandlerResult::ExecutionDenied(denial);
        }
        HandlerResult::Completed(EstateMutationResult {
            estate: disbursement.estate,
        })
    }
}

#[derive(Debug)]
struct InvalidEstateDisbursementCandidate;

impl std::fmt::Display for InvalidEstateDisbursementCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("estate disbursement produced an invalid journal candidate")
    }
}

impl std::error::Error for InvalidEstateDisbursementCandidate {}
