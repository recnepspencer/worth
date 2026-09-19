use bank_domain::payments::BusinessPayment;
use bank_domain::proposals::{
    BankInvariantApprovedProposal, BankProposalDenial, BankProposalEngine, BankProposedEffect,
};
use bank_domain::schema::{
    Approval, ApprovalPrincipal, ApprovePayment, ApprovePaymentMutationBinding, BankSchema,
    PaymentApproval, PaymentDecisionResult, PaymentIdentityField, PaymentIntent,
    PaymentStatusField, PrincipalIdentityField, RejectPayment, RejectPaymentMutationBinding,
    PAYMENT_DECISION_OUTPUT_PAYMENT,
};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_host::facade::domain::{
    OperationCreates, OperationLinks, OperationReads, OperationWrites,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerInterruption, HandlerResult,
    OperationHandler, WorthQueryApplicationEffectEntity, WorthQueryApplicationEntityKey,
    WorthQueryApplicationOutputRole, WorthQueryPreserveOutput,
};

use super::journal::author_journal;
use crate::bank_projection::{project_payment_approval, project_payment_rejection};
use crate::graph_bootstrap::approval_key;
use crate::operation_admission::bank_operation_scope_binding;

pub(crate) struct ApprovePaymentHandler;

impl OperationHandler<BankSchema, ApprovePaymentMutationBinding> for ApprovePaymentHandler {
    fn decide(
        &self,
        input: &ApprovePayment,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, ApprovePaymentMutationBinding>,
    ) -> HandlerResult<BankInvariantApprovedProposal, BankProposalDenial> {
        if let Err(interruption) = reader.checkpoint() {
            return interrupted(interruption);
        }
        if *reader.principal_identity() != input.approver {
            return HandlerResult::DomainDenied(BankProposalDenial::AuthenticatedActorMismatch);
        }
        let payment = reader.scope().clone();
        let decision = match project_payment_approval(reader.reader(), &payment, input) {
            Ok(decision) => decision,
            Err(error) => return execution_denied(error),
        };
        match BankProposalEngine::prepare_approve_payment_from_decision(
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
        _: &ApprovePayment,
        _: &BankInvariantApprovedProposal,
    ) -> ApplicationCandidateRequirements {
        ApprovePaymentMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &ApprovePayment,
        decision: BankInvariantApprovedProposal,
        candidate: &mut CandidateWriter<'_, BankSchema, ApprovePaymentMutationBinding>,
    ) -> HandlerResult<PaymentDecisionResult, BankProposalDenial> {
        if let Err(interruption) = candidate.checkpoint() {
            return interrupted(interruption);
        }
        match author_approval_candidate(&decision, candidate) {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

pub(crate) struct RejectPaymentHandler;

impl OperationHandler<BankSchema, RejectPaymentMutationBinding> for RejectPaymentHandler {
    fn decide(
        &self,
        input: &RejectPayment,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, RejectPaymentMutationBinding>,
    ) -> HandlerResult<BankInvariantApprovedProposal, BankProposalDenial> {
        if let Err(interruption) = reader.checkpoint() {
            return interrupted(interruption);
        }
        if *reader.principal_identity() != input.rejecting_principal {
            return HandlerResult::DomainDenied(BankProposalDenial::AuthenticatedActorMismatch);
        }
        let payment = reader.scope().clone();
        let decision = match project_payment_rejection(reader.reader(), &payment, input) {
            Ok(decision) => decision,
            Err(error) => return execution_denied(error),
        };
        match BankProposalEngine::prepare_reject_payment_from_decision(
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
        _: &RejectPayment,
        _: &BankInvariantApprovedProposal,
    ) -> ApplicationCandidateRequirements {
        RejectPaymentMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &RejectPayment,
        decision: BankInvariantApprovedProposal,
        candidate: &mut CandidateWriter<'_, BankSchema, RejectPaymentMutationBinding>,
    ) -> HandlerResult<PaymentDecisionResult, BankProposalDenial> {
        if let Err(interruption) = candidate.checkpoint() {
            return interrupted(interruption);
        }
        match author_rejection_candidate(&decision, candidate) {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

fn author_approval_candidate(
    proposal: &BankInvariantApprovedProposal,
    candidate: &mut CandidateWriter<'_, BankSchema, ApprovePaymentMutationBinding>,
) -> Result<PaymentDecisionResult, HandlerExecutionDenial> {
    let (journal, payment) = exact_approved_payment(proposal.effects())?;
    author_journal(candidate, journal, proposal.proposed_snapshot())?;
    let payment_entity = author_payment_decision(candidate, payment)?;
    candidate
        .preserve_output(
            WorthQueryApplicationOutputRole::<
                ApprovePaymentMutationBinding,
                PaymentIntent,
                WorthQueryPreserveOutput,
            >::from_static(PAYMENT_DECISION_OUTPUT_PAYMENT),
            &payment_entity,
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(PaymentDecisionResult {
        payment: payment.id(),
    })
}

fn author_rejection_candidate(
    proposal: &BankInvariantApprovedProposal,
    candidate: &mut CandidateWriter<'_, BankSchema, RejectPaymentMutationBinding>,
) -> Result<PaymentDecisionResult, HandlerExecutionDenial> {
    let payment = exact_updated_payment(proposal.effects())?;
    let payment_entity = author_payment_decision(candidate, payment)?;
    candidate
        .preserve_output(
            WorthQueryApplicationOutputRole::<
                RejectPaymentMutationBinding,
                PaymentIntent,
                WorthQueryPreserveOutput,
            >::from_static(PAYMENT_DECISION_OUTPUT_PAYMENT),
            &payment_entity,
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(PaymentDecisionResult {
        payment: payment.id(),
    })
}

fn author_payment_decision<Binding>(
    candidate: &mut CandidateWriter<'_, BankSchema, Binding>,
    replacement: &BusinessPayment,
) -> Result<WorthQueryApplicationEffectEntity<BankSchema, PaymentIntent>, HandlerExecutionDenial>
where
    Binding: ApplicationMutationBinding<BankSchema>,
    Approval: OperationCreates<Binding::Operation>,
    PaymentIdentityField: OperationReads<Binding::Operation>,
    PrincipalIdentityField: OperationReads<Binding::Operation>,
    PaymentStatusField: OperationWrites<Binding::Operation>,
    PaymentApproval: OperationLinks<Binding::Operation>,
    ApprovalPrincipal: OperationLinks<Binding::Operation>,
{
    let payment = candidate
        .resolve_entity(PaymentIdentityField::reference(), replacement.id())
        .map_err(HandlerExecutionDenial::new)?;
    let decider = replacement
        .deciding_principal()
        .ok_or_else(|| HandlerExecutionDenial::new(InvalidPaymentDecisionCandidate))?;
    let decider = candidate
        .resolve_entity(PrincipalIdentityField::reference(), decider)
        .map_err(HandlerExecutionDenial::new)?;
    let approval = candidate
        .create_entity_in_context(
            &payment,
            Approval::reference(),
            WorthQueryApplicationEntityKey::new(approval_key(replacement.id()))
                .map_err(HandlerExecutionDenial::new)?,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .write_field(
            &payment,
            PaymentStatusField::reference(),
            replacement.status(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            PaymentApproval::reference(),
            format!("payment-approval:{}", replacement.id().canonical_text()),
            &payment,
            &approval,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            ApprovalPrincipal::reference(),
            format!("approval-principal:{}", replacement.id().canonical_text()),
            &approval,
            &decider,
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(payment)
}

fn exact_approved_payment(
    effects: &[BankProposedEffect],
) -> Result<(&bank_domain::accounting::BankJournalEntry, &BusinessPayment), HandlerExecutionDenial>
{
    let [BankProposedEffect::AppendJournal(journal), BankProposedEffect::UpdatePayment {
        payment,
        replacement,
    }] = effects
    else {
        return Err(HandlerExecutionDenial::new(InvalidPaymentDecisionCandidate));
    };
    if *payment != replacement.id() {
        return Err(HandlerExecutionDenial::new(InvalidPaymentDecisionCandidate));
    }
    Ok((journal, replacement))
}

fn exact_updated_payment(
    effects: &[BankProposedEffect],
) -> Result<&BusinessPayment, HandlerExecutionDenial> {
    let [BankProposedEffect::UpdatePayment {
        payment,
        replacement,
    }] = effects
    else {
        return Err(HandlerExecutionDenial::new(InvalidPaymentDecisionCandidate));
    };
    if *payment != replacement.id() {
        return Err(HandlerExecutionDenial::new(InvalidPaymentDecisionCandidate));
    }
    Ok(replacement)
}

fn execution_denied<Value, Denial>(
    error: impl std::error::Error + Send + Sync + 'static,
) -> HandlerResult<Value, Denial> {
    HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
}

fn interrupted<Value, Denial>(interruption: HandlerInterruption) -> HandlerResult<Value, Denial> {
    match interruption {
        HandlerInterruption::Cancelled => HandlerResult::Cancelled,
        HandlerInterruption::DeadlineExceeded => HandlerResult::DeadlineExceeded,
    }
}

#[derive(Debug)]
struct InvalidPaymentDecisionCandidate;

impl std::fmt::Display for InvalidPaymentDecisionCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("payment decision proposal has an invalid effect shape")
    }
}

impl std::error::Error for InvalidPaymentDecisionCandidate {}
