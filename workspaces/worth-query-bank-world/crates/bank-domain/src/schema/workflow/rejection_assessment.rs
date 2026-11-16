//! Rejection assessment preserves the actual rejected payment through its native producer.
use super::{
    ApprovedPaymentAssessmentBinding, ApprovedPaymentAssessmentInput,
    ApprovedPaymentAssessmentOutput, ApprovedPaymentAssessmentProvider,
};
use crate::{
    model::PaymentId,
    proposals::BankIdempotencyKey,
    queries::{PaymentDetailQueryBinding, PaymentDetailRequest},
    reads::PaymentSummary,
    schema::{BankSchema, PaymentStatus},
};
use worth_query_host::facade::application_contribution::*;

const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "rejected-payment-assessment",
    WorthQueryProducerLifecyclePosture::Initial,
);
const PRESERVE: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "rejected-payment-assessment",
    WorthQueryProducerLifecyclePosture::Preserve,
);
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[INITIAL, PRESERVE];
pub struct RejectedPaymentAssessmentOutputFamily;
impl WorthQueryProducerOutputFamily<BankSchema> for RejectedPaymentAssessmentOutputFamily {
    type Source = PaymentDetailQueryBinding;
    type Entity = crate::schema::PaymentIntent;
    const IDENTITY: &'static str = "worth.bank.rejected-payment-assessment.output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(row: &PaymentSummary) -> &'static str {
        if row.status() == PaymentStatus::Rejected {
            "rejected-payment-assessment"
        } else {
            "payment-not-rejected"
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RejectedPaymentAssessmentDemand {
    payment: PaymentId,
}
impl RejectedPaymentAssessmentDemand {
    pub const fn new(payment: PaymentId) -> Self {
        Self { payment }
    }
}
impl WorthQueryApplicationOutputDemand<BankSchema> for RejectedPaymentAssessmentDemand {
    type OutputFamily = RejectedPaymentAssessmentOutputFamily;
    fn source_intent(&self) -> PaymentDetailRequest {
        PaymentDetailRequest::new(self.payment)
    }
}
pub struct RejectedPaymentAssessmentProducer;
impl WorthQueryApplicationProducerBinding<BankSchema> for RejectedPaymentAssessmentProducer {
    type Operation = ApprovedPaymentAssessmentBinding;
    type OutputFamily = RejectedPaymentAssessmentOutputFamily;
    type Provider = ApprovedPaymentAssessmentProvider;
    type OutputRole = ApprovedPaymentAssessmentOutput;
    const IDENTITY: &'static str = "worth.bank.rejected-payment-assessment.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<BankSchema, RejectedPaymentAssessmentProducer>
    for ApprovedPaymentAssessmentProvider
{
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "worth.bank.rejected-payment-assessment.provider.v1";
    fn operation_input(&self, source: &PaymentSummary) -> ApprovedPaymentAssessmentInput {
        ApprovedPaymentAssessmentInput {
            payment: source.id(),
            status: source.status(),
        }
    }
    fn idempotency_key(
        &self,
        _: &PaymentSummary,
        source_identity: &[u8; 32],
    ) -> BankIdempotencyKey {
        let mut value = "rejected-payment-assessment:".to_owned();
        for byte in source_identity {
            use std::fmt::Write as _;
            write!(value, "{byte:02x}").unwrap();
        }
        BankIdempotencyKey::new(value).expect("assessment identity fits the Bank key contract")
    }
}
