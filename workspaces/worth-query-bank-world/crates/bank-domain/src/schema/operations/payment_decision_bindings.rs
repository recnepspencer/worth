use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputPosture,
        ApplicationMutationOutputRoleDescriptor,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{BankPrincipalId, PaymentId};
use crate::proposals::{
    BankIdempotencyKey, BankInvariantApprovedProposal, BankProposalDenial, CanonicalProposalPayload,
};

use super::create_personal_account_binding::client_key_identity;
use super::{ApprovePayment, ApprovePaymentInputBinding, RejectPayment, RejectPaymentInputBinding};
use crate::schema::{
    ApprovePaymentOperation, BankPrincipalBinding, BankPrincipalIdBinding, BankSchema,
    ExternalPrincipalMapping, PaymentIdentity, PaymentIdentityField, PaymentIntent, Principal,
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

pub const PAYMENT_DECISION_OUTPUT_PAYMENT: &str = "payment";

impl ApplicationMutationOutputContract<BankSchema> for PaymentDecisionOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            BankSchema,
            PaymentIntent,
        >(
            PAYMENT_DECISION_OUTPUT_PAYMENT,
            ApplicationMutationOutputPosture::Preserve,
        )];
}

fn payment_scope_from_approval(input: &ApprovePayment) -> PaymentId {
    input.payment
}

fn payment_scope_from_rejection(input: &RejectPayment) -> PaymentId {
    input.payment
}

pub(crate) fn approval_input_identity(input: &ApprovePayment) -> [u8; 32] {
    *CanonicalProposalPayload::new("application-approve-payment")
        .text("payment", &input.payment.canonical_text())
        .u64("approver", input.approver.get())
        .derive_identity()
        .bytes()
}

fn rejection_input_identity(input: &RejectPayment) -> [u8; 32] {
    *CanonicalProposalPayload::new("application-reject-payment")
        .text("payment", &input.payment.canonical_text())
        .u64("rejecting-principal", input.rejecting_principal.get())
        .derive_identity()
        .bytes()
}

worth_query_mutation_binding!(
    pub ApprovePaymentMutationBinding for ApprovePayment, schema BankSchema,
    identity "bank.operation.approve-payment.mutation-binding.v1",
    input ApprovePaymentInputBinding,
    operation ApprovePaymentOperation,
    result PaymentDecisionResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
        key_identity client_key_identity,
        input_identity approval_input_identity,
    decision BankInvariantApprovedProposal,
    denial PaymentDecisionDenialBinding,
    handler identity "bank.operation.approve-payment.handler.v1",
    program required,
    workflow_authority required,
    outputs PaymentDecisionOutputs,
    principal BankPrincipalBinding,
        mapping ExternalPrincipalMapping,
        principal_entity Principal,
        principal_identity BankPrincipalId,
        identity_binding BankPrincipalIdBinding,
    scope PaymentIntent,
        PaymentIdentity,
        PaymentIdentityField,
        PaymentId,
        ReadOnly,
        NoApplicationUnit,
    field PaymentIdentityField::reference(),
    value payment_scope_from_approval,
    candidates creates 4, deletes 0, links 6, unlinks 0, writes 13, emits 3,
    resources retained_representation_bytes 32768, validator_work 17423
);

worth_query_mutation_binding!(
    pub RejectPaymentMutationBinding for RejectPayment, schema BankSchema,
    identity "bank.operation.reject-payment.mutation-binding.v1",
    input RejectPaymentInputBinding,
    operation RejectPaymentOperation,
    result PaymentDecisionResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
        key_identity client_key_identity,
        input_identity rejection_input_identity,
    decision BankInvariantApprovedProposal,
    denial PaymentDecisionDenialBinding,
    handler identity "bank.operation.reject-payment.handler.v1",
    program required,
    outputs PaymentDecisionOutputs,
    principal BankPrincipalBinding,
        mapping ExternalPrincipalMapping,
        principal_entity Principal,
        principal_identity BankPrincipalId,
        identity_binding BankPrincipalIdBinding,
    scope PaymentIntent,
        PaymentIdentity,
        PaymentIdentityField,
        PaymentId,
        ReadOnly,
        NoApplicationUnit,
    field PaymentIdentityField::reference(),
    value payment_scope_from_rejection,
    candidates creates 1, deletes 0, links 2, unlinks 0, writes 1, emits 0,
    resources retained_representation_bytes 4096, validator_work 132
);
