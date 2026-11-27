//! The approved-payment workflow's control steps: authoring, instance start,
//! advance and approval, each a `WORKFLOW_CONTROL` binding over `ApprovePayment`.
use worth_query_decl::facade::{
    application_operation::ApplicationMutationIntent, worth_query_workflow_controls,
};

use crate::proposals::BankProposalDenial;

use super::super::{
    ApprovePayment, ApprovePaymentInputBinding, ApprovePaymentMutationBinding, BankBindingContext,
    BankSchema, PaymentDecisionDenialBinding, PaymentDecisionResult, PaymentDecisionResultBinding,
    PaymentIdentityField, PaymentIntent,
};

worth_query_workflow_controls! {
    schema: BankSchema,
    binding_context: BankBindingContext,
    input: ApprovePayment => ApprovePaymentInputBinding,
    result: PaymentDecisionResult => PaymentDecisionResultBinding,
    denial: BankProposalDenial => PaymentDecisionDenialBinding,
    scope: PaymentIntent, field PaymentIdentityField = PaymentIdentityField::at("PaymentIdentityField"), input.payment,
    capability_context: pub ApprovedBusinessPaymentControlContext = "worth.bank.approved-business-payment-control-context.v1",
    grant: "manage-approved-business-payment-workflow" for "business-payment-approval",
    controls: [
        pub ApprovedBusinessPaymentApproval = "worth.bank.approved-business-payment-approval" {
            ApprovedBusinessPaymentApprovalOperation,
            ApprovedBusinessPaymentApprovalBinding,
            ApprovedBusinessPaymentApprovalIntent,
        },
        pub ApprovedBusinessPaymentAuthoring = "worth.bank.approved-business-payment-authoring" {
            ApprovedBusinessPaymentAuthoringOperation,
            ApprovedBusinessPaymentAuthoringBinding,
            ApprovedBusinessPaymentAuthoringIntent,
        },
        pub ApprovedBusinessPaymentInstanceStart = "worth.bank.approved-business-payment-instance-start" {
            ApprovedBusinessPaymentInstanceStartOperation,
            ApprovedBusinessPaymentInstanceStartBinding,
            ApprovedBusinessPaymentInstanceStartIntent,
        },
        pub ApprovedBusinessPaymentAdvance = "worth.bank.approved-business-payment-advance" {
            ApprovedBusinessPaymentAdvanceOperation,
            ApprovedBusinessPaymentAdvanceBinding,
            ApprovedBusinessPaymentAdvanceIntent,
        },
    ],
}

/// Applies the approved payment: the ordinary approval mutation, not a control step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedBusinessPaymentApplyIntent {
    pub input: ApprovePayment,
}

impl ApplicationMutationIntent<BankSchema> for ApprovedBusinessPaymentApplyIntent {
    type Binding = ApprovePaymentMutationBinding;

    fn input(&self) -> &ApprovePayment {
        &self.input
    }

    fn scope_binding(
        &self,
    ) -> worth_query_decl::facade::application_operation::ApplicationMutationFieldScope<
        PaymentIdentityField,
    > {
        worth_query_decl::facade::application_operation::ApplicationMutationFieldScope::new(
            PaymentIdentityField::at("PaymentIdentityField"),
            self.input.payment,
        )
    }
}
