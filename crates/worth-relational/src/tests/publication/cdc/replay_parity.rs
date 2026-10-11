use crate::tests::support::*;

#[test]
fn subscriber_stream_matches_patch_stream_for_committed_history() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(execution, &runtime, "a");
    let _ = create_entity_outcome(execution, &runtime, "b");

    let patch_batch = runtime
        .publication()
        .read_patch_stream(PatchStreamRequest {
            after_position: None,
            max_commits: 8,
        })
        .unwrap();
    let subscriber_batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(8))
        .unwrap();

    assert_eq!(subscriber_batch.patches, patch_batch.patches);
}

#[test]
fn durable_source_subscriber_stream_matches_recovered_runtime_patch_stream() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "a");
    runtime.durability_authority().checkpoint().unwrap();
    let _second = create_entity_outcome(execution, &runtime, "b");
    let _third = create_entity_outcome(execution, &runtime, "c");

    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id));

    let subscriber_batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            checkpoint_for_schema_version(PatchStreamPosition(1), SchemaVersionId(1)),
            8,
        ))
        .unwrap();

    let recovery_plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, recovery_plan)
        .unwrap();
    let recovered_patch_batch = recovered
        .publication()
        .read_patch_stream(PatchStreamRequest {
            after_position: Some(PatchStreamPosition(1)),
            max_commits: 8,
        })
        .unwrap();

    assert_eq!(subscriber_batch.patches, recovered_patch_batch.patches);
}

#[test]
fn subscriber_stream_keeps_patch_parity_when_later_retained_envelope_is_missing() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let _first = create_entity_outcome(execution, &runtime, "a");
    runtime.durability_authority().checkpoint().unwrap();
    let second = create_entity_outcome(execution, &runtime, "b");
    let _third = create_entity_outcome(execution, &runtime, "c");

    assert!(runtime
        .history_authority()
        .evict_commit_envelope_preserving_patch_position_for_durable_recovery_test(
            second.commit.commit_id,
        ));

    let patch_batch = runtime
        .publication()
        .read_patch_stream(PatchStreamRequest {
            after_position: None,
            max_commits: 8,
        })
        .unwrap();
    let subscriber_batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(8))
        .unwrap();

    assert_eq!(subscriber_batch.patches, patch_batch.patches);
}
