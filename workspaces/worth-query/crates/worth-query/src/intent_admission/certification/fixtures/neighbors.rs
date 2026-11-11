use super::authoritative_declaration;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::facade::runtime::{
    WorthQueryIntentAdmissionCoveredEntrypoint, WorthQueryIntentAdmissionDecision,
    WorthQueryIntentViolationDecision,
};
use crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData;
use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;
use crate::intent_admission::{admit_runtime_intent_request, WorthQueryRawIntentAdmissionRequest};

#[derive(Clone)]
pub(in crate::intent_admission::certification) struct CertifiedDeferredIntentFixture {
    pub(in crate::intent_admission::certification) request: WorthQueryRawIntentAdmissionRequest,
    pub(in crate::intent_admission::certification) decision: WorthQueryIntentAdmissionDecision,
    pub(in crate::intent_admission::certification) trace: WorthQueryIntentDecisionTraceEnvelope,
}

#[derive(Clone)]
pub(in crate::intent_admission::certification) struct CertifiedUnsupportedIntentFixture {
    pub(in crate::intent_admission::certification) request: WorthQueryRawIntentAdmissionRequest,
    pub(in crate::intent_admission::certification) decision: WorthQueryIntentAdmissionDecision,
    pub(in crate::intent_admission::certification) trace: WorthQueryIntentDecisionTraceEnvelope,
}

pub(in crate::intent_admission::certification) fn certified_deferred_intent_fixture(
) -> Result<CertifiedDeferredIntentFixture, IdentityDerivationDenial> {
    let declaration = authoritative_declaration("certification-deferred-intent");
    let request = WorthQueryRawIntentAdmissionRequest::deferred_neighbor(
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteInspectionNeighborDeferred,
        declaration,
    )
    .expect("deferred certification request should build");
    let decision = admit_runtime_intent_request(request.clone())?;
    let trace = review_trace(&request, &decision, "deferred")?;
    Ok(CertifiedDeferredIntentFixture {
        request,
        decision,
        trace,
    })
}

pub(in crate::intent_admission::certification) fn certified_unsupported_intent_fixture(
) -> Result<CertifiedUnsupportedIntentFixture, IdentityDerivationDenial> {
    let declaration = authoritative_declaration("certification-unsupported-intent");
    let request = WorthQueryRawIntentAdmissionRequest::deferred_neighbor(
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteInspectionNeighborDeferred,
        declaration,
    )
    .expect("unsupported certification request should build");
    let eligibility = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        request.clone(),
    )?;
    let violation = WorthQueryIntentViolationDecision::from_eligibility_denial(
        &eligibility,
        "support-unsupported",
        "neighbor-unsupported-until-coverage",
    );
    let decision = WorthQueryIntentAdmissionDecision::Violation(violation);
    let trace = review_trace(&request, &decision, "unsupported")?;
    Ok(CertifiedUnsupportedIntentFixture {
        request,
        decision,
        trace,
    })
}

fn review_trace(
    request: &WorthQueryRawIntentAdmissionRequest,
    decision: &WorthQueryIntentAdmissionDecision,
    lane: &str,
) -> Result<WorthQueryIntentDecisionTraceEnvelope, IdentityDerivationDenial> {
    Ok(WorthQueryRuntimeIntentAdmissionReviewData::from_decision(
        request.clone(),
        decision.clone(),
    )?
    .decision_trace_envelope()
    .unwrap_or_else(|| panic!("{lane} certification review should preserve a trace"))
    .clone())
}
