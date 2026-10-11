use crate::tests::support::*;

#[test]
fn latest_bundle_remains_a_convenience_surface_not_the_subscriber_contract() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(execution, &runtime, "anchor");

    let publication = runtime.publication();
    let bundle = publication.latest_bundle().unwrap();
    let observation = publication.observation_snapshot();
    let subscriber = publication
        .read_subscriber_stream(SubscriberResumeRequest::from_head(8))
        .unwrap();

    assert_eq!(bundle.patch, subscriber.patches[0]);
    assert_eq!(observation.latest_commit_id, Some(bundle.commit.commit_id));
    assert_eq!(
        observation.publication_snapshot_id,
        Some(bundle.snapshot.snapshot_id)
    );
    assert_eq!(
        observation.latest_patch_position,
        Some(bundle.patch.position)
    );
    assert!(subscriber.latest_available_checkpoint.is_some());
}

#[test]
fn publication_snapshots_are_the_outward_artifact_surface_for_latest_exports() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(execution, &runtime, "anchor");

    let publication = runtime.publication();
    let artifact_snapshot = publication.artifact_snapshot();
    let diagnostics_snapshot = publication.diagnostics_snapshot();

    assert_eq!(
        artifact_snapshot.observation,
        publication.observation_snapshot()
    );
    assert_eq!(artifact_snapshot.latest_patch, publication.latest_patch());
    assert_eq!(artifact_snapshot.latest_replay, publication.latest_replay());
    assert_eq!(
        diagnostics_snapshot.observation,
        publication.observation_snapshot()
    );
    assert_eq!(
        diagnostics_snapshot.diagnostics,
        publication.diagnostic_artifacts()
    );
}
