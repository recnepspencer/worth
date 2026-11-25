//! Complete frame refusal mapping is pure value logic, with no runtime root.
use super::canonical_readmission_stop;
use super::initial_inputs_stop;
use crate::durability::source_initial_inputs::RelationalInitialInputsRefusal as Initial;
use contracts::content::denial::RelationalContentDenial as Content;
use contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Frame;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use contracts::execution::denial_cause::RelationalExecutionDenialCause as Request;
use worth_relational_contracts::internal_port as contracts;

#[test]
fn complete_original_frame_content_denials_preserve_their_semantic_and_request_causes() {
    assert!(matches!(
        canonical_readmission_stop(Frame::Content(Content::Execution(Request::Cancelled))),
        Denial::Request(Request::Cancelled)
    ));
    assert!(matches!(
        canonical_readmission_stop(Frame::Content(Content::IntegrityMismatch)),
        Denial::Content(Content::IntegrityMismatch)
    ));
    assert!(matches!(
        initial_inputs_stop(Initial::Payload(Frame::Content(Content::Execution(
            Request::Cancelled
        )))),
        Denial::Request(Request::Cancelled)
    ));
    assert!(matches!(
        initial_inputs_stop(Initial::Payload(Frame::Content(Content::IntegrityMismatch))),
        Denial::Content(Content::IntegrityMismatch)
    ));
}
