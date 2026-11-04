//! A selected source cannot substitute original replay for a missing image.
use super::*;

#[test]
fn selected_checkpoint_without_admission_image_refuses_before_canonical_body_reads() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let (source, medium) = initialized(request);
    let committed = create_entity_outcome(request, &source, "missing-selected-image");
    release_test_commit_snapshot(&source, &committed);
    let capture = capture_external(&source, &medium, request);
    medium.file_checkpoint(capture.checkpoint().clone());
    let reads = medium.canonical_member_reads();
    assert!(matches!(
        home_input(&medium, capture.source_artifact().clone(), request),
        Err(RelationalInitialInputsRefusal::Native(
            RelationalRecoveryStop::AdmissionImage(Refusal::MissingArtifact,)
        ))
    ));
    assert_eq!(medium.canonical_member_reads(), reads);
    assert!(home_image_input(&medium, &capture, request).is_ok());
}
