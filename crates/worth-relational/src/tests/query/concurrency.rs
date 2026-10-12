use std::sync::Arc;

use crate::diagnostics::data::DiagnosticCode;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::facade::query::IndexParityMode;
use crate::tests::support::*;

#[test]
fn concurrent_snapshot_and_version_reads_match_serial_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let created = create_entity_outcome(execution, &runtime, "before");
    let created_version_id = created.version_id;
    let entity = changed_entities(&created)[0];
    let explicit_snapshot = runtime.visibility_authority().snapshot();
    let updated = update_entity(execution, &runtime, entity, "after");
    let serial_snapshot_name = {
        let read = runtime
            .read_truth()
            .read_snapshot(&explicit_snapshot, execution)
            .unwrap();
        read_entity_name(read.get_entity(entity).unwrap())
            .unwrap()
            .to_string()
    };
    let serial_version_name = {
        let read = runtime
            .read_truth()
            .read_version(execution, created_version_id);
        read_entity_name(read.get_entity(entity).unwrap())
            .unwrap()
            .to_string()
    };
    let serial_latest_name = {
        let read = runtime
            .read_truth()
            .read_snapshot(&updated.snapshot, execution)
            .unwrap();
        read_entity_name(read.get_entity(entity).unwrap())
            .unwrap()
            .to_string()
    };
    let runtime = Arc::new(runtime);

    std::thread::scope(|scope| {
        let mut snapshot_threads = Vec::new();
        for _ in 0..8 {
            let runtime = Arc::clone(&runtime);
            let explicit_snapshot = explicit_snapshot.clone();
            let published_snapshot = updated.snapshot.clone();
            snapshot_threads.push(scope.spawn(move || {
                let snapshot_read = runtime
                    .read_truth()
                    .read_snapshot(&explicit_snapshot, execution)
                    .unwrap();
                let version_read = runtime
                    .read_truth()
                    .read_version(execution, created_version_id);
                let latest_read = runtime
                    .read_truth()
                    .read_snapshot(&published_snapshot, execution)
                    .unwrap();
                (
                    read_entity_name(snapshot_read.get_entity(entity).unwrap())
                        .unwrap()
                        .to_string(),
                    read_entity_name(version_read.get_entity(entity).unwrap())
                        .unwrap()
                        .to_string(),
                    read_entity_name(latest_read.get_entity(entity).unwrap())
                        .unwrap()
                        .to_string(),
                )
            }));
        }

        for thread in snapshot_threads {
            let (snapshot_name, version_name, latest_name) = thread.join().unwrap();
            assert_eq!(snapshot_name, serial_snapshot_name);
            assert_eq!(version_name, serial_version_name);
            assert_eq!(latest_name, serial_latest_name);
        }
    });
}

#[test]
fn concurrent_read_pressure_keeps_cache_diagnostics_coherent() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let created = create_entity_outcome(execution, &runtime, "baseline");
    let created_version_id = created.version_id;
    let entity = changed_entities(&created)[0];
    let explicit_snapshot = runtime.visibility_authority().snapshot();
    let updated = update_entity(execution, &runtime, entity, "mutated");
    let _ = create_entity_outcome(execution, &runtime, "churn-1");
    let _ = create_entity_outcome(execution, &runtime, "churn-2");
    let _ = create_entity_outcome(execution, &runtime, "churn-3");
    runtime.performance_access().reset_counters();
    let runtime = Arc::new(runtime);

    std::thread::scope(|scope| {
        let mut readers = Vec::new();
        for _ in 0..6 {
            let runtime = Arc::clone(&runtime);
            let explicit_snapshot = explicit_snapshot.clone();
            let published_snapshot = updated.snapshot.clone();
            readers.push(scope.spawn(move || {
                let snapshot_diag = runtime
                    .read_truth()
                    .inspect_snapshot_read_path(&explicit_snapshot)
                    .expect("explicit snapshot diagnostics");
                let published_diag = runtime
                    .read_truth()
                    .inspect_snapshot_read_path(&published_snapshot)
                    .expect("published snapshot diagnostics");
                let historical = runtime
                    .read_truth()
                    .read_version(execution, created_version_id);
                let historical_name = read_entity_name(historical.get_entity(entity).unwrap())
                    .unwrap()
                    .to_string();
                (
                    snapshot_diag.entries.len(),
                    published_diag.entries.len(),
                    historical_name,
                )
            }));
        }

        for reader in readers {
            let (snapshot_entries, published_entries, historical_name) = reader.join().unwrap();
            assert!(snapshot_entries > 0);
            assert!(published_entries > 0);
            assert_eq!(historical_name, "baseline");
        }
    });

    let counters = runtime.performance_access().counters();
    assert!(counters.visibility_cache_hits > 0);
}

#[test]
fn published_snapshot_read_diagnostics_use_authoritative_binding_version() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let created = create_entity_outcome(execution, &runtime, "baseline");
    let entity = changed_entities(&created)[0];
    let updated = update_entity(execution, &runtime, entity, "mutated");
    let mut stale_handle = updated.snapshot.clone();
    stale_handle.version_id = created.snapshot.version_id;

    let diagnostics = runtime
        .read_truth()
        .inspect_snapshot_read_path(&stale_handle)
        .expect("published snapshot diagnostics");
    let publication_entry = diagnostics
        .entries
        .iter()
        .find(|entry| entry.code == DiagnosticCode::PublishedSnapshotHandleRead)
        .expect("published snapshot entry");

    assert_eq!(
        diagnostic_field(publication_entry, "version_id"),
        &crate::diagnostics::data::RelationalDiagnosticValue::VersionId(
            updated.snapshot.version_id
        )
    );
}

#[test]
fn concurrent_pinned_traversal_reads_stay_snapshot_stable_under_hot_rewrite_pressure() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::GeometryKernel)
        .schema_registry(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .build();
    let seeds = vec![
        create_entity_in_partition(execution, &runtime, "s0", PartitionId(7)),
        create_entity_in_partition(execution, &runtime, "s1", PartitionId(11)),
        create_entity_in_partition(execution, &runtime, "s2", PartitionId(13)),
        create_entity_in_partition(execution, &runtime, "s3", PartitionId(17)),
        create_entity_in_partition(execution, &runtime, "s4", PartitionId(19)),
    ];
    let neighbors = [
        create_entity_in_partition(execution, &runtime, "n0", PartitionId(23)),
        create_entity_in_partition(execution, &runtime, "n1", PartitionId(29)),
        create_entity_in_partition(execution, &runtime, "n2", PartitionId(31)),
        create_entity_in_partition(execution, &runtime, "n3", PartitionId(37)),
        create_entity_in_partition(execution, &runtime, "n4", PartitionId(41)),
    ];
    for (index, (seed, neighbor)) in seeds.iter().zip(neighbors.iter()).enumerate() {
        create_relation_in_partition(
            execution,
            &runtime,
            *seed,
            *neighbor,
            &format!("edge-{index}"),
            PartitionId(43 + index as u32),
        );
    }
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "snapshot-stable-traversal".to_string(),
        context_id: context,
        scope: QueryScope::OutgoingNeighborhood {
            seeds: Arc::from(seeds.clone()),
            relation_kind_scope: Some(Arc::from([KindId(2)])),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(1101),
        target_count_hint: seeds.len(),
    };
    let baseline = runtime
        .read_truth()
        .execute_query_plan(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet.clone())
                .expect("baseline query plan"),
            execution,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("baseline query outcome")
        .result;

    let churn_entity =
        create_entity_in_partition(execution, &runtime, "rewrite-anchor", PartitionId(53));
    let _ = update_entity(execution, &runtime, churn_entity, "rewrite-anchor-2");
    let extra_neighbor =
        create_entity_in_partition(execution, &runtime, "late-neighbor", PartitionId(59));
    let _ = create_relation_in_partition(
        execution,
        &runtime,
        seeds[0],
        extra_neighbor,
        "late-edge",
        PartitionId(61),
    );

    let runtime = Arc::new(runtime);
    std::thread::scope(|scope| {
        let mut readers = Vec::new();
        for _ in 0..8 {
            let runtime = Arc::clone(&runtime);
            let snapshot = snapshot.clone();
            let packet = packet.clone();
            let expected = baseline.clone();
            readers.push(scope.spawn(move || {
                // Each independent reader call owns its request budget.
                let host_request = crate::test_execution_policy::serial_request();
                let execution = &worth_execution::ExecutionRequest::serial(&host_request);
                let result = runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, packet)
                            .expect("thread query plan"),
                        execution,
                    )
                    .expect("request remains live")
                    .map(|read| read.outcome)
                    .expect("thread query outcome")
                    .result;
                assert_eq!(result, expected);
            }));
        }

        for reader in readers {
            reader.join().unwrap();
        }
    });
}

#[path = "concurrency/relation_index_parity.rs"]
mod relation_index_parity;
