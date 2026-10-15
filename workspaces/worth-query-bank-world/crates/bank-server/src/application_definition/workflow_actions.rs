//! The guarded payment effect declares no output contract; workflow authority remains native.
use super::composition::BankPaymentsFeature;
use bank_domain::schema::{ApprovePaymentMutationBinding, BankSchema};
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationRootComposition,
    ApplicationSourceCompletion,
};
pub struct ApprovedPaymentEffectDelivery;
impl ApplicationActionDelivery<BankSchema, ApprovePaymentMutationBinding>
    for ApprovedPaymentEffectDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.workflow.approved-payment.effect-completion.v1";
}
pub type ApprovedPaymentEffectAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankPaymentsFeature,
    ApprovePaymentMutationBinding,
    ApprovedPaymentEffectDelivery,
>;
