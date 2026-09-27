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
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    },
};

/// The handler every approved-payment workflow control binding must install.
///
/// Query installs a handler for each mutation binding, but the workflow kernel
/// authorizes and records authoring, start, advance and approval itself and
/// never calls one. The approver named in the input is checked where it
/// matters: the payment operation's own handler compares it with the
/// authenticated principal before any effect. Should a kernel ever call this
/// handler, it refuses rather than complete a step it never checked.
pub(crate) struct ApprovedBusinessPaymentControlHandler;

#[derive(Debug)]
struct KernelRecordedControlStep;

impl std::fmt::Display for KernelRecordedControlStep {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("the workflow kernel records approved-payment control steps itself")
    }
}

impl std::error::Error for KernelRecordedControlStep {}

fn kernel_recorded<Value>() -> HandlerResult<Value, BankProposalDenial> {
    HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(KernelRecordedControlStep))
}

macro_rules! control_handler {
    ($binding:ty) => {
        impl OperationHandler<BankSchema, $binding> for ApprovedBusinessPaymentControlHandler {
            fn decide(
                &self,
                _: &ApprovePayment,
                _: &mut DecisionReader<'_, '_, '_, BankSchema, $binding>,
            ) -> HandlerResult<(), BankProposalDenial> {
                kernel_recorded()
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
                _: &ApprovePayment,
                _: (),
                _: &mut CandidateWriter<'_, BankSchema, $binding>,
            ) -> HandlerResult<PaymentDecisionResult, BankProposalDenial> {
                kernel_recorded()
            }
        }
    };
}

control_handler!(ApprovedBusinessPaymentAuthoringBinding);
control_handler!(ApprovedBusinessPaymentApprovalBinding);
control_handler!(ApprovedBusinessPaymentInstanceStartBinding);
control_handler!(ApprovedBusinessPaymentAdvanceBinding);
