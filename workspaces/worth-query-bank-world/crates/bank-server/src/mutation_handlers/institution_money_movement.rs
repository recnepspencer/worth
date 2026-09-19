use bank_domain::proposals::{
    BankInvariantApprovedProposal, BankProposalDenial, BankProposalEngine, BankProposedEffect,
};
use bank_domain::schema::{
    ApplyOpeningFunding, ApplyOpeningFundingMutationBinding, BankSchema, Deposit,
    DepositMutationBinding, MoneyMovementResult, Withdraw, WithdrawMutationBinding,
};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};

use super::journal::author_journal;
use crate::bank_projection::project_institution_money_movement;
use crate::operation_admission::bank_operation_scope_binding;

pub(crate) struct ApplyOpeningFundingHandler;
pub(crate) struct DepositHandler;
pub(crate) struct WithdrawHandler;

macro_rules! institution_movement_handler {
    ($Handler:ty, $Binding:ty, $Input:ty, $balance:expr, $prepare:ident) => {
        impl OperationHandler<BankSchema, $Binding> for $Handler {
            fn decide(
                &self,
                input: &$Input,
                reader: &mut DecisionReader<'_, '_, '_, BankSchema, $Binding>,
            ) -> HandlerResult<BankInvariantApprovedProposal, BankProposalDenial> {
                let institution = reader.scope().clone();
                let decision = match project_institution_money_movement(
                    reader.reader(),
                    &institution,
                    input.institution,
                    input.account,
                    $balance(input),
                ) {
                    Ok(decision) => decision,
                    Err(error) => return execution_denied(error),
                };
                match BankProposalEngine::$prepare(
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
                _: &$Input,
                _: &BankInvariantApprovedProposal,
            ) -> ApplicationCandidateRequirements {
                <$Binding>::CANDIDATES
            }

            fn build_candidate(
                &self,
                _: &$Input,
                decision: BankInvariantApprovedProposal,
                candidate: &mut CandidateWriter<'_, BankSchema, $Binding>,
            ) -> HandlerResult<MoneyMovementResult, BankProposalDenial> {
                match author_movement(&decision, candidate) {
                    Ok(result) => HandlerResult::Completed(result),
                    Err(error) => HandlerResult::ExecutionDenied(error),
                }
            }
        }
    };
}

institution_movement_handler!(
    ApplyOpeningFundingHandler,
    ApplyOpeningFundingMutationBinding,
    ApplyOpeningFunding,
    |_| [],
    prepare_opening_funding_from_decision
);
institution_movement_handler!(
    DepositHandler,
    DepositMutationBinding,
    Deposit,
    |_| [],
    prepare_deposit_from_decision
);
institution_movement_handler!(
    WithdrawHandler,
    WithdrawMutationBinding,
    Withdraw,
    |input: &Withdraw| [input.account],
    prepare_withdrawal_from_decision
);

fn author_movement<Binding>(
    proposal: &BankInvariantApprovedProposal,
    candidate: &mut CandidateWriter<'_, BankSchema, Binding>,
) -> Result<MoneyMovementResult, HandlerExecutionDenial>
where
    Binding: ApplicationMutationBinding<BankSchema>,
    bank_domain::schema::AccountIdentity:
        worth_query_host::facade::domain::OperationReads<Binding::Operation>,
    bank_domain::schema::JournalEntry:
        worth_query_host::facade::domain::OperationCreates<Binding::Operation>,
    bank_domain::schema::Posting:
        worth_query_host::facade::domain::OperationCreates<Binding::Operation>,
    bank_domain::schema::JournalIdentityField:
        worth_query_host::facade::domain::OperationWrites<Binding::Operation>,
    bank_domain::schema::JournalPurpose:
        worth_query_host::facade::domain::OperationWrites<Binding::Operation>,
    bank_domain::schema::PostingIdentityField:
        worth_query_host::facade::domain::OperationWrites<Binding::Operation>,
    bank_domain::schema::PostingAmount:
        worth_query_host::facade::domain::OperationWrites<Binding::Operation>,
    bank_domain::schema::PostingAccountSequence:
        worth_query_host::facade::domain::OperationWrites<Binding::Operation>,
    bank_domain::schema::Purpose:
        worth_query_host::facade::domain::OperationWrites<Binding::Operation>,
    bank_domain::schema::AccountingRevision:
        worth_query_host::facade::domain::OperationWrites<Binding::Operation>,
    bank_domain::schema::JournalPosting:
        worth_query_host::facade::domain::OperationLinks<Binding::Operation>,
    bank_domain::schema::PostingAccount:
        worth_query_host::facade::domain::OperationLinks<Binding::Operation>,
    bank_domain::schema::AccountActivityEffect:
        worth_query_host::facade::declaration::application_schema::ApplicationEffectMarkerIdentity<BankSchema>
        + worth_query_host::facade::declaration::application_schema::OperationEmits<Binding::Operation>,
{
    let [BankProposedEffect::AppendJournal(journal)] = proposal.effects() else {
        return Err(HandlerExecutionDenial::new(InvalidMoneyMovementCandidate));
    };
    author_journal(candidate, journal, proposal.proposed_snapshot())?;
    Ok(MoneyMovementResult {
        journal: journal.id(),
    })
}

fn execution_denied<Value, Denial>(
    error: impl std::error::Error + Send + Sync + 'static,
) -> HandlerResult<Value, Denial> {
    HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
}

#[derive(Debug)]
struct InvalidMoneyMovementCandidate;

impl std::fmt::Display for InvalidMoneyMovementCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("money movement proposal has an invalid effect shape")
    }
}

impl std::error::Error for InvalidMoneyMovementCandidate {}
