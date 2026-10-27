use crate::facade::publication::PatchStreamReadErrorClass;
use crate::tests::support::*;

#[test]
fn patch_stream_resume_batches_commits_without_duplication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(execution, &runtime, "a");
    let _second = create_entity_outcome(execution, &runtime, "b");
    let _third = create_entity_outcome(execution, &runtime, "c");

    let first_batch = runtime
        .publication()
        .read_patch_stream(
            execution,
            PatchStreamRequest {
                after_position: None,
                max_commits: 2,
            },
        )
        .unwrap();
    let resumed = runtime
        .publication()
        .read_patch_stream(
            execution,
            PatchStreamRequest {
                after_position: first_batch.next_position,
                max_commits: 2,
            },
        )
        .unwrap();

    assert_eq!(first_batch.patches.len(), 2);
    assert_eq!(first_batch.next_position, Some(PatchStreamPosition(2)));
    assert_eq!(first_batch.latest_position, Some(PatchStreamPosition(3)));
    assert_eq!(resumed.patches.len(), 1);
    assert_eq!(resumed.resumed_after, Some(PatchStreamPosition(2)));
    assert_eq!(resumed.patches[0].position, PatchStreamPosition(3));
}

#[test]
fn patch_stream_rejects_unknown_resume_position() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(execution, &runtime, "anchor");

    let error = runtime
        .publication()
        .read_patch_stream(
            execution,
            PatchStreamRequest {
                after_position: Some(PatchStreamPosition(99)),
                max_commits: 1,
            },
        )
        .unwrap_err();

    assert_eq!(
        error.class,
        PatchStreamReadErrorClass::UnknownResumePosition
    );
}

#[test]
fn patch_stream_uses_durable_canonical_history_when_retained_envelope_is_missing() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "a");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let _second = create_entity_outcome(execution, &runtime, "b");
    let _third = create_entity_outcome(execution, &runtime, "c");

    assert!(runtime
        .history_authority()
        .evict_commit_envelope_preserving_patch_position_for_durable_recovery_test(
            first.commit.commit_id,
        ));

    let batch = runtime
        .publication()
        .read_patch_stream(
            execution,
            PatchStreamRequest {
                after_position: None,
                max_commits: 8,
            },
        )
        .unwrap();

    assert_eq!(batch.patches.len(), 3);
    assert_eq!(batch.patches[0].position, PatchStreamPosition(1));
    assert_eq!(batch.next_position, Some(PatchStreamPosition(3)));
}
