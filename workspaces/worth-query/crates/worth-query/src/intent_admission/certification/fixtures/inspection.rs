use super::authoritative_declaration;
use super::bridge::certification_bridge;
use super::runtime::certification_runtime;
use crate::facade::runtime::{
    CausalEvidenceFamily, CausalInspection, CausalInspectionMaterializationPolicy,
    CausalInspectionRedactionPolicy, QueryObservationReceipt, WorthQueryInspection,
};
use crate::runtime::CausalInspectionBoundaryAudit;

#[derive(Clone)]
pub(in crate::intent_admission::certification) struct CertifiedInspectionAdvisoryRedactionFixture {
    pub(in crate::intent_admission::certification) request_digest: String,
    pub(in crate::intent_admission::certification) eligibility_digest: String,
    pub(in crate::intent_admission::certification) decision_trace_digest: String,
    pub(in crate::intent_admission::certification) execution_provenance_chain_digest: String,
    pub(in crate::intent_admission::certification) full_artifact_digest: String,
    pub(in crate::intent_admission::certification) redacted_artifact_digest: String,
    pub(in crate::intent_admission::certification) causal_identity_digest: String,
    pub(in crate::intent_admission::certification) boundary_audit_digest: String,
}

pub(in crate::intent_admission::certification) fn certified_inspection_advisory_redaction_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CertifiedInspectionAdvisoryRedactionFixture {
    let inspection_basis = crate::basis_lifecycle::basis_lifecycle()
        .current_head()
        .inspect()
        .expect("inspection certification basis should admit");
    let bridge = certification_bridge();
    let declaration = authoritative_declaration("certification-inspection-advisory-redaction");
    let mut runtime = certification_runtime(execution);
    let receipt = runtime
        .intent(declaration)
        .execute(execution)
        .expect("inspection certification intent should execute through lattice");
    let review = runtime
        .inspect_intent(&receipt)
        .review()
        .expect("inspection certification review should succeed");
    let request_digest = review.request().request_digest().to_string();
    let eligibility_digest = review.eligibility().eligibility_digest().to_string();
    let inspection_result = review
        .admit()
        .expect("inspection certification should admit through unified inspection intent")
        .execute(execution)
        .expect("inspection certification should execute through unified inspection intent");
    let observation = match inspection_result.inspection() {
        WorthQueryInspection::IntentReceipt(inspection) => {
            QueryObservationReceipt::from_intent_receipt_inspection(
                inspection,
                inspection_basis.clone(),
            )
        }
        other => panic!("expected intent receipt inspection, got {other:?}"),
    };

    let full = CausalInspection::for_observation(observation.clone(), inspection_basis.clone())
        .why_changed()
        .materialized_detail()
        .evidence_families([
            CausalEvidenceFamily::QueryInspection,
            CausalEvidenceFamily::RelationalAuthority,
        ])
        .redaction(CausalInspectionRedactionPolicy::PreserveDetail)
        .materialization(CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact)
        .plan()
        .expect("full inspection plan should build")
        .materialize_with_bridge(&bridge)
        .expect("full inspection artifact should materialize");

    let redacted = CausalInspection::for_observation(observation, inspection_basis)
        .why_changed()
        .materialized_detail()
        .evidence_families([
            CausalEvidenceFamily::QueryInspection,
            CausalEvidenceFamily::RelationalAuthority,
        ])
        .redaction(CausalInspectionRedactionPolicy::DigestOnly)
        .materialization(CausalInspectionMaterializationPolicy::DigestReferenceOnly)
        .plan()
        .expect("redacted inspection plan should build")
        .materialize_with_bridge(&bridge)
        .expect("redacted inspection artifact should materialize");

    let boundary_audit =
        CausalInspectionBoundaryAudit::from_query_artifact_public_surface(&redacted);
    let execution_provenance_chain_digest = inspection_result
        .receipt()
        .execution_provenance()
        .expect("lattice inspection receipt should retain provenance")
        .execution_provenance_chain_digest()
        .to_string();
    let decision_trace_digest = inspection_result
        .receipt()
        .decision_trace_envelope()
        .expect("lattice inspection receipt should retain decision trace")
        .trace_digest()
        .to_string();
    CertifiedInspectionAdvisoryRedactionFixture {
        request_digest,
        eligibility_digest,
        decision_trace_digest,
        execution_provenance_chain_digest,
        full_artifact_digest: full.artifact_for_reporting().to_string(),
        redacted_artifact_digest: redacted.artifact_for_reporting().to_string(),
        causal_identity_digest: redacted.causal_identity_for_reporting().to_string(),
        boundary_audit_digest: boundary_audit.audit_digest().to_string(),
    }
}
