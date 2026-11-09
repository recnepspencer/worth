//! Original roots, not reconstructed expected hashes, are the independent oracle.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn streamed_visibility_preserves_original_buffered_preimage_and_request_stop() {
    use sha2::{Digest, Sha256};
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (_runtime, original, envelope) = committed_root_and_envelope(&execution);
    let axes = original.axes().unwrap();
    let mut reference = Sha256::new();
    reference.update(b"worth.relational.branch-visibility.v1\0");
    reference.update(axes.storage_root);
    reference.update(axes.schema_root);
    reference.update([0]);
    reference.update(envelope.encode_authoritative_payload().unwrap());
    let expected: [u8; 32] = reference.finalize().into();
    assert_eq!(expected, axes.visibility.digest());
    assert_eq!(
        RelationalBranchVisibilityCommitment::for_root_for_request(
            &envelope,
            axes.storage_root,
            axes.schema_root,
            axes.correctness_index,
            &execution,
        )
        .unwrap()
        .digest(),
        expected
    );
    let cancellation = CancellationSource::new();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    cancellation.cancel();
    assert!(matches!(
        RelationalBranchVisibilityCommitment::for_root_for_request(
            &envelope,
            axes.storage_root,
            axes.schema_root,
            axes.correctness_index,
            &ExecutionRequest::serial(&host),
        ),
        Err(RelationalBranchRootCaptureDenial::Request(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        ))
    ));
}
