use crate::tests::support::*;

#[test]
fn subscriber_stream_resume_uses_checkpoint_type_and_batches_without_duplication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(execution, &runtime, "a");
    let _second = create_entity_outcome(execution, &runtime, "b");
    let _third = create_entity_outcome(execution, &runtime, "c");

    let first_batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(2))
        .unwrap();
    let resumed = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            first_batch.next_checkpoint.clone().unwrap(),
            2,
        ))
        .unwrap();

    assert_eq!(first_batch.patches.len(), 2);
    assert_eq!(first_batch.next_checkpoint.unwrap().position().0, 2);
    assert_eq!(
        first_batch
            .latest_available_checkpoint
            .unwrap()
            .position()
            .0,
        3
    );
    assert_eq!(resumed.patches.len(), 1);
    assert_eq!(resumed.resumed_from.unwrap().position().0, 2);
    assert_eq!(resumed.patches[0].position.0, 3);
}
