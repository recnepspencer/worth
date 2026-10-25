use crate::facade::config::{
    ConfigValueSource, MvccConfig, RetentionBackend, SnapshotReleasePolicy,
};
use crate::tests::support::*;

#[test]
fn entity_slot_reuse_increments_generation() {
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
fn snapshot_reads_are_immutable_after_later_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let first = create_entity(execution, &runtime, "first");
    let snapshot = runtime.visibility_authority().snapshot();
    let _second = create_entity(execution, &runtime, "second");
    let read = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("read request remains live")
        .unwrap();

    assert!(read.get_entity(first).is_some());
    assert_eq!(read.entities().len(), 1);
}

#[test]
fn snapshot_release_rejects_colliding_foreign_ids_and_double_release() {
    let first = runtime_with_test_schema();
    let second = runtime_with_test_schema();
    let (_, first_basis) = first.observe_branch(&first.main_branch_identity()).unwrap();
    let (_, second_basis) = second
        .observe_branch(&second.main_branch_identity())
        .unwrap();
    let first_snapshot = first
        .snapshots()
        .snapshot_for_observation(&first_basis.observation())
        .unwrap();
    let second_snapshot = second
        .snapshots()
        .snapshot_for_observation(&second_basis.observation())
        .unwrap();
    assert_eq!(first_snapshot.snapshot_id(), second_snapshot.snapshot_id());

    assert!(matches!(
        first.snapshots().release_snapshot(&second_snapshot),
        Err(crate::visibility::RelationalSnapshotReleaseDenial::ForeignRuntime { .. })
    ));
    assert!(first
        .read_truth()
        .inspect_snapshot(&first_snapshot)
        .is_some());
    assert!(first.snapshots().release_snapshot(&first_snapshot).is_ok());
    assert_eq!(
        first.snapshots().release_snapshot(&first_snapshot),
        Err(crate::visibility::RelationalSnapshotReleaseDenial::UnknownSnapshot)
    );
    assert!(second
        .snapshots()
        .release_snapshot(&second_snapshot)
        .is_ok());
}

#[test]
fn snapshots_resolve_historical_entity_aspects_by_version() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let create_outcome = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&create_outcome)[0];
    let snapshot = runtime.visibility_authority().snapshot();
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

#[test]
fn profile_resolution_and_provenance_are_explicit() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::GeometryKernel)
        .schema_registry(test_schema_registry())
        .entity_capacity(999)
        .build();

    assert_eq!(
        runtime.config().profile,
        RelationalRuntimeProfile::GeometryKernel
    );
    assert_eq!(runtime.config().storage.initial_entity_capacity, 999);
    assert!(runtime.config().diagnostics.profile.detailed_traces_enabled);
    assert_eq!(runtime.config().storage.layout.entity_chunk_size, 2048);
    assert_eq!(
        runtime
            .config()
            .provenance
            .source_for("storage.initial_entity_capacity")
            .unwrap()
            .source,
        ConfigValueSource::BuilderOverride
    );
    assert_eq!(
        runtime
            .config()
            .provenance
            .source_for("storage.layout")
            .unwrap()
            .source,
        ConfigValueSource::ProfileDefault
    );
    assert_eq!(
        runtime
            .config()
            .provenance
            .source_for("visibility.cache_policy")
            .unwrap()
            .source,
        ConfigValueSource::ProfileDefault
    );
    assert!(runtime.config().visibility.cache_policy.enabled);
}

#[test]
fn snapshot_root_obligation_survives_substrate_reclaim_until_release() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let create_outcome = create_entity_outcome(execution, &runtime, "pinned");
    let create_snapshot = runtime.visibility_authority().snapshot();
    let entity = changed_entities(&create_outcome)[0];
    let _delete_outcome = delete_entity(execution, &runtime, entity);
    let delete_snapshot = runtime.visibility_authority().snapshot();
    let first_retention = runtime.retention().run_pass(execution).unwrap();

    assert!(first_retention.entity_reclaimed <= 1);
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .reusable_entity_slots,
        1
    );
    assert_eq!(first_retention.entity_chunks_scanned, 1);
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

    assert_eq!(second_retention.entity_reclaimed, 0);
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .reusable_entity_slots,
        1
    );
}

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
        .build();
    let create_outcome = create_entity_outcome(execution, &runtime, "epoch-pinned");
    let create_snapshot = runtime.visibility_authority().snapshot();
    let entity = changed_entities(&create_outcome)[0];
    let _delete_outcome = delete_entity(execution, &runtime, entity);
    let delete_snapshot = runtime.visibility_authority().snapshot();

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

#[test]
fn external_basis_retention_keeps_observed_truth_until_independent_release() {
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
        .build();
    let created = create_entity_outcome(execution, &runtime, "managed-observation-basis");
    let entity = changed_entities(&created)[0];
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let external = runtime
        .retain_component_basis(&basis)
        .expect("owner should retain its admitted component basis");
    drop(basis);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());

    let deleted = delete_entity(execution, &runtime, entity);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    let retained = runtime.retention().run_pass(execution).unwrap();

    assert!(retained.entity_reclaimed <= 1);
    let readmitted = runtime
        .readmit_branch_basis(&descriptor)
        .expect("external retention keeps exact readmission available");
    let retained_snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&readmitted.observation())
        .unwrap();
    assert!(runtime
        .read_truth()
        .read_snapshot(&retained_snapshot, execution)
        .expect("read request remains live")
        .and_then(|read| read.get_entity(entity).cloned())
        .is_some());

    assert!(runtime
        .snapshots()
        .release_snapshot(&retained_snapshot)
        .is_ok());
    drop(readmitted);
    let receipt = runtime.release_component_basis(external).unwrap();
    assert_eq!(receipt.descriptor(), &descriptor);
    let released = runtime.retention().run_pass(execution).unwrap();
    assert!(released.entity_reclaimed <= 1);
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .reusable_entity_slots,
        1
    );
}

mod read_record_metadata;
