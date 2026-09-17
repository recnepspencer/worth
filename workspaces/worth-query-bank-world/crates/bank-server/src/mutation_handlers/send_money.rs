use bank_domain::proposals::{
    BankInvariantApprovedProposal, BankProposalDenial, BankProposalEngine, BankProposedEffect,
};
use bank_domain::schema::{BankSchema, MoneyMovementResult, SendMoney, SendMoneyMutationBinding};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};

use super::journal::author_journal;
use crate::bank_projection::project_send_money_decision;
use crate::operation_admission::bank_operation_scope_binding;

pub(crate) struct SendMoneyHandler;

impl OperationHandler<BankSchema, SendMoneyMutationBinding> for SendMoneyHandler {
    fn decide(
        &self,
        input: &SendMoney,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, SendMoneyMutationBinding>,
    ) -> HandlerResult<BankInvariantApprovedProposal, BankProposalDenial> {
        let source = reader.scope().clone();
        let decision = match project_send_money_decision(reader.reader(), &source, input) {
            Ok(decision) => decision,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        match BankProposalEngine::prepare_send_money_from_decision(
            decision,
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
        _: &SendMoney,
        _: &BankInvariantApprovedProposal,
    ) -> ApplicationCandidateRequirements {
        SendMoneyMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &SendMoney,
        decision: BankInvariantApprovedProposal,
        candidate: &mut CandidateWriter<'_, BankSchema, SendMoneyMutationBinding>,
    ) -> HandlerResult<MoneyMovementResult, BankProposalDenial> {
        let [BankProposedEffect::AppendJournal(journal)] = decision.effects() else {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                InvalidSendMoneyCandidate,
            ));
        };
        if let Err(error) = author_journal(candidate, journal, decision.proposed_snapshot()) {
            return HandlerResult::ExecutionDenied(error);
        }
        HandlerResult::Completed(MoneyMovementResult {
            journal: journal.id(),
        })
    }
}

#[derive(Debug)]
struct InvalidSendMoneyCandidate;

impl std::fmt::Display for InvalidSendMoneyCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("send-money proposal has an invalid effect shape")
    }
}

impl std::error::Error for InvalidSendMoneyCandidate {}
