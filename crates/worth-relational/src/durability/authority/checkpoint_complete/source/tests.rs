//! Complete frame refusal mapping is pure value logic, with no runtime root.
use super::{canonical_readmission_stop, initial_inputs_stop, Denial};
use crate::content::RelationalContentDenial as Content;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal as Frame;
use crate::durability::source_initial_inputs::RelationalInitialInputsRefusal as Initial;
use crate::execution::RelationalExecutionDenialCause as Request;

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
