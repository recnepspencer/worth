use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

use worth_relational::facade::{
    config::{
        ConfigValueSource, MvccConfig, RelationalRuntimeProfile, RetentionBackend,
        SnapshotReleasePolicy,
    },
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    snapshots::SnapshotHandle,
};
use worth_testkit_relational::{
    aspect_storage::publications::changed_entities,
    durable_publication::{
        create_entity_outcome, memory_runtime, memory_runtime_with_profile, schema,
    },
    ordinary_records::{create_entity, delete_entity},
};

#[test]
fn snapshot_reads_are_immutable_after_later_mutation() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let first = create_entity(execution, &runtime, "first");
    let snapshot = main_snapshot(&runtime);
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
    let host = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let first = memory_runtime();
    let second = memory_runtime();
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
        Err(
            worth_relational::facade::snapshots::RelationalSnapshotReleaseDenial::ForeignRuntime { .. }
        )
    ));
    assert!(first
        .read_truth()
        .inspect_snapshot(execution, &first_snapshot)
        .unwrap()
        .is_some());
    assert!(first.snapshots().release_snapshot(&first_snapshot).is_ok());
    assert_eq!(
        first.snapshots().release_snapshot(&first_snapshot),
        Err(worth_relational::facade::snapshots::RelationalSnapshotReleaseDenial::UnknownSnapshot)
    );
    assert!(second
        .snapshots()
        .release_snapshot(&second_snapshot)
        .is_ok());
}

#[test]
fn profile_resolution_and_provenance_are_explicit() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::GeometryKernel)
        .schema_registry(schema())
        .entity_capacity(999)
        .build()
        .expect("valid native runtime configuration");

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
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime_with_profile(RelationalRuntimeProfile::AiWorkflow);
    let create_outcome = create_entity_outcome(execution, &runtime, "pinned");
    let create_snapshot = main_snapshot(&runtime);
    let entity = changed_entities(&create_outcome)[0];
    let _delete_outcome = delete_entity(execution, &runtime, entity);
    let delete_snapshot = main_snapshot(&runtime);
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
        .snapshots()
        .release_snapshot(&create_snapshot)
        .is_ok());
    assert!(runtime
        .snapshots()
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
fn external_basis_retention_keeps_observed_truth_until_independent_release() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::ChipSimulation)
        .schema_registry(schema())
        .mvcc(MvccConfig {
            track_visibility_metadata: true,
            snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
            auto_reclaim_deleted_records: true,
            reclaim_batch_size: 128,
            retention_backend: RetentionBackend::EpochChunkRetention,
        })
        .build()
        .expect("valid native runtime configuration");
    let created = create_entity_outcome(execution, &runtime, "managed-observation-basis");
    let entity = changed_entities(&created)[0];
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let external = runtime
        .retain_component_basis(&basis)
        .expect("owner should retain its admitted component basis");
    drop(basis);
    assert!(runtime
        .snapshots()
        .release_snapshot(&created.snapshot)
        .is_ok());

    let deleted = delete_entity(execution, &runtime, entity);
    assert!(runtime
        .snapshots()
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

fn main_snapshot(runtime: &RelationalRuntime) -> SnapshotHandle {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap()
}
