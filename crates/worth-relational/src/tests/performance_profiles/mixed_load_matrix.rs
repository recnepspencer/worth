use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_mixed_load_matrix() {
    let suite = "mixed_load_matrix";

    let snapshot_version_pressure_samples =
        capture_perf_samples(suite, "concurrent_snapshot_version_read_pressure", || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            let created = create_entity_outcome(&runtime, "baseline");
            let created_version_id = created.version_id;
            let entity = changed_entities(&created)[0];
            let explicit_snapshot = runtime.visibility_authority().snapshot();
            let updated = update_entity(&runtime, entity, "mutated");

            let serial_snapshot_name = {
                let read = runtime
                    .read_truth()
                    .read_snapshot(&explicit_snapshot)
                    .expect("snapshot read");
                read_entity_name(read.get_entity(entity).expect("snapshot entity"))
                    .expect("snapshot name")
                    .to_string()
            };
            let serial_version_name = {
                let read = runtime.read_truth().read_version(created_version_id);
                read_entity_name(read.get_entity(entity).expect("version entity"))
                    .expect("version name")
                    .to_string()
            };
            let serial_latest_name = {
                let read = runtime
                    .read_truth()
                    .read_snapshot(&updated.snapshot)
                    .expect("latest read");
                read_entity_name(read.get_entity(entity).expect("latest entity"))
                    .expect("latest name")
                    .to_string()
            };

            runtime.performance_access().reset_counters();
            let runtime = Arc::new(runtime);
            let started_at = Instant::now();
            std::thread::scope(|scope| {
                let mut readers = Vec::new();
                for _ in 0..8 {
                    let runtime = Arc::clone(&runtime);
                    let explicit_snapshot = explicit_snapshot.clone();
                    let published_snapshot = updated.snapshot.clone();
                    readers.push(scope.spawn(move || {
                        let snapshot_read = runtime
                            .read_truth()
                            .read_snapshot(&explicit_snapshot)
                            .expect("thread snapshot read");
                        let version_read = runtime.read_truth().read_version(created_version_id);
                        let latest_read = runtime
                            .read_truth()
                            .read_snapshot(&published_snapshot)
                            .expect("thread latest read");
                        (
                            read_entity_name(
                                snapshot_read.get_entity(entity).expect("snapshot entity"),
                            )
                            .expect("snapshot name")
                            .to_string(),
                            read_entity_name(
                                version_read.get_entity(entity).expect("version entity"),
                            )
                            .expect("version name")
                            .to_string(),
                            read_entity_name(
                                latest_read.get_entity(entity).expect("latest entity"),
                            )
                            .expect("latest name")
                            .to_string(),
                        )
                    }));
                }

                for reader in readers {
                    let (snapshot_name, version_name, latest_name) = reader.join().unwrap();
                    assert_eq!(snapshot_name, serial_snapshot_name);
                    assert_eq!(version_name, serial_version_name);
                    assert_eq!(latest_name, serial_latest_name);
                }
            });
            let elapsed_micros = started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "reader_count": 8,
                    "snapshot_name_len": serial_snapshot_name.len(),
                    "version_name_len": serial_version_name.len(),
                    "latest_name_len": serial_latest_name.len(),
                    "visibility_cache_hits": counters.visibility_cache_hits,
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "concurrent_snapshot_version_read_pressure",
        &snapshot_version_pressure_samples,
        &[
            ("reader_count", &["reader_count"]),
            ("visibility_cache_hits", &["visibility_cache_hits"]),
        ],
    );
    assert!(snapshot_version_pressure_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &snapshot_version_pressure_samples,
        "mixed read pressure should preserve snapshot/version truth and hit the visibility cache",
        |metrics| {
            metric_u64(metrics, "reader_count") == 8
                && metric_u64(metrics, "snapshot_name_len") > 0
                && metric_u64(metrics, "version_name_len") > 0
                && metric_u64(metrics, "latest_name_len") > 0
                && metric_u64(metrics, "visibility_cache_hits") > 0
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );

    let relation_index_pressure_samples =
        capture_perf_samples(suite, "concurrent_relation_index_parity_pressure", || {
            let runtime = runtime_with_test_schema();
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
                plan_key: DeterministicQueryPlanKey(4401),
                target_count_hint: 0,
            };
            let expected = runtime
                .index_access()
                .execute_query_plan_with_index_parity(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, packet.clone())
                        .expect("baseline relation plan"),
                    IndexParityMode::CertificationParity,
                )
                .expect("baseline relation outcome");

            runtime.performance_access().reset_counters();
            let runtime = Arc::new(runtime);
            let started_at = Instant::now();
            std::thread::scope(|scope| {
                let mut readers = Vec::new();
                for _ in 0..8 {
                    let runtime = Arc::clone(&runtime);
                    let snapshot = snapshot.clone();
                    let packet = packet.clone();
                    let expected = expected.clone();
                    readers.push(scope.spawn(move || {
                        let outcome = runtime
                            .index_access()
                            .execute_query_plan_with_index_parity(
                                runtime
                                    .read_truth()
                                    .plan_query_packet(&snapshot, packet)
                                    .expect("thread relation plan"),
                                IndexParityMode::CertificationParity,
                            )
                            .expect("thread relation outcome");
                        assert_eq!(outcome.access_path, expected.access_path);
                        assert_eq!(outcome.execution.result, expected.execution.result);
                        assert_eq!(outcome.parity_basis_digest, expected.parity_basis_digest);
                    }));
                }

                for reader in readers {
                    reader.join().unwrap();
                }
            });
            let elapsed_micros = started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "reader_count": 8,
                    "matched_relation_count": expected.execution.result.relations.len(),
                    "access_path": format!("{:?}", expected.access_path),
                    "parity_digest_present": !expected.parity_basis_digest.is_empty(),
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "concurrent_relation_index_parity_pressure",
        &relation_index_pressure_samples,
        &[
            ("reader_count", &["reader_count"]),
            ("matched_relation_count", &["matched_relation_count"]),
        ],
    );
    assert!(relation_index_pressure_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &relation_index_pressure_samples,
        "mixed relation index pressure should preserve certification parity under scheduler contention",
        |metrics| {
            metric_u64(metrics, "reader_count") == 8
                && metric_u64(metrics, "matched_relation_count") == 0
                && metrics["parity_digest_present"].as_bool() == Some(true)
                && metrics["access_path"].as_str().unwrap_or("").contains("DerivedIndexGeneration")
                && counter_u64(metrics, "query_index_attempt_count") == 8
                && counter_u64(metrics, "query_index_path_count") == 8
                && counter_u64(metrics, "query_index_parity_verification_count") == 8
                && counter_u64(metrics, "query_index_rejection_count") == 0
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );
}
