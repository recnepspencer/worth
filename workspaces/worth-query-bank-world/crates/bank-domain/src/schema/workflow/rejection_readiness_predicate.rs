//! The authored condition follows the Bank payment decision, never a timer.
use super::RejectedPaymentAssessmentReadyNode;
use crate::schema::{PaymentStatus, PaymentStatusBinding};
use worth_query_host::facade::{
    declaration::application_schema::ApplicationReadableScalarValueBinding, domain,
};

pub struct RejectedPaymentAssessmentPredicate(PaymentStatus);
impl RejectedPaymentAssessmentPredicate {
    pub(super) fn awaiting_rejection() -> Self {
        Self(PaymentStatus::Pending)
    }
    pub fn rejected() -> Self {
        Self(PaymentStatus::Rejected)
    }
}
impl domain::WorthQueryHostConditionalPredicateProvider<RejectedPaymentAssessmentReadyNode>
    for RejectedPaymentAssessmentPredicate
{
    const SEMANTIC_IDENTITY: &'static str = "worth.bank.rejected-payment-assessment.predicate.v1";
    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        Ok(domain::WorthQueryHostProviderHeapRetention::none())
    }
    fn evaluate(
        &self,
        observation: domain::WorthQueryConditionalObservationView<'_>,
    ) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure>
    {
        let dependency = observation
            .dependency(0)
            .ok_or_else(|| unsupported("payment status dependency is absent"))?;
        let status = match dependency.current() {
            domain::WorthQueryConditionalObservedValue::Absent => None,
            domain::WorthQueryConditionalObservedValue::Present(artifact) => {
                let raw = artifact
                    .field(&domain::FieldKey::new("PaymentStatusField").unwrap())
                    .ok_or_else(|| unsupported("payment status field is absent"))?;
                Some(
                    PaymentStatusBinding::decode(raw)
                        .map_err(|cause| unsupported(format!("payment status codec: {cause:?}")))?,
                )
            }
        };
        Ok(if status == Some(self.0) {
            domain::WorthQueryHostPredicateDecision::Satisfied
        } else {
            domain::WorthQueryHostPredicateDecision::Unsatisfied
        })
    }
}
impl domain::WorthQueryAuthoredHostConditionalPredicateProvider<RejectedPaymentAssessmentReadyNode>
    for RejectedPaymentAssessmentPredicate
{
    fn authored_provider_description(&self) -> domain::BridgeAuthoredProviderDescription {
        domain::BridgeAuthoredProviderDescription::new(
            domain::BoundaryProtocolIdentity::new(
                "worth.bank.rejected-payment-assessment.predicate",
            ),
            domain::BoundaryProtocolVersion::new(1),
            self.0.canonical_text().as_bytes().to_vec(),
        )
    }
    fn readmit_authored_provider(
        &self,
        source: &domain::BridgeAuthoredProviderDescription,
    ) -> Result<Self, domain::AuthoredProviderReconstructionDenial> {
        self.authored_provider_description()
            .admit_source_version(source)?;
        match source.payload() {
            b"Pending" => Ok(Self::awaiting_rejection()),
            b"Rejected" => Ok(Self::rejected()),
            _ => Err(domain::AuthoredProviderReconstructionDenial::MalformedPayload),
        }
    }
}
fn unsupported(detail: impl Into<String>) -> domain::WorthQueryHostPredicateFailure {
    domain::WorthQueryHostPredicateFailure::new(
        domain::WorthQueryHostPredicateFailureKind::ObservationUnsupported,
        detail,
    )
}
