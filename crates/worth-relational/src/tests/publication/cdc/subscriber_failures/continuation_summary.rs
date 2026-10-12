use crate::publication::cdc::data::SubscriberContinuationSummary;
use crate::schema::data::{DescriptorSemanticsVersion, SchemaContinuationClassification};
use crate::tests::support::*;

#[test]
fn subscriber_stream_rejects_checkpoint_with_inconsistent_continuation_summary() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(execution, &runtime, "anchor");

    let batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(1))
        .unwrap();
    let checkpoint = batch
        .next_checkpoint
        .unwrap()
        .with_incoherent_continuation_for_test(
            "default.subscriber.contract".to_string(),
            batch
                .latest_available_checkpoint
                .as_ref()
                .map(
                    |checkpoint: &crate::publication::cdc::data::SubscriberCheckpoint| {
                        checkpoint.normalized_continuation_proof().clone()
                    },
                )
                .unwrap_or_default(),
            SubscriberContinuationSummary::new(
                "default.subscriber.contract".to_string(),
                SchemaContinuationClassification::ContinueWithVisibleBridge,
                1,
                99,
                DescriptorSemanticsVersion::default(),
                false,
            ),
            DescriptorSemanticsVersion::default(),
        );

    let error = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(checkpoint, 1))
        .unwrap_err();

    assert_eq!(
        error.class,
        SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch
    );
}
