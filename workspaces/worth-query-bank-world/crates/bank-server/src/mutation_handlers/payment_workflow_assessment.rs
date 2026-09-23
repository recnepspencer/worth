use bank_domain::schema::{
    ApprovedPaymentAssessmentBinding, ApprovedPaymentAssessmentDenial,
    ApprovedPaymentAssessmentInput, ApprovedPaymentAssessmentPublished, BankSchema,
    PaymentIdentityField, PaymentIntent, PaymentStatusField,
};
use worth_query_host::facade::{
    declaration::application_operation::{
        ApplicationCandidateRequirements, ApplicationMutationBinding,
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryApplicationOutputRole, WorthQueryInvariantMutationTarget,
    },
};

pub(crate) struct ApprovedPaymentAssessmentHandler;

impl OperationHandler<BankSchema, ApprovedPaymentAssessmentBinding>
    for ApprovedPaymentAssessmentHandler
{
    fn decide(
        &self,
        input: &ApprovedPaymentAssessmentInput,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, ApprovedPaymentAssessmentBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<BankSchema, PaymentIntent>,
        ApprovedPaymentAssessmentDenial,
    > {
        let payment = match reader.resolve_entity(PaymentIdentityField::reference(), input.payment)
        {
            Ok(payment) => payment,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&payment, PaymentStatusField::reference()) {
            Ok(Some(status)) if status == input.status => {}
            Ok(_) => {
                return HandlerResult::DomainDenied(ApprovedPaymentAssessmentDenial::SourceChanged)
            }
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&payment) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &ApprovedPaymentAssessmentInput,
        _: &WorthQueryInvariantMutationTarget<BankSchema, PaymentIntent>,
    ) -> ApplicationCandidateRequirements {
        ApprovedPaymentAssessmentBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &ApprovedPaymentAssessmentInput,
        target: WorthQueryInvariantMutationTarget<BankSchema, PaymentIntent>,
        writer: &mut CandidateWriter<'_, BankSchema, ApprovedPaymentAssessmentBinding>,
    ) -> HandlerResult<ApprovedPaymentAssessmentPublished, ApprovedPaymentAssessmentDenial> {
        let payment = match writer.projected_entity(&target) {
            Ok(payment) => payment,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.preserve_output(
            WorthQueryApplicationOutputRole::from_static("assessment"),
            &payment,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(ApprovedPaymentAssessmentPublished {
            status: input.status,
        })
    }
}
