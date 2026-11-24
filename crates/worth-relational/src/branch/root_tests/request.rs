//! Original roots, not reconstructed expected hashes, are the independent oracle.
use super::*;
use crate::branch::root::identity::RelationalBranchRootIdentityIssuer;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn request_root_refusal_preserves_original_and_releases_partial_backing() {
    let host = crate::execution::test_policy::serial_request();
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
            crate::execution::denial_cause::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    cancellation.cancel();
    assert!(matches!(
        attempt(),
        Err(RelationalBranchRootCaptureDenial::Request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(original.axes(), axes);
}

#[test]
fn streamed_visibility_preserves_original_buffered_preimage_and_request_stop() {
    use sha2::{Digest, Sha256};
    let host = crate::execution::test_policy::serial_request();
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
            crate::execution::denial_cause::RelationalExecutionDenialCause::Cancelled
        ))
    ));
}

#[test]
fn interim_predecessor_memory_exhaustion_preserves_typed_request_refusal() {
    use crate::branch::recorded_basis::refusal::RelationalRecordedComponentRefusal as Refusal;
    use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;

    let host = crate::execution::test_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let source = runtime_with_test_schema();
    let _ = create_entity_outcome(&execution, &source, "interim-seed");

    let (native, mut capture) = source
        .durability_authority()
        .capture_interim_checkpoint(
            &execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let image = capture
        .emit_interim_image(
            1,
            worth_foundational::CanonicalDigestId::new([7; 32]),
            &execution,
        )
        .unwrap();
    let native_len = native.bytes().len();
    let total = native_len + image.as_bytes().len();
    let mut bytes =
        worth_execution::ExecutionRequestArrayBuilder::allocate(execution, total).unwrap();
    bytes.extend_from_slice(native.bytes()).unwrap();
    bytes.extend_from_slice(image.as_bytes()).unwrap();
    drop(native);
    let origin = capture
        .seal_transport(
            bytes.seal_immutable_bytes().unwrap(),
            0..native_len,
            native_len..total,
            &execution,
        )
        .unwrap();

    let mut receiver = runtime_with_test_schema();
    let _ = receiver
        .durability_recovery()
        .restore_interim_checkpoint_with_authority(&execution, &origin)
        .unwrap();
    let (_, interim_basis) = receiver
        .observe_branch(&receiver.main_branch_identity())
        .unwrap();
    assert!(
        interim_basis.inner.root.canonical_envelope().is_none(),
        "interim restored head carries no resident canonical envelope"
    );

    let measure_budget = SerialMemoryBudget::new(None);
    let measure_host = SerialRequest::from_memory(
        measure_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let observed = receiver
        .owner_component_services()
        .basis_port()
        .observe_memory_component(&ExecutionRequest::serial(&measure_host), &interim_basis)
        .unwrap();
    let retained_bytes = measure_budget.charged_memory_bytes();
    assert!(retained_bytes > 0);
    drop(observed);
    assert_eq!(measure_budget.charged_memory_bytes(), 0);

    // One byte short of the final interim component reservation must surface as
    // a typed PolicyMemoryExhausted request refusal, never a generic Source error.
    let tight_budget = SerialMemoryBudget::new(Some(retained_bytes - 1));
    let tight_host = SerialRequest::from_memory(
        tight_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let err = receiver
        .owner_component_services()
        .basis_port()
        .observe_memory_component(&ExecutionRequest::serial(&tight_host), &interim_basis)
        .unwrap_err();
    assert!(
        matches!(err, Refusal::Request(Cause::PolicyMemoryExhausted { .. })),
        "expected typed PolicyMemoryExhausted, got: {err:?}"
    );
    assert_eq!(tight_budget.charged_memory_bytes(), 0);
}
