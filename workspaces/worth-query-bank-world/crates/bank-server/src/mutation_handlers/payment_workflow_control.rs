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

/// The handler every approved-payment workflow control binding must install.
///
/// Query installs a handler for each mutation binding, but the workflow kernel
/// authorizes and records authoring, start, advance and approval itself and
/// never calls one. The approver named in the input is checked where it
/// matters: the payment operation's own handler compares it with the
/// authenticated principal before any effect.
pub(crate) struct ApprovedBusinessPaymentControlHandler;

macro_rules! control_handler {
    ($binding:ty) => {
        impl OperationHandler<BankSchema, $binding> for ApprovedBusinessPaymentControlHandler {
            fn decide(
                &self,
                _: &ApprovePayment,
                _: &mut DecisionReader<'_, '_, '_, BankSchema, $binding>,
            ) -> HandlerResult<(), BankProposalDenial> {
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
