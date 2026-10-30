pub use worth_query::facade::{certification, domain, runtime};

pub use super::super::installed_operation_fixture::EvidenceScenario;
pub use super::assertions::{
    assert_governance, assert_mandatory_core, assert_optional_counter_sidecar,
};
pub use super::execution::{admitted_receipt, evidence, inspection};

pub fn assert_admitted_evidence(
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

pub fn assert_non_owning_copy(
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

pub fn assert_digest_only_copy(
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

pub fn assert_certification_bundle(
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
