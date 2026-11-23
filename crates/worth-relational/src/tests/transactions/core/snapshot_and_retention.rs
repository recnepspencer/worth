use crate::facade::config::{MvccConfig, RetentionBackend, SnapshotReleasePolicy};
use crate::tests::support::*;

#[test]
fn observed_storage_blocks_reclaim_until_edition_is_released() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let create_outcome = create_entity_outcome(execution, &runtime, "first");
    let entity_a = changed_entities(&create_outcome)[0];
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&create_outcome.snapshot)
        .is_ok());
    let delete_outcome = delete_entity(execution, &runtime, entity_a);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&delete_outcome.snapshot)
        .is_ok());
    let edition = runtime.storage_access().current_edition();
    let denied = runtime.retention().try_run_pass(execution).unwrap_err();
    assert_eq!(
        denied.stop(),
        crate::runtime::RelationalRetentionPassStop::StorageObserved
    );
    assert!(denied.progress().slots_scanned() > 0);
    assert_eq!(denied.progress().outcome().entity_reclaimed, 0);
    drop(edition);

    let retention = runtime.retention().run_pass(execution).unwrap();
    let entity_b = create_entity(execution, &runtime, "second");

    assert!(retention.entity_reclaimed <= 1);
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .reusable_entity_slots,
        0
    );
    assert_eq!(entity_a.local_slot, entity_b.local_slot);
    assert!(entity_b.generation.0 > entity_a.generation.0);
}

#[test]
fn snapshots_resolve_historical_entity_aspects_by_version() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let create_outcome = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&create_outcome)[0];
    let snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let update_outcome = update_entity(execution, &runtime, entity, "after");

    let old_read = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let current_read = runtime
        .read_truth()
        .read_snapshot(&update_outcome.snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let version_read = runtime
        .read_truth()
        .read_version(execution, create_outcome.version_id)
        .expect("read request remains live");

    assert_eq!(
        read_entity_name(old_read.get_entity(entity).unwrap()),
        Some("before".into())
    );
    assert_eq!(
        read_entity_name(current_read.get_entity(entity).unwrap()),
        Some("after".into())
    );
    assert_eq!(
        read_entity_name(version_read.get_entity(entity).unwrap()),
        Some("before".into())
    );
}

#[test]
fn historical_reads_preserve_generation_and_aspects_after_slot_reuse() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let created = create_entity_outcome(execution, &runtime, "before");
    let original = changed_entities(&created)[0];
    let deleted = delete_entity(execution, &runtime, original);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    let _ = runtime.retention().run_pass(execution).unwrap();
    let replacement = create_entity(execution, &runtime, "after");

    let historical = runtime
        .read_truth()
        .read_version(execution, created.version_id)
        .expect("read request remains live");
    let record = historical.get_entity(original).unwrap();

    assert_eq!(record.entity_id, original);
    assert_eq!(read_entity_name(record), Some("before".into()));
    assert_eq!(original.local_slot, replacement.local_slot);
    assert!(replacement.generation.0 > original.generation.0);
}

mod read_record_metadata;

#[path = "snapshot_and_retention/epoch.rs"]
mod epoch;
