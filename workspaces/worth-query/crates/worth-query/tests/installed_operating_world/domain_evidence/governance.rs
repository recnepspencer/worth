use worth_foundational::facade::RetentionDeliveryProfile;
use worth_query::facade::{domain, runtime};

use super::super::installed_operation_fixture::{EvidenceGovernance, EvidenceScenario};
use super::assertions::assert_mandatory_core;
use super::execution::{admitted_receipt, admitted_receipt_with_governance, evidence, inspection};

#[test]
fn never_disclose_discards_malformed_optional_payload_before_sidecar_validation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for (name, scenario) in [
        (
            "domain-evidence-never-disclose-malformed-records",
            EvidenceScenario::MalformedSidecars,
        ),
        (
            "domain-evidence-never-disclose-malformed-counter",
            EvidenceScenario::MalformedOptionalCounter,
        ),
        (
            "domain-evidence-never-disclose-malformed-transformation",
            EvidenceScenario::MalformedTransformationSidecar,
        ),
    ] {
        let receipt = admitted_receipt(
            request_execution,
            name,
            scenario,
            domain::WorthQueryArtifactRedactionPosture::NeverDisclose,
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
    }
}

#[test]
fn governance_redaction_never_recovers_payload_and_preserves_policy() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for (name, redaction, expect_digest) in [
        (
            "domain-evidence-canonical-projection",
            domain::WorthQueryArtifactRedactionPosture::CanonicalProjectionOnly,
            true,
        ),
        (
            "domain-evidence-never-disclose",
            domain::WorthQueryArtifactRedactionPosture::NeverDisclose,
            false,
        ),
        (
            "domain-evidence-domain-redactor-required",
            domain::WorthQueryArtifactRedactionPosture::DomainRedactorRequired,
            false,
        ),
    ] {
        let receipt =
            admitted_receipt(request_execution, name, EvidenceScenario::Honest, redaction);
        let evidence = evidence(&receipt);
        assert_mandatory_core(evidence.core());
        assert_eq!(evidence.governance().redaction(), redaction);
        assert_eq!(evidence.counter_sidecar().digest().is_some(), expect_digest);
        assert_eq!(
            evidence.decision_sidecar().digest().is_some(),
            expect_digest
        );
        assert_eq!(
            evidence.candidate_sidecar().digest().is_some(),
            expect_digest
        );
        assert_eq!(
            evidence.transformation_sidecar().digest().is_some(),
            expect_digest
        );
        assert!(evidence.counter_sidecar().records().is_none());
        assert!(evidence.decision_sidecar().records().is_none());
        assert!(evidence.candidate_sidecar().records().is_none());
        assert!(evidence.transformation_sidecar().records().is_none());

        let inspection = inspection(
            evidence,
            runtime::CausalInspectionRedactionPolicy::PreserveDetail,
        );
        assert_eq!(inspection.governance(), evidence.governance());
        assert!(inspection.counter_sidecar().records().is_none());
        assert!(inspection.decision_sidecar().records().is_none());
        assert!(inspection.candidate_sidecar().records().is_none());
        assert!(inspection.transformation_sidecar().records().is_none());
        let bundle =
            worth_query::facade::certification::WorthQueryDomainEvidenceCertificationBundle::derive(
                &inspection,
            );
        assert_eq!(bundle.governance(), evidence.governance());
        assert_eq!(bundle.counter_sidecar().digest().is_some(), expect_digest);
        assert_eq!(bundle.decision_sidecar().digest().is_some(), expect_digest);
        assert_eq!(bundle.candidate_sidecar().digest().is_some(), expect_digest);
        assert_eq!(
            bundle.transformation_sidecar().digest().is_some(),
            expect_digest
        );
    }
}

#[test]
fn expiry_deletion_and_legal_hold_propagate_without_detached_payload() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for (name, governance) in [
        (
            "domain-evidence-expiring",
            EvidenceGovernance {
                redaction: domain::WorthQueryArtifactRedactionPosture::NotRequired,
                retention: RetentionDeliveryProfile::Ephemeral,
                decision_retention: RetentionDeliveryProfile::Ephemeral,
                secondary_decision_retention: None,
                deletion: domain::WorthQueryArtifactDeletionPosture::DeleteAfterRetention,
                legal_hold: domain::WorthQueryArtifactLegalHoldPosture::NotEligible,
            },
        ),
        (
            "domain-evidence-delete-with-run",
            EvidenceGovernance {
                redaction: domain::WorthQueryArtifactRedactionPosture::NotRequired,
                retention: RetentionDeliveryProfile::Retained,
                decision_retention: RetentionDeliveryProfile::Retained,
                secondary_decision_retention: None,
                deletion: domain::WorthQueryArtifactDeletionPosture::DeleteWithRun,
                legal_hold: domain::WorthQueryArtifactLegalHoldPosture::DomainControlled,
            },
        ),
        (
            "domain-evidence-held-delete-with-run",
            EvidenceGovernance {
                redaction: domain::WorthQueryArtifactRedactionPosture::NotRequired,
                retention: RetentionDeliveryProfile::Durable,
                decision_retention: RetentionDeliveryProfile::Durable,
                secondary_decision_retention: None,
                deletion: domain::WorthQueryArtifactDeletionPosture::DeleteWithRun,
                legal_hold: domain::WorthQueryArtifactLegalHoldPosture::RequiredWhenDirected,
            },
        ),
    ] {
        let receipt = admitted_receipt_with_governance(
            request_execution,
            name,
            EvidenceScenario::Honest,
            governance,
        );
        let evidence = evidence(&receipt);
        assert_eq!(evidence.governance().retention(), governance.retention);
        assert_eq!(evidence.governance().deletion(), governance.deletion);
        assert_eq!(evidence.governance().legal_hold(), governance.legal_hold);
        assert_payload_is_digest_only(evidence);

        let inspection = inspection(
            evidence,
            runtime::CausalInspectionRedactionPolicy::PreserveDetail,
        );
        assert_eq!(inspection.governance(), evidence.governance());
        assert!(inspection.counter_sidecar().records().is_none());
        assert!(inspection.decision_sidecar().records().is_none());
        assert!(inspection.candidate_sidecar().records().is_none());
        assert!(inspection.transformation_sidecar().records().is_none());

        let bundle =
            worth_query::facade::certification::WorthQueryDomainEvidenceCertificationBundle::derive(
                &inspection,
            );
        assert_eq!(bundle.governance(), evidence.governance());
        assert!(bundle.counter_sidecar().digest().is_some());
        assert!(bundle.decision_sidecar().digest().is_some());
        assert!(bundle.candidate_sidecar().digest().is_some());
        assert!(bundle.transformation_sidecar().digest().is_some());
    }
}

#[test]
fn shorter_lived_decision_records_never_enter_the_durable_artifact_sidecar() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let receipt = admitted_receipt_with_governance(
        request_execution,
        "domain-evidence-shorter-decision-retention",
        EvidenceScenario::Honest,
        EvidenceGovernance {
            redaction: domain::WorthQueryArtifactRedactionPosture::NotRequired,
            retention: RetentionDeliveryProfile::Durable,
            decision_retention: RetentionDeliveryProfile::Retained,
            secondary_decision_retention: None,
            deletion: domain::WorthQueryArtifactDeletionPosture::DomainControlled,
            legal_hold: domain::WorthQueryArtifactLegalHoldPosture::RequiredWhenDirected,
        },
    );
    let evidence = evidence(&receipt);

    assert!(matches!(
        evidence.decision_sidecar(),
        domain::WorthQueryAdmittedDomainEvidenceSidecar::DigestOnly { .. }
    ));
    assert!(evidence.counter_sidecar().records().is_some());
    assert!(evidence.candidate_sidecar().records().is_some());
    assert!(evidence.transformation_sidecar().records().is_some());

    let inspection = inspection(
        evidence,
        runtime::CausalInspectionRedactionPolicy::PreserveDetail,
    );
    assert!(inspection.decision_sidecar().records().is_none());
    let bundle =
        worth_query::facade::certification::WorthQueryDomainEvidenceCertificationBundle::derive(
            &inspection,
        );
    assert!(bundle.decision_sidecar().digest().is_some());
}

#[test]
fn mixed_decision_governance_materializes_only_schema_eligible_records() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let receipt = admitted_receipt_with_governance(
        request_execution,
        "domain-evidence-mixed-decision-retention",
        EvidenceScenario::MixedDecisionGovernance,
        EvidenceGovernance {
            redaction: domain::WorthQueryArtifactRedactionPosture::NotRequired,
            retention: RetentionDeliveryProfile::Durable,
            decision_retention: RetentionDeliveryProfile::Durable,
            secondary_decision_retention: Some(RetentionDeliveryProfile::Retained),
            deletion: domain::WorthQueryArtifactDeletionPosture::DomainControlled,
            legal_hold: domain::WorthQueryArtifactLegalHoldPosture::RequiredWhenDirected,
        },
    );
    let evidence = evidence(&receipt);
    let sidecar = evidence.decision_sidecar();

    assert!(matches!(
        sidecar,
        domain::WorthQueryAdmittedDomainEvidenceSidecar::PartiallyMaterialized { .. }
    ));
    let records = sidecar.records().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].kind().as_str(), "candidate-rejected");
    assert!(sidecar.digest().is_some());

    let inspection = inspection(
        evidence,
        runtime::CausalInspectionRedactionPolicy::PreserveDetail,
    );
    assert!(inspection.decision_sidecar().records().is_none());
    assert_eq!(
        inspection.decision_sidecar().digest(),
        evidence.decision_sidecar().digest()
    );
}

fn assert_payload_is_digest_only(evidence: &domain::WorthQueryAdmittedDomainEvidence) {
    assert!(evidence.counter_sidecar().records().is_none());
    assert!(evidence.counter_sidecar().digest().is_some());
    assert!(evidence.decision_sidecar().records().is_none());
    assert!(evidence.decision_sidecar().digest().is_some());
    assert!(evidence.candidate_sidecar().records().is_none());
    assert!(evidence.candidate_sidecar().digest().is_some());
    assert!(evidence.transformation_sidecar().records().is_none());
    assert!(evidence.transformation_sidecar().digest().is_some());
}
