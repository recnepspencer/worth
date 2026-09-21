use bank_domain::{
    proposals::BankProposalDenial,
    schema::{
        ApprovePayment, ApprovedBusinessPaymentAdvanceBinding,
        ApprovedBusinessPaymentApprovalBinding, ApprovedBusinessPaymentAuthoringBinding,
        ApprovedBusinessPaymentInstanceStartBinding, BankSchema, PaymentDecisionResult,
    },
};
use worth_query_host::facade::{
    declaration::application_operation::{
        ApplicationCandidateRequirements, ApplicationMutationBinding,
    },
    primary_graph::{CandidateWriter, DecisionReader, HandlerResult, OperationHandler},
};

pub(crate) struct ApprovedBusinessPaymentControlHandler;

macro_rules! control_handler {
    ($binding:ty) => {
        impl OperationHandler<BankSchema, $binding> for ApprovedBusinessPaymentControlHandler {
            fn decide(
                &self,
                input: &ApprovePayment,
                reader: &mut DecisionReader<'_, '_, '_, BankSchema, $binding>,
            ) -> HandlerResult<(), BankProposalDenial> {
                if *reader.principal_identity() != input.approver {
                    return HandlerResult::DomainDenied(
                        BankProposalDenial::AuthenticatedActorMismatch,
                    );
                }
                HandlerResult::Completed(())
            }

            fn candidate_requirements(
                &self,
                _: &ApprovePayment,
                _: &(),
            ) -> ApplicationCandidateRequirements {
                <$binding>::CANDIDATES
            }

            fn build_candidate(
                &self,
                input: &ApprovePayment,
                _: (),
                _: &mut CandidateWriter<'_, BankSchema, $binding>,
            ) -> HandlerResult<PaymentDecisionResult, BankProposalDenial> {
                HandlerResult::Completed(PaymentDecisionResult {
                    payment: input.payment,
                })
            }
        }
    };
}

control_handler!(ApprovedBusinessPaymentAuthoringBinding);
control_handler!(ApprovedBusinessPaymentApprovalBinding);
control_handler!(ApprovedBusinessPaymentInstanceStartBinding);
control_handler!(ApprovedBusinessPaymentAdvanceBinding);
