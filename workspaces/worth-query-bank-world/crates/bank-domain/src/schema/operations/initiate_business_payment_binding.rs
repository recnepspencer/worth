use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputPosture,
        ApplicationMutationOutputRoleDescriptor,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{BankPrincipalId, BusinessId, PaymentId};
use crate::payments::BusinessPayment;
use crate::proposals::{
    BankIdempotencyClaim, BankIdempotencyKey, BankProposalDenial, CanonicalProposalPayload,
};

use super::create_personal_account_binding::client_key_identity;
use super::{InitiateBusinessPayment, InitiateBusinessPaymentInputBinding};
use crate::schema::{
    BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, Business, BusinessIdentity,
    BusinessIdentityField, ExternalPrincipalMapping, InitiateBusinessPaymentOperation,
    PaymentIntent, Principal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InitiateBusinessPaymentResult {
    pub payment: PaymentId,
}

worth_query_structured_value_binding!(
    pub InitiateBusinessPaymentResultBinding for InitiateBusinessPaymentResult {
        identity: "bank.operation.initiate-business-payment.result.v1"
    }
);

worth_query_structured_value_binding!(
    pub InitiateBusinessPaymentDenialBinding for BankProposalDenial {
        identity: "bank.operation.initiate-business-payment.denial.v1"
    }
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitiateBusinessPaymentDecision {
    payment: BusinessPayment,
}

impl InitiateBusinessPaymentDecision {
    pub fn from_payment(payment: BusinessPayment) -> Self {
        Self { payment }
    }

    pub fn into_payment(self) -> BusinessPayment {
        self.payment
    }
}

pub struct InitiateBusinessPaymentOutputs;

pub const INITIATE_BUSINESS_PAYMENT_OUTPUT_PAYMENT: &str = "payment";

impl ApplicationMutationOutputContract<BankSchema> for InitiateBusinessPaymentOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            BankSchema,
            PaymentIntent,
        >(
            INITIATE_BUSINESS_PAYMENT_OUTPUT_PAYMENT,
            ApplicationMutationOutputPosture::Create,
        )];
}

fn business_scope(input: &InitiateBusinessPayment) -> BusinessId {
    input.business
}

fn input_identity(input: &InitiateBusinessPayment) -> [u8; 32] {
    *CanonicalProposalPayload::new("application-initiate-business-payment")
        .u64("business", input.business.get())
        .text("source", &input.from.canonical_text())
        .u64("recipient", input.recipient.get())
        .i64("amount-minor-units", input.amount.minor_units())
        .derive_identity()
        .bytes()
}

pub fn initiate_business_payment_application_idempotency(
    binding: crate::proposals::BankOperationScopeBinding,
    key: &BankIdempotencyKey,
    input: &InitiateBusinessPayment,
) -> BankIdempotencyClaim {
    let payload = CanonicalProposalPayload::new("initiate-business-payment")
        .u64("business", input.business.get())
        .text("source", &input.from.canonical_text())
        .u64("recipient", input.recipient.get())
        .i64("amount-minor-units", input.amount.minor_units());
    BankIdempotencyClaim::derive(binding, key, payload)
}

worth_query_mutation_binding!(
    pub InitiateBusinessPaymentMutationBinding for InitiateBusinessPayment, schema BankSchema,
    identity "bank.operation.initiate-business-payment.mutation-binding.v1",
    input InitiateBusinessPaymentInputBinding,
    operation InitiateBusinessPaymentOperation,
    result InitiateBusinessPaymentResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
        key_identity client_key_identity,
        input_identity input_identity,
    decision InitiateBusinessPaymentDecision,
    denial InitiateBusinessPaymentDenialBinding,
    handler identity "bank.operation.initiate-business-payment.handler.v1",
    program required,
    outputs InitiateBusinessPaymentOutputs,
    principal BankPrincipalBinding,
        mapping ExternalPrincipalMapping,
        principal_entity Principal,
        principal_identity BankPrincipalId,
        identity_binding BankPrincipalIdBinding,
    scope Business,
        BusinessIdentity,
        BusinessIdentityField,
        BusinessId,
        ReadOnly,
        NoApplicationUnit,
    field BusinessIdentityField::reference(),
    value business_scope,
    candidates creates 1, deletes 0, links 4, unlinks 0, writes 3, emits 0,
    resources retained_representation_bytes 4096, validator_work 72
);
