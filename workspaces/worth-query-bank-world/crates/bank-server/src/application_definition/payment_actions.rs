//! Exact installed plain actions deliver source completion, without query or workflow authority.
use super::composition::BankPaymentsFeature;
use bank_domain::schema::*;
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationRootComposition,
    ApplicationSourceCompletion,
};
/// Receipt-only completion for the declared InitiateBusinessPayment action.
pub struct PaymentInitiationDelivery;
impl ApplicationActionDelivery<BankSchema, InitiateBusinessPaymentMutationBinding>
    for PaymentInitiationDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.plain.paymentinitiation.completion.v1";
}
/// The exact installed InitiateBusinessPayment occurrence.
pub type PaymentInitiationAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankPaymentsFeature,
    InitiateBusinessPaymentMutationBinding,
    PaymentInitiationDelivery,
>;
/// Receipt-only completion for the declared RejectPayment action.
pub struct PaymentRejectionDelivery;
impl ApplicationActionDelivery<BankSchema, RejectPaymentMutationBinding>
    for PaymentRejectionDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.plain.paymentrejection.completion.v1";
}
/// The exact installed RejectPayment occurrence.
pub type PaymentRejectionAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankPaymentsFeature,
    RejectPaymentMutationBinding,
    PaymentRejectionDelivery,
>;
