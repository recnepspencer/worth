use crate::tests::support::*;

#[test]
fn subscriber_stream_recovers_from_durable_canonical_envelopes_when_checkpoint_is_not_in_memory() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "a");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let _second = create_entity_outcome(execution, &runtime, "b");

    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id));

    let checkpoint = checkpoint_for_schema_version(PatchStreamPosition(1), SchemaVersionId(1));
    let resumed = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(checkpoint, 8))
        .unwrap();

    assert_eq!(resumed.patches.len(), 1);
    assert_eq!(resumed.patches[0].position.0, 2);
    assert_eq!(
        resumed.recovery_decision.source,
        crate::facade::publication::SubscriberRecoverySource::DurableCanonicalRecovery
    );
    assert_eq!(
        resumed
            .latest_available_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.position().0),
        Some(2)
    );
}

#[test]
fn subscriber_stream_reports_durable_latest_available_checkpoint_when_latest_head_is_not_in_memory()
{
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "a");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let second = create_entity_outcome(execution, &runtime, "b");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();

    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id));
    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(second.commit.commit_id));

    let checkpoint = checkpoint_for_schema_version(PatchStreamPosition(1), SchemaVersionId(1));
    let resumed = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(checkpoint, 8))
        .unwrap();

    assert_eq!(resumed.patches.len(), 1);
    assert_eq!(resumed.patches[0].position.0, 2);
    assert_eq!(
        resumed.recovery_decision.source,
        crate::facade::publication::SubscriberRecoverySource::DurableCanonicalRecovery
    );
    assert_eq!(
        resumed
            .latest_available_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.position().0),
        Some(2)
    );
}
