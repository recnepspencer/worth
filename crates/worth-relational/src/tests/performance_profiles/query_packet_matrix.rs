use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_query_packet_matrix() {
    let suite = "query_packet_matrix";

    let explicit_target_samples =
        capture_perf_samples(suite, "explicit_targets_cross_partition", || {
            let runtime = runtime_with_test_schema();
            let targets = (0..64)
                .map(|index| {
                    let partition_id = match index % 4 {
                        0 => PartitionId(1),
                        1 => PartitionId(3),
                        2 => PartitionId(5),
                        _ => PartitionId(7),
                    };
                    RecordRef::Entity(create_entity_in_partition(
                        &runtime,
                        &format!("target-{index}"),
                        partition_id,
                    ))
                })
                .rev()
                .collect::<Vec<_>>();
            let snapshot = runtime.visibility_authority().snapshot();
            let packet = explicit_query_packet(&runtime, &snapshot, "explicit-targets", targets);

            runtime.performance_access().reset_counters();
            let planning_started_at = Instant::now();
            let planned = runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet)
                .expect("planned explicit query");
            let planning_micros = planning_started_at.elapsed().as_micros();
            let execution_started_at = Instant::now();
            let outcome = runtime
                .read_truth()
                .execute_query_plan(planned)
                .expect("explicit target query outcome");
            let execution_micros = execution_started_at.elapsed().as_micros();
            let elapsed_micros = planning_micros + execution_micros;
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "result_entities": outcome.result.entities.len(),
                    "result_relations": outcome.result.relations.len(),
                    "phase_timing": {
                        "planning_micros": planning_micros,
                        "execution_micros": execution_micros,
                    },
                    "shape_metrics": {
                        "packet_count": outcome.complexity.packet_count,
                        "scope_unit_count": counters.query_scope_unit_count,
                    },
                    "complexity": outcome.complexity,
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "explicit_targets_cross_partition",
        &explicit_target_samples,
        &[
            ("planning_micros", &["phase_timing", "planning_micros"]),
            ("execution_micros", &["phase_timing", "execution_micros"]),
            ("packet_count", &["shape_metrics", "packet_count"]),
            ("scope_unit_count", &["shape_metrics", "scope_unit_count"]),
        ],
    );
    assert!(explicit_target_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &explicit_target_samples,
        "explicit target queries should stay packetized and avoid broad storage scans",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "query_packet_count") <= 4
                && counter_u64(metrics, "query_index_attempt_count") == 0
                && metrics["result_entities"].as_u64() == Some(64)
        },
    );

    let kind_scan_samples =
        capture_perf_samples(suite, "entity_kind_scan_partition_matrix", || {
            let runtime = runtime_with_test_schema();
            for index in 0..128 {
                let partition_id = match index % 4 {
                    0 => PartitionId(1),
                    1 => PartitionId(3),
                    2 => PartitionId(5),
                    _ => PartitionId(7),
                };
                let _ =
                    create_entity_in_partition(&runtime, &format!("scan-{index}"), partition_id);
            }
            let snapshot = runtime.visibility_authority().snapshot();
            let context = runtime
                .read_truth()
                .query_plan_context(&snapshot)
                .expect("query plan context");
            let packet = PlannedQueryPacket {
                label: "entity-kind-scan".to_string(),
                context_id: context,
                scope: QueryScope::EntityKindScan {
                    kind_id: KindId(1),
                    partition_scope: Some(Arc::from([
                        PartitionId(1),
                        PartitionId(3),
                        PartitionId(5),
                        PartitionId(7),
                    ])),
                },
                locality: QueryLocalityClass::PartitionBounded {
                    partitions: Arc::from([
                        PartitionId(1),
                        PartitionId(3),
                        PartitionId(5),
                        PartitionId(7),
                    ]),
                },
                ordering: QueryOrderingContract::CanonicalEntityIdOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(20_001),
                target_count_hint: 0,
            };

            runtime.performance_access().reset_counters();
            let started_at = Instant::now();
            let outcome = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, packet)
                        .expect("planned query packet"),
                )
                .expect("entity kind scan outcome");
            let elapsed_micros = started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "result_entities": outcome.result.entities.len(),
                    "phase_timing": {
                        "planning_micros": 0,
                        "execution_micros": elapsed_micros,
                    },
                    "shape_metrics": {
                        "packet_count": outcome.complexity.packet_count,
                        "scope_unit_count": counters.query_scope_unit_count,
                    },
                    "complexity": outcome.complexity,
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "entity_kind_scan_partition_matrix",
        &kind_scan_samples,
        &[
            ("execution_micros", &["phase_timing", "execution_micros"]),
            ("packet_count", &["shape_metrics", "packet_count"]),
            ("scope_unit_count", &["shape_metrics", "scope_unit_count"]),
        ],
    );
    assert!(kind_scan_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &kind_scan_samples,
        "partition-bounded kind scans should remain bounded to the requested entity surface",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "query_packet_count") <= 4
                && counter_u64(metrics, "query_authoritative_entity_records_emitted") == 128
                && metrics["result_entities"].as_u64() == Some(128)
        },
    );

    let traversal_samples =
        capture_perf_samples(suite, "connectivity_traversal_cross_partition", || {
            let runtime = runtime_with_test_schema();
            let seeds = (0..12)
                .map(|index| {
                    create_entity_in_partition(
                        &runtime,
                        &format!("seed-{index}"),
                        PartitionId(10 + index as u32),
                    )
                })
                .collect::<Vec<_>>();
            let neighbors = (0..12)
                .map(|index| {
                    create_entity_in_partition(
                        &runtime,
                        &format!("neighbor-{index}"),
                        PartitionId(40 + index as u32),
                    )
                })
                .collect::<Vec<_>>();
            for (index, (seed, neighbor)) in seeds.iter().zip(neighbors.iter()).enumerate() {
                let _ = create_relation_in_partition(
                    &runtime,
                    *seed,
                    *neighbor,
                    &format!("edge-{index}"),
                    PartitionId(70 + index as u32),
                );
            }
            let snapshot = runtime.visibility_authority().snapshot();
            let context = runtime
                .read_truth()
                .query_plan_context(&snapshot)
                .expect("query plan context");
            let packet = PlannedQueryPacket {
                label: "connectivity-traversal".to_string(),
                context_id: context,
                scope: QueryScope::ConnectivityTraversal {
                    seeds: Arc::from(seeds.clone()),
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                    max_depth: Some(1),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(20_002),
                target_count_hint: seeds.len(),
            };

            runtime.performance_access().reset_counters();
            let planning_started_at = Instant::now();
            let planned = runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet)
                .expect("planned traversal packet");
            let planning_micros = planning_started_at.elapsed().as_micros();
            let execution_started_at = Instant::now();
            let outcome = runtime
                .read_truth()
                .execute_query_plan(planned)
                .expect("connectivity traversal outcome");
            let execution_micros = execution_started_at.elapsed().as_micros();
            let elapsed_micros = planning_micros + execution_micros;
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "result_entities": outcome.result.entities.len(),
                    "result_relations": outcome.result.relations.len(),
                    "phase_timing": {
                        "planning_micros": planning_micros,
                        "execution_micros": execution_micros,
                    },
                    "shape_metrics": {
                        "packet_count": outcome.complexity.packet_count,
                        "scope_unit_count": counters.query_scope_unit_count,
                    },
                    "complexity": outcome.complexity,
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "connectivity_traversal_cross_partition",
        &traversal_samples,
        &[
            ("planning_micros", &["phase_timing", "planning_micros"]),
            ("execution_micros", &["phase_timing", "execution_micros"]),
            ("packet_count", &["shape_metrics", "packet_count"]),
            ("scope_unit_count", &["shape_metrics", "scope_unit_count"]),
        ],
    );
    assert!(traversal_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &traversal_samples,
        "connectivity traversal should stay narrow, clone-free, and relation-bounded",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "query_packet_count") <= 3
                && counter_u64(metrics, "query_scope_unit_count") <= 12
                && counter_u64(metrics, "query_authoritative_relation_records_emitted") == 12
                && counter_u64(metrics, "query_packet_peak_width_total") <= 4
                && metrics["result_entities"].as_u64() == Some(24)
                && metrics["result_relations"].as_u64() == Some(12)
        },
    );
}
