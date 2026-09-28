use bank_domain::proposals::{BankProposalDenial, BankProposalEngine};
use bank_domain::schema::{
    initiate_business_payment_application_idempotency, AccountIdentity, BankSchema,
    BusinessIdentityField, InitiateBusinessPayment, InitiateBusinessPaymentDecision,
    InitiateBusinessPaymentMutationBinding, InitiateBusinessPaymentResult, PaymentAmount,
    PaymentBusiness, PaymentDestination, PaymentIdentityField, PaymentInitiator, PaymentIntent,
    PaymentSource, PaymentStatusField, PrincipalIdentityField,
    INITIATE_BUSINESS_PAYMENT_OUTPUT_PAYMENT,
};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerInterruption, HandlerResult,
    OperationHandler, WorthQueryApplicationEntityKey, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput,
};

use super::payment_approval_grant::author_approval_grant;
use crate::bank_projection::project_business_payment_initiation;
use crate::graph_bootstrap::payment_key;
use crate::operation_admission::bank_operation_scope_binding;

pub(crate) struct InitiateBusinessPaymentHandler;

impl OperationHandler<BankSchema, InitiateBusinessPaymentMutationBinding>
    for InitiateBusinessPaymentHandler
{
    fn decide(
        &self,
        input: &InitiateBusinessPayment,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, InitiateBusinessPaymentMutationBinding>,
    ) -> HandlerResult<InitiateBusinessPaymentDecision, BankProposalDenial> {
        if let Err(interruption) = reader.checkpoint() {
            return interrupted(interruption);
        }
        let actor = *reader.principal_identity();
        let business = reader.scope().clone();
        let snapshot =
            match project_business_payment_initiation(reader.reader(), &business, actor, input) {
                Ok(snapshot) => snapshot,
                Err(error) => return execution_denied(error),
            };
        let idempotency = initiate_business_payment_application_idempotency(
            bank_operation_scope_binding(reader.operation_scope_binding()),
            reader.idempotency_key(),
            input,
        );
        match BankProposalEngine::decide_initiate_business_payment_from_application(
            &snapshot,
            idempotency,
            actor,
            input,
        ) {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(denial) => HandlerResult::DomainDenied(denial),
        }
    }

    fn candidate_requirements(
        &self,
        _: &InitiateBusinessPayment,
        _: &InitiateBusinessPaymentDecision,
    ) -> ApplicationCandidateRequirements {
        InitiateBusinessPaymentMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &InitiateBusinessPayment,
        decision: InitiateBusinessPaymentDecision,
        candidate: &mut CandidateWriter<'_, BankSchema, InitiateBusinessPaymentMutationBinding>,
    ) -> HandlerResult<InitiateBusinessPaymentResult, BankProposalDenial> {
        if let Err(interruption) = candidate.checkpoint() {
            return interrupted(interruption);
        }
        match author_candidate(decision, candidate) {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

fn author_candidate(
    decision: InitiateBusinessPaymentDecision,
    candidate: &mut CandidateWriter<'_, BankSchema, InitiateBusinessPaymentMutationBinding>,
) -> Result<InitiateBusinessPaymentResult, HandlerExecutionDenial> {
    let (payment, approval_grantees) = decision.into_parts();
    let business = candidate
        .resolve_entity(BusinessIdentityField::reference(), payment.business())
        .map_err(HandlerExecutionDenial::new)?;
    let source = candidate
        .resolve_entity(AccountIdentity::reference(), payment.source())
        .map_err(HandlerExecutionDenial::new)?;
    let destination = candidate
        .resolve_entity(AccountIdentity::reference(), payment.destination())
        .map_err(HandlerExecutionDenial::new)?;
    let initiator = candidate
        .resolve_entity(PrincipalIdentityField::reference(), payment.initiator())
        .map_err(HandlerExecutionDenial::new)?;
    let key = WorthQueryApplicationEntityKey::<BankSchema, PaymentIntent>::new(
        payment_key(payment.id()).into_boxed_str().into_string(),
    )
    .map_err(HandlerExecutionDenial::new)?;
    let created = candidate
        .create_entity_in_context(&business, PaymentIntent::reference(), key)
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(&created, PaymentIdentityField::reference(), payment.id())
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(&created, PaymentAmount::reference(), payment.amount())
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(&created, PaymentStatusField::reference(), payment.status())
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            PaymentSource::reference(),
            format!("payment-source:{}", payment.id().canonical_text()),
            &created,
            &source,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            PaymentDestination::reference(),
            format!("payment-destination:{}", payment.id().canonical_text()),
            &created,
            &destination,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            PaymentBusiness::reference(),
            format!("payment-business:{}", payment.id().canonical_text()),
            &created,
            &business,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            PaymentInitiator::reference(),
            format!("payment-initiator:{}", payment.id().canonical_text()),
            &initiator,
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    for grantee in approval_grantees {
        author_approval_grant(
            candidate, &payment, &business, &created, &initiator, grantee,
        )?;
    }
    candidate
        .create_output(
            WorthQueryApplicationOutputRole::<
                InitiateBusinessPaymentMutationBinding,
                PaymentIntent,
                WorthQueryCreateOutput,
            >::from_static(INITIATE_BUSINESS_PAYMENT_OUTPUT_PAYMENT),
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(InitiateBusinessPaymentResult {
        payment: payment.id(),
    })
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
