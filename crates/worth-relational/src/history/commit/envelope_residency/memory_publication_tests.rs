//! Real Memory publication owns one original frame; tuple observations stay scalar.
use super::*;
use crate::tests::support::*;
use sha2::{Digest, Sha256};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn memory_original_frame_survives_export_checkpoint_and_runtime_drop() {
    let budget = SerialMemoryBudget::new(Some(8 << 20));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let result = create_entity_outcome(&execution, &runtime, "original-memory-frame");
    let cell = runtime
        .history
        .commit_artifact(result.commit.commit_id)
        .unwrap()
        .envelope_residency();
    let original = {
        let state = cell.state.lock().unwrap();
        let EnvelopeBacking::Memory {
            original: Some(bytes),
            ..
        } = &state.backing
        else {
            panic!("real Memory publication owns original bytes")
        };
        bytes.clone()
    };
    let positioned = cell.demand_positioned(&execution, 1 << 30).unwrap();
    // Independent existing encoder is a fixture reference, outside the export claim.
    let reference = RelationalCanonicalCommitArtifact::from_positioned(
        &positioned,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    assert_eq!(reference.as_bytes(), original.bytes());
    let expected = original.bytes().to_vec();
    let exported = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(exported.as_bytes().as_ptr(), original.bytes().as_ptr());
    let native = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    assert!(native
        .bytes()
        .starts_with(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC));
    let again = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(again.as_bytes().as_ptr(), original.bytes().as_ptr());
    release_test_commit_snapshot(&runtime, &result);
    drop(native);
    drop(positioned);
    drop(reference);
    drop(result);
    drop(cell);
    drop(runtime);
    drop(original);
    drop(again);
    assert_eq!(exported.as_bytes(), expected);
    let clone = exported.clone();
    drop(exported);
    assert_eq!(clone.as_bytes(), expected);
    drop(clone);
}
#[test]
fn cancelled_memory_publication_keeps_position_and_head_then_allows_ordinary_publication() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("cancel-memory-frame"),
            setup_request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(&setup_request, &runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(&setup_request, proposal)
        .unwrap();
    let counters = runtime.patch_position_reservation_counters();
    let before_head = runtime.history.latest_recorded_patch_position();
    let cancel = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
    let original = ExecutionRequest::serial(&host);
    cancel.cancel();
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(&original, candidate),
        crate::mvcc::RelationalPublicationOutcome::RequestStopped(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        )
    ));
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments
    );
    assert_eq!(
        runtime.history.latest_recorded_patch_position(),
        before_head
    );
    runtime.compare_current_exact(&basis).unwrap();
    drop(basis);
    let result = create_entity_outcome(&setup_request, &runtime, "caller-after-cancellation");
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments + 1
    );
    release_test_commit_snapshot(&runtime, &result);
}
#[test]
fn memory_frame_preserves_original_payload_and_scalar_tuple_observations() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let result = create_entity_outcome(&execution, &runtime, "tuple-observations");
    let artifact = runtime
        .history
        .commit_artifact(result.commit.commit_id)
        .unwrap();
    let envelope = artifact.resident_envelope().unwrap();
    let bytes = envelope.encode_authoritative_payload().unwrap();
    assert_eq!(artifact.canonical_payload_bytes(), bytes.len() as u64);
    assert_eq!(
        artifact.canonical_payload_digest().unwrap(),
        <[u8; 32]>::from(Sha256::digest(&bytes))
    );
    assert!(artifact.envelope_residency().payload_storage_bytes() > bytes.len() as u64);
    let cell = artifact.envelope_residency();
    let original = cell.export_artifact(&execution).unwrap();
    assert_ne!(
        original.payload_digest().as_bytes(),
        &artifact.canonical_payload_digest().unwrap()
    );
    drop(envelope);
    release_test_commit_snapshot(&runtime, &result);
}
