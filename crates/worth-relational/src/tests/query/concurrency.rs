use std::sync::Arc;

use crate::diagnostics::data::DiagnosticCode;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::facade::query::IndexParityMode;
use crate::tests::support::*;

#[test]
fn concurrent_snapshot_and_version_reads_match_serial_truth() {
    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let created = create_entity_outcome(&runtime, "before");
    let created_version_id = created.version_id;
    let entity = changed_entities(&created)[0];
    let explicit_snapshot = runtime.visibility_authority().snapshot();
    let updated = update_entity(&runtime, entity, "after");
    let serial_snapshot_name = {
        let read = runtime
            .read_truth()
            .read_snapshot(&explicit_snapshot)
            .unwrap();
        read_entity_name(read.get_entity(entity).unwrap())
            .unwrap()
            .to_string()
    };
    let serial_version_name = {
        let read = runtime.read_truth().read_version(created_version_id);
        read_entity_name(read.get_entity(entity).unwrap())
            .unwrap()
            .to_string()
    };
    let serial_latest_name = {
        let read = runtime
            .read_truth()
            .read_snapshot(&updated.snapshot)
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
                    .read_snapshot(&explicit_snapshot)
                    .unwrap();
                let version_read = runtime.read_truth().read_version(created_version_id);
                let latest_read = runtime
                    .read_truth()
                    .read_snapshot(&published_snapshot)
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
    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let created = create_entity_outcome(&runtime, "baseline");
    let created_version_id = created.version_id;
    let entity = changed_entities(&created)[0];
    let explicit_snapshot = runtime.visibility_authority().snapshot();
    let updated = update_entity(&runtime, entity, "mutated");
    let _ = create_entity_outcome(&runtime, "churn-1");
    let _ = create_entity_outcome(&runtime, "churn-2");
    let _ = create_entity_outcome(&runtime, "churn-3");
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
                let historical = runtime.read_truth().read_version(created_version_id);
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
    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let created = create_entity_outcome(&runtime, "baseline");
    let entity = changed_entities(&created)[0];
    let updated = update_entity(&runtime, entity, "mutated");
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
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::GeometryKernel)
        .schema_registry(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .execution_model(crate::facade::runtime::RelationalExecutionModel::ParallelPreparation)
        .build();
    let seeds = vec![
        create_entity_in_partition(&runtime, "s0", PartitionId(7)),
        create_entity_in_partition(&runtime, "s1", PartitionId(11)),
        create_entity_in_partition(&runtime, "s2", PartitionId(13)),
        create_entity_in_partition(&runtime, "s3", PartitionId(17)),
        create_entity_in_partition(&runtime, "s4", PartitionId(19)),
    ];
    let neighbors = [
        create_entity_in_partition(&runtime, "n0", PartitionId(23)),
        create_entity_in_partition(&runtime, "n1", PartitionId(29)),
        create_entity_in_partition(&runtime, "n2", PartitionId(31)),
        create_entity_in_partition(&runtime, "n3", PartitionId(37)),
        create_entity_in_partition(&runtime, "n4", PartitionId(41)),
    ];
    for (index, (seed, neighbor)) in seeds.iter().zip(neighbors.iter()).enumerate() {
        create_relation_in_partition(
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
        )
        .expect("baseline query outcome")
        .result;

    let churn_entity = create_entity_in_partition(&runtime, "rewrite-anchor", PartitionId(53));
    let _ = update_entity(&runtime, churn_entity, "rewrite-anchor-2");
    let extra_neighbor = create_entity_in_partition(&runtime, "late-neighbor", PartitionId(59));
    let _ = create_relation_in_partition(
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
                let result = runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, packet)
                            .expect("thread query plan"),
                    )
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

#[test]
fn concurrent_relation_index_certification_parity_stays_stable_under_scheduler_pressure() {
    let runtime = runtime_with_test_schema_execution_model(
        crate::facade::runtime::RelationalExecutionModel::ParallelPreparation,
    );
    let source = create_entity_outcome(&runtime, "source");
    let source_id = changed_entities(&source)[0];
    let targets = [
        create_entity_in_partition(&runtime, "r0", PartitionId(7)),
        create_entity_in_partition(&runtime, "r1", PartitionId(11)),
        create_entity_in_partition(&runtime, "r2", PartitionId(13)),
    ];
    for (index, target) in targets.into_iter().enumerate() {
        create_relation_in_partition(
            &runtime,
            source_id,
            target,
            if index < 2 { "fast" } else { "slow" },
            PartitionId(23 + index as u32),
        );
    }
    let commit = create_entity_outcome(&runtime, "anchor");
    let relation_index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "relation.name".to_string(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![relation_index.index_id],
        });

    let snapshot = commit.snapshot.clone();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "relation-index-certification".to_string(),
        context_id: context,
        scope: QueryScope::RelationFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("fast"),
            partition_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalRelationIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(1401),
        target_count_hint: 0,
    };
    let baseline = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet.clone())
                .expect("baseline plan"),
            IndexParityMode::CertificationParity,
        )
        .expect("baseline outcome");
    let runtime = Arc::new(runtime);

    std::thread::scope(|scope| {
        let mut readers = Vec::new();
        for _ in 0..8 {
            let runtime = Arc::clone(&runtime);
            let snapshot = snapshot.clone();
            let packet = packet.clone();
            let expected = baseline.clone();
            readers.push(scope.spawn(move || {
                let outcome = runtime
                    .index_access()
                    .execute_query_plan_with_index_parity(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, packet)
                            .expect("thread plan"),
                        IndexParityMode::CertificationParity,
                    )
                    .expect("thread outcome");
                assert_eq!(outcome.access_path, expected.access_path);
                assert_eq!(outcome.execution.result, expected.execution.result);
                assert_eq!(outcome.parity_basis_digest, expected.parity_basis_digest);
            }));
        }

        for reader in readers {
            reader.join().unwrap();
        }
    });
}
