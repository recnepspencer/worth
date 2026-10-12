use crate::publication::cdc::data::{NormalizedContinuationProof, SubscriberContinuationSummary};
use crate::schema::data::{DescriptorSemanticsVersion, SchemaContinuationClassification};
use crate::tests::support::*;

#[test]
fn subscriber_stream_rejects_durable_only_checkpoint_with_descriptor_version_mismatch() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "anchor");
    runtime.durability_authority().checkpoint().unwrap();

    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id));

    let checkpoint = checkpoint_for_schema_version(PatchStreamPosition(1), SchemaVersionId(1))
        .with_incoherent_continuation_for_test(
            "default.subscriber.contract".to_string(),
            NormalizedContinuationProof::new(Vec::new(), DescriptorSemanticsVersion(99)),
            SubscriberContinuationSummary::new(
                "default.subscriber.contract".to_string(),
                SchemaContinuationClassification::ContinueUnchanged,
                0,
                0,
                DescriptorSemanticsVersion(99),
                false,
            ),
            DescriptorSemanticsVersion(99),
        );

    let error = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(checkpoint, 1))
        .unwrap_err();

    assert_eq!(
        error.class,
        SubscriberStreamFailureClass::DescriptorVersionMismatch
    );
    assert_eq!(
        error
            .latest_available_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.position().0),
        Some(1)
    );
}
