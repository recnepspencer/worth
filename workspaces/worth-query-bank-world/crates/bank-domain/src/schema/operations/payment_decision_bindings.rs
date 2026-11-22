use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
        WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
        WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
    },
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::PaymentId;
use crate::proposals::{BankInvariantApprovedProposal, BankProposalDenial};

use super::{ApprovePayment, ApprovePaymentInputBinding, RejectPayment, RejectPaymentInputBinding};
use crate::schema::{
    ApprovePaymentOperation, BankSchema, PaymentIdentityField, PaymentIntent,
    RejectPaymentOperation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaymentDecisionResult {
    pub payment: PaymentId,
}

worth_query_structured_value_binding!(
    pub PaymentDecisionResultBinding for PaymentDecisionResult {
        identity: "bank.operation.payment-decision.result.v1"
    }
);

worth_query_structured_value_binding!(
    pub PaymentDecisionDenialBinding for BankProposalDenial {
        identity: "bank.operation.payment-decision.denial.v1"
    }
);

pub struct PaymentDecisionOutputs;

/// The rejected payment and its admitted readiness definition publish together.
pub struct RejectedPaymentDecision {
    pub proposal: BankInvariantApprovedProposal,
    pub definition:
        worth_query_host::facade::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition,
}

/// The payment intent preserved by a rejection.
pub struct DecidedPaymentOutput;

impl WorthQueryApplicationOutputRole for DecidedPaymentOutput {
    type Schema = BankSchema;
    type Contract = PaymentDecisionOutputs;
    type Entity = PaymentIntent;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "payment";
}

impl ApplicationMutationOutputContract<BankSchema> for PaymentDecisionOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<DecidedPaymentOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}

fn payment_scope_from_approval(input: &ApprovePayment) -> PaymentId {
    input.payment
}

fn payment_scope_from_rejection(input: &RejectPayment) -> PaymentId {
    input.payment
}

worth_query_mutation_binding!(
    pub ApprovePaymentMutationBinding for ApprovePayment, schema BankSchema, context crate::schema::BankBindingContext,
    identity "bank.operation.approve-payment.mutation-binding.v1",
    input ApprovePaymentInputBinding,
    operation ApprovePaymentOperation,
    result PaymentDecisionResultBinding,
    decision BankInvariantApprovedProposal,
    denial PaymentDecisionDenialBinding,
    handler identity "bank.operation.approve-payment.handler.v1",
    program required,
    workflow_authority required,
    workflow_requirements worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::requiring_approval::<crate::schema::workflow::ApprovedBusinessPaymentApproval>(),
    outputs super::ApprovedPaymentOutputs,
    scope PaymentIdentityField,
    field PaymentIdentityField::at("PaymentIdentityField"),
    value payment_scope_from_approval,
);

worth_query_mutation_binding!(
    pub RejectPaymentMutationBinding for RejectPayment, schema BankSchema, context crate::schema::BankBindingContext,
    identity "bank.operation.reject-payment.mutation-binding.v1",
    input RejectPaymentInputBinding,
    operation RejectPaymentOperation,
    result PaymentDecisionResultBinding,
    decision RejectedPaymentDecision,
    denial PaymentDecisionDenialBinding,
    handler identity "bank.operation.reject-payment.handler.v1",
    program required,
    outputs PaymentDecisionOutputs,
    scope PaymentIdentityField,
    field PaymentIdentityField::at("PaymentIdentityField"),
    value payment_scope_from_rejection,
);
