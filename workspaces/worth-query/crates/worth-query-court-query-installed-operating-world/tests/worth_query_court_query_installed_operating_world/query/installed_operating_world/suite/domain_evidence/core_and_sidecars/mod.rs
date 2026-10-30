pub(crate) use worth_query_testkit_query::facade::query::suite::domain_evidence::core_and_sidecars::*;

#[test]
fn honest_evidence_keeps_payload_in_its_owner_and_narrows_all_derived_copies() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
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

#[test]
fn provider_omission_preserves_mandatory_search_and_loss_meaning() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
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
