use worth_query::facade::{certification, domain, runtime};

use super::super::installed_operation_fixture::EvidenceScenario;
use super::assertions::{
    assert_governance, assert_mandatory_core, assert_optional_counter_sidecar,
};
use super::execution::{admitted_receipt, evidence, inspection};

#[test]
fn honest_evidence_keeps_payload_in_its_owner_and_narrows_all_derived_copies() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let receipt = admitted_receipt(
        request_execution,
        "domain-evidence-honest",
        EvidenceScenario::Honest,
        domain::WorthQueryArtifactRedactionPosture::NotRequired,
    );
    let evidence = evidence(&receipt);
    assert_admitted_evidence(&receipt, evidence);

    let preserved = inspection(
        evidence,
        runtime::CausalInspectionRedactionPolicy::PreserveDetail,
    );
    assert_non_owning_copy(&preserved, evidence);

    let digested = inspection(
        evidence,
        runtime::CausalInspectionRedactionPolicy::DigestOnly,
    );
    assert_digest_only_copy(&digested, evidence);

    let bundle = certification::WorthQueryDomainEvidenceCertificationBundle::derive(&preserved);
    assert_certification_bundle(&bundle, evidence);
}

fn assert_admitted_evidence(
    receipt: &domain::WorthQueryBoundExecutionReceipt,
    evidence: &domain::WorthQueryAdmittedDomainEvidence,
) {
    assert_eq!(
        evidence.binding().binding_identity(),
        receipt.binding_identity()
    );
    assert_eq!(
        evidence.binding().output_occurrence_identity(),
        receipt.output_identity()
    );
    assert!(evidence.binding().run_identity().is_none());
    assert!(evidence.binding().stage_identity().is_none());
    assert!(!evidence.binding().operation_identity().is_empty());
    assert!(!evidence.binding().basis_identity().is_empty());
    assert!(!evidence.binding().execution_snapshot_identity().is_empty());
    assert_governance(evidence.governance());
    assert_mandatory_core(evidence.core());
    assert_optional_counter_sidecar(evidence.counter_sidecar().records().unwrap());
    assert_eq!(
        evidence.authority_posture(),
        domain::WorthQueryDomainEvidenceAuthorityPosture::DescriptiveOnly
    );
    assert_eq!(evidence.decision_sidecar().records().unwrap().len(), 1);
    assert_eq!(evidence.candidate_sidecar().records().unwrap().len(), 2);
    let transformations = evidence.transformation_sidecar().records().unwrap();
    assert_eq!(transformations.len(), 1);
    assert_eq!(transformations[0].source_occurrence_identity(), "source-1");
    assert_eq!(
        transformations[0].output_occurrence_identities(),
        &[
            receipt.output_identity().to_owned(),
            "output-secondary".into()
        ]
    );
}

fn assert_non_owning_copy(
    preserved: &runtime::WorthQueryDomainEvidenceInspectionCopy,
    evidence: &domain::WorthQueryAdmittedDomainEvidence,
) {
    assert_eq!(preserved.core(), evidence.core());
    assert_eq!(preserved.governance(), evidence.governance());
    assert!(preserved.counter_sidecar().records().is_none());
    assert!(preserved.decision_sidecar().records().is_none());
    assert!(preserved.candidate_sidecar().records().is_none());
    assert!(preserved.transformation_sidecar().records().is_none());
    assert_eq!(
        preserved.counter_sidecar().digest(),
        evidence.counter_sidecar().digest()
    );
    assert_eq!(
        preserved.decision_sidecar().digest(),
        evidence.decision_sidecar().digest()
    );
    assert_eq!(
        preserved.candidate_sidecar().digest(),
        evidence.candidate_sidecar().digest()
    );
    assert_eq!(
        preserved.transformation_sidecar().digest(),
        evidence.transformation_sidecar().digest()
    );
    assert_eq!(
        preserved.authority_posture(),
        domain::WorthQueryDomainEvidenceAuthorityPosture::DescriptiveOnly
    );
}

fn assert_digest_only_copy(
    digested: &runtime::WorthQueryDomainEvidenceInspectionCopy,
    evidence: &domain::WorthQueryAdmittedDomainEvidence,
) {
    assert_eq!(digested.core(), evidence.core());
    assert_eq!(digested.governance(), evidence.governance());
    assert!(digested.counter_sidecar().records().is_none());
    assert!(digested.decision_sidecar().records().is_none());
    assert!(digested.candidate_sidecar().records().is_none());
    assert!(digested.transformation_sidecar().records().is_none());
    assert_eq!(
        digested.counter_sidecar().digest(),
        evidence.counter_sidecar().digest()
    );
    assert_eq!(
        digested.decision_sidecar().digest(),
        evidence.decision_sidecar().digest()
    );
    assert_eq!(
        digested.candidate_sidecar().digest(),
        evidence.candidate_sidecar().digest()
    );
    assert_eq!(
        digested.transformation_sidecar().digest(),
        evidence.transformation_sidecar().digest()
    );
}

fn assert_certification_bundle(
    bundle: &certification::WorthQueryDomainEvidenceCertificationBundle,
    evidence: &domain::WorthQueryAdmittedDomainEvidence,
) {
    assert_eq!(bundle.core(), evidence.core());
    assert_eq!(bundle.governance(), evidence.governance());
    assert_eq!(
        bundle.counter_sidecar().digest(),
        evidence.counter_sidecar().digest()
    );
    assert_eq!(
        bundle.decision_sidecar().digest(),
        evidence.decision_sidecar().digest()
    );
    assert_eq!(
        bundle.candidate_sidecar().digest(),
        evidence.candidate_sidecar().digest()
    );
    assert_eq!(
        bundle.transformation_sidecar().digest(),
        evidence.transformation_sidecar().digest()
    );
    assert_eq!(
        bundle.authority_posture(),
        domain::WorthQueryDomainEvidenceAuthorityPosture::DescriptiveOnly
    );
}

#[test]
fn provider_omission_preserves_mandatory_search_and_loss_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let receipt = admitted_receipt(
        request_execution,
        "domain-evidence-provider-omission",
        EvidenceScenario::OmitSidecars,
        domain::WorthQueryArtifactRedactionPosture::NotRequired,
    );
    let evidence = evidence(&receipt);

    assert_mandatory_core(evidence.core());
    assert!(matches!(
        evidence.counter_sidecar(),
        domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
    ));
    assert!(matches!(
        evidence.decision_sidecar(),
        domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
    ));
    assert!(matches!(
        evidence.candidate_sidecar(),
        domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
    ));
    assert!(matches!(
        evidence.transformation_sidecar(),
        domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
    ));

    let inspection = inspection(
        evidence,
        runtime::CausalInspectionRedactionPolicy::PreserveDetail,
    );
    assert_mandatory_core(inspection.core());
    assert!(inspection.counter_sidecar().is_omitted());
    assert!(inspection.decision_sidecar().is_omitted());
    assert!(inspection.candidate_sidecar().is_omitted());
    assert!(inspection.transformation_sidecar().is_omitted());
    let bundle = certification::WorthQueryDomainEvidenceCertificationBundle::derive(&inspection);
    assert_mandatory_core(bundle.core());
    assert!(bundle.counter_sidecar().is_omitted());
    assert!(bundle.decision_sidecar().is_omitted());
    assert!(bundle.candidate_sidecar().is_omitted());
    assert!(bundle.transformation_sidecar().is_omitted());
}
