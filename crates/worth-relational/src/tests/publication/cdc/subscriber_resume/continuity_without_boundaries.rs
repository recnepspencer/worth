use crate::schema::data::SchemaContinuationClassification;
use crate::tests::support::*;

#[test]
fn subscriber_stream_without_schema_boundaries_reports_unchanged_continuity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(execution, &runtime, "a");
    let _second = create_entity_outcome(execution, &runtime, "b");

    let batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(2))
        .unwrap();

    assert_eq!(
        batch.continuation.continuation_outcome(),
        SchemaContinuationClassification::ContinueUnchanged
    );
    assert!(batch.continuation.crossed_boundaries().is_empty());
    assert!(!batch.continuation.contract_upgrade_applied());

    let next_checkpoint = batch.next_checkpoint.unwrap();
    assert_eq!(
        next_checkpoint
            .normalized_continuation_proof()
            .normalized_boundary_count(),
        0
    );
    assert!(next_checkpoint
        .normalized_continuation_proof()
        .boundary_fingerprints()
        .is_empty());
}
