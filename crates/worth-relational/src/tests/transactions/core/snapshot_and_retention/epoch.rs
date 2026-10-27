use super::*;

#[test]
fn epoch_retention_backend_preserves_snapshot_visibility_until_release() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::ChipSimulation)
        .schema_registry(test_schema_registry())
        .mvcc(MvccConfig {
            track_visibility_metadata: true,
            snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
            auto_reclaim_deleted_records: true,
            reclaim_batch_size: 128,
            retention_backend: RetentionBackend::EpochChunkRetention,
        })
        .build()
        .expect("valid native runtime configuration");
    let create_outcome = create_entity_outcome(execution, &runtime, "epoch-pinned");
    let create_snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let entity = changed_entities(&create_outcome)[0];
    let _delete_outcome = delete_entity(execution, &runtime, entity);
    let delete_snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");

    let first_retention = runtime.retention().run_pass(execution).unwrap();
    assert_eq!(
        runtime.config().storage.retention.backend,
        RetentionBackend::EpochChunkRetention
    );
    assert_eq!(first_retention.entity_reclaimed, 0);
    assert!(runtime
        .read_truth()
        .read_snapshot(&create_snapshot, execution)
        .expect("read request remains live")
        .unwrap()
        .get_entity(entity)
        .is_some());

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&create_snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&delete_snapshot)
        .is_ok());
    let second_retention = runtime.retention().run_pass(execution).unwrap();

    assert!(second_retention.entity_reclaimed <= 1);
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .reusable_entity_slots,
        1
    );
}
