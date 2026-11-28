use crate::tests::support::*;

#[test]
fn subscriber_patch_and_snapshot_visibility_agree_with_the_commit_result() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "anchor");

    let publication = runtime.publication();
    let observation = publication.observation_snapshot();
    let subscriber = publication
        .read_subscriber_stream(SubscriberResumeRequest::from_head(8))
        .unwrap();

    assert_eq!(subscriber.patches[0].position, committed.patch_position());
    assert_eq!(observation.latest_commit_id, Some(committed.commit.commit_id));
    assert_eq!(observation.publication_snapshot_id, Some(committed.final_snapshot_id()));
    assert!(subscriber.latest_available_checkpoint.is_some());
}

#[test]
fn publication_diagnostics_snapshot_projects_canonical_commit_summary() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "anchor");
    let snapshot = runtime.publication().diagnostics_snapshot();

    assert_eq!(snapshot.observation.latest_commit_id, Some(committed.commit.commit_id));
    assert_eq!(snapshot.diagnostics, [committed.envelope().diagnostics_summary.clone()]);
}
