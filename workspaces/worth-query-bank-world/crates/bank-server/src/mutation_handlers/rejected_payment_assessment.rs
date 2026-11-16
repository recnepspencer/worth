use bank_domain::schema::{
    BankSchema, PaymentIdentityField, PaymentIntent, PaymentStatusField,
    RejectedPaymentAssessmentBinding, RejectedPaymentAssessmentDenial,
    RejectedPaymentAssessmentInput, RejectedPaymentAssessmentOutput,
    RejectedPaymentAssessmentPublished,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

pub(crate) struct RejectedPaymentAssessmentHandler;

impl OperationHandler<BankSchema, RejectedPaymentAssessmentBinding>
    for RejectedPaymentAssessmentHandler
{
    fn declared_durable_capabilities() -> Option<
        &'static [worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability],
    > {
        use worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability as Capability;
        Some(&[Capability::RelationalMutation])
    }

    fn decide(
        &self,
        input: &RejectedPaymentAssessmentInput,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, RejectedPaymentAssessmentBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<BankSchema, PaymentIntent>,
        RejectedPaymentAssessmentDenial,
    > {
        let payment = match reader.resolve_entity(
            PaymentIdentityField::at("PaymentIdentityField"),
            input.payment,
        ) {
            Ok(payment) => payment,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&payment, PaymentStatusField::at("PaymentStatusField")) {
            Ok(Some(status)) if status == input.status => {}
            Ok(_) => {
                return HandlerResult::DomainDenied(RejectedPaymentAssessmentDenial::SourceChanged)
            }
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&payment) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn build_candidate(
        &self,
        input: &RejectedPaymentAssessmentInput,
        target: WorthQueryInvariantMutationTarget<BankSchema, PaymentIntent>,
        writer: &mut CandidateWriter<'_, BankSchema, RejectedPaymentAssessmentBinding>,
    ) -> HandlerResult<RejectedPaymentAssessmentPublished, RejectedPaymentAssessmentDenial> {
        let payment = match writer.projected_entity(&target) {
            Ok(payment) => payment,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.preserve_output::<RejectedPaymentAssessmentOutput>(&payment) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(RejectedPaymentAssessmentPublished {
            status: input.status,
        })
    }
}
