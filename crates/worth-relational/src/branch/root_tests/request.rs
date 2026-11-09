//! Original roots, not reconstructed expected hashes, are the independent oracle.
use super::*;
use crate::branch::RelationalBranchRootIdentityIssuer;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn request_root_refusal_preserves_original_and_releases_partial_backing() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (_runtime, original, _) = committed_root_and_envelope(&execution);
    let axes = original.axes();
    let issuer = RelationalBranchRootIdentityIssuer::default();
    let calibration = SerialMemoryBudget::new(Some(1024 * 1024));
    let host =
        SerialRequest::from_memory(calibration.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&host);
    let mut controller = 0;
    execution
        .run_native_with_checkpoint(|_| controller = calibration.charged_memory_bytes())
        .unwrap();
    let budget = SerialMemoryBudget::new(Some(
        controller + size_of::<super::super::RelationalBranchRoot>() as u64 - 1,
    ));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&host);
    let attempt = || {
        super::super::RelationalBranchRoot::readmit_unchanged_for_request(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            &issuer,
            &original,
            original.canonical_payload().unwrap(),
            original.descriptor().unwrap().clone(),
            original.retained_schema_authority(),
            original.output_publication_positions.clone(),
            &execution,
        )
    };
    assert!(matches!(
        attempt(),
        Err(RelationalBranchRootCaptureDenial::Request(
            crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    cancellation.cancel();
    assert!(matches!(
        attempt(),
        Err(RelationalBranchRootCaptureDenial::Request(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(original.axes(), axes);
}

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
