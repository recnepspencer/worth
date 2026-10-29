pub use worth_foundational::facade::RetentionDeliveryProfile;
pub use worth_query::facade::{domain, runtime};

pub use super::super::installed_operation_fixture::{EvidenceGovernance, EvidenceScenario};
pub use super::assertions::assert_mandatory_core;
pub use super::execution::{
    admitted_receipt, admitted_receipt_with_governance, evidence, inspection,
};

pub fn assert_payload_is_digest_only(evidence: &domain::WorthQueryAdmittedDomainEvidence) {
    assert!(evidence.counter_sidecar().records().is_none());
    assert!(evidence.counter_sidecar().digest().is_some());
    assert!(evidence.decision_sidecar().records().is_none());
    assert!(evidence.decision_sidecar().digest().is_some());
    assert!(evidence.candidate_sidecar().records().is_none());
    assert!(evidence.candidate_sidecar().digest().is_some());
    assert!(evidence.transformation_sidecar().records().is_none());
    assert!(evidence.transformation_sidecar().digest().is_some());
}
