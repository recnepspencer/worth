//! A real producer's handler predicate survives captured output readmission.
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryApplicationRequestExt,
    WorthQueryOutputCurrentnessDenial,
};
pub use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
pub use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
pub use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;
pub use worth_relational::facade::identity::EntityId;

pub use super::document_retention_model::{
    assessment_output::{AssessmentOutput, RetentionAssessmentDemand, RetentionAssessmentOutputs},
    host::{publish, restore},
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_retention,
    programs::validated_second_program,
    retention_entry::DOCUMENT_IDENTITY,
    settled_verdict::{settle, RetentionVerdict},
};

#[path = "producer_predicate_checkpoint/ordinary_source.rs"]
pub mod ordinary_source;
#[path = "producer_predicate_checkpoint/program.rs"]
pub mod program;
#[path = "producer_predicate_checkpoint/transition.rs"]
pub mod transition;
pub use program::{validated_program, AssessmentRoot};

pub fn assert_captured_document_predicate(bytes: &[u8], accepted_bytes: usize, subject: EntityId) {
    // Inspect only Query's accepted-output section. The native payload remains opaque.
    // This independent current v13 wire expectation fails if capture or readmission drops
    // the predicate while retaining the document's ordinary field observations.
    assert_eq!(&bytes[40..42], &13_u16.to_be_bytes());
    assert_eq!(DOCUMENT_IDENTITY, "document-1");
    let value = br#"{"String":{"Raw":"document-1"}}"#;
    let mut expected = Vec::new();
    expected.extend_from_slice(&(value.len() as u32).to_be_bytes());
    expected.extend_from_slice(value);
    expected.extend_from_slice(&2_u64.to_be_bytes());
    expected.extend_from_slice(&1_u32.to_be_bytes());
    expected.extend_from_slice(&subject.partition_value().to_be_bytes());
    expected.extend_from_slice(&subject.local_slot_value().to_be_bytes());
    expected.extend_from_slice(&subject.generation_value().to_be_bytes());
    let accepted = &bytes[bytes.len() - accepted_bytes..];
    assert_eq!(
        accepted
            .windows(expected.len())
            .filter(|window| *window == expected)
            .count(),
        1,
        "the complete document equality predicate must survive capture and readmission"
    );
}
