use super::*;

pub(super) fn certify_mixed_topology_query_churn_stability(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let mixed_topology_churn_samples = capture_perf_samples(
        suite,
        "mixed_topology_query_churn_stability",
        || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            let entities = (0..24)
                .map(|index| {
                    create_entity_in_partition(
                        execution,
                        &runtime,
                        &format!("mixed-topology-{index}"),
                        PartitionId((index % 6) as u32 + 1),
                    )
                })
                .collect::<Vec<_>>();
            for index in 0..24 {
                create_relation_in_partition(
                    execution,
                    &runtime,
                    entities[index],
                    entities[(index + 1) % 24],
                    &format!("mixed-ring-{index}"),
                    PartitionId(40 + (index % 4) as u32),
                );
                if index % 4 == 0 {
                    create_relation_in_partition(
                        execution,
                        &runtime,
                        entities[index],
                        entities[(index + 6) % 24],
                        &format!("mixed-brace-{index}"),
                        PartitionId(50 + (index % 3) as u32),
                    );
                }
            }

            const ITERATIONS: usize = 48;
            let mut total_update_micros = 0u128;
            let mut total_explicit_query_micros = 0u128;
            let mut total_traversal_micros = 0u128;
            let mut max_packets_per_iteration = 0usize;
            let mut max_scope_units_per_iteration = 0usize;
            let mut previous_packets = 0usize;
            let mut previous_scope_units = 0usize;

            runtime.performance_access().reset_counters();
            for index in 0..ITERATIONS {
                let hot_entity = entities[(index * 3) % entities.len()];
                let update_started_at = Instant::now();
                let _ = update_entity(
                    execution,
                    &runtime,
                    hot_entity,
                    &format!("mixed-topology-hot-{index}"),
                );
                total_update_micros += update_started_at.elapsed().as_micros();

                let snapshot = runtime.visibility_authority().snapshot();
                let explicit_targets = vec![
                    RecordRef::Entity(entities[(index * 3) % entities.len()]),
                    RecordRef::Entity(entities[(index * 3 + 1) % entities.len()]),
                    RecordRef::Entity(entities[(index * 3 + 6) % entities.len()]),
                    RecordRef::Entity(entities[(index * 3 + 12) % entities.len()]),
                ];
                let explicit_packet = explicit_query_packet(
                    &runtime,
                    &snapshot,
                    "mixed-topology-explicit",
                    explicit_targets,
                );
                let explicit_started_at = Instant::now();
                let _ = runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, explicit_packet)
                            .expect("planned mixed topology explicit query"),
                        execution,
                    )
                    .expect("request remains live")
                    .map(|read| read.outcome)
                    .expect("mixed topology explicit query outcome");
                total_explicit_query_micros += explicit_started_at.elapsed().as_micros();

                let context = runtime
                    .read_truth()
                    .query_plan_context(&snapshot)
                    .expect("mixed topology query plan context");
                let traversal_packet = PlannedQueryPacket {
                    label: "mixed-topology-traversal".to_string(),
                    context_id: context,
                    scope: QueryScope::ConnectivityTraversal {
                        seeds: Arc::from([
                            entities[(index * 3) % entities.len()],
                            entities[(index * 3 + 6) % entities.len()],
                        ]),
                        relation_kind_scope: Some(Arc::from([KindId(2)])),
                        max_depth: Some(2),
                    },
                    locality: QueryLocalityClass::CrossPartitionTraversal,
                    ordering: QueryOrderingContract::CanonicalTraversalOrder,
                    access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                    execution_shape: QueryExecutionShape::BulkPacketized,
                    reduction: ReductionDiscipline::DeterministicMerge,
                    plan_key: DeterministicQueryPlanKey(92_001),
                    target_count_hint: 2,
                };
                let traversal_started_at = Instant::now();
                let _ = runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, traversal_packet)
                            .expect("planned mixed topology traversal query"),
                        execution,
                    )
                    .expect("request remains live")
                    .map(|read| read.outcome)
                    .expect("mixed topology traversal outcome");
                total_traversal_micros += traversal_started_at.elapsed().as_micros();

                let counters = runtime.performance_access().counters();
                max_packets_per_iteration = max_packets_per_iteration
                    .max(counters.query_packet_count.saturating_sub(previous_packets));
                max_scope_units_per_iteration = max_scope_units_per_iteration.max(
                    counters
                        .query_scope_unit_count
                        .saturating_sub(previous_scope_units),
                );
                previous_packets = counters.query_packet_count;
                previous_scope_units = counters.query_scope_unit_count;
            }

            let elapsed_micros =
                total_update_micros + total_explicit_query_micros + total_traversal_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "iterations": ITERATIONS,
                    "average_update_micros": total_update_micros / ITERATIONS as u128,
                    "average_explicit_query_micros": total_explicit_query_micros / ITERATIONS as u128,
                    "average_traversal_micros": total_traversal_micros / ITERATIONS as u128,
                    "max_packets_per_iteration": max_packets_per_iteration,
                    "max_scope_units_per_iteration": max_scope_units_per_iteration,
                    "counters": runtime.performance_access().counters(),
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "mixed_topology_query_churn_stability",
        &mixed_topology_churn_samples,
        &[
            ("average_update_micros", &["average_update_micros"]),
            (
                "average_explicit_query_micros",
                &["average_explicit_query_micros"],
            ),
            ("average_traversal_micros", &["average_traversal_micros"]),
            ("max_packets_per_iteration", &["max_packets_per_iteration"]),
            (
                "max_scope_units_per_iteration",
                &["max_scope_units_per_iteration"],
            ),
        ],
    );
    assert!(mixed_topology_churn_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &mixed_topology_churn_samples,
        "mixed sustained topology churn should keep packet and scope growth bounded across repeated update plus read waves",
        |metrics| {
            metrics["iterations"].as_u64() == Some(48)
                && metrics["max_packets_per_iteration"].as_u64().unwrap_or(0) <= 8
                && metrics["max_scope_units_per_iteration"].as_u64().unwrap_or(0) <= 8
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "query_packet_count") >= 96
                && counter_u64(metrics, "query_scope_unit_count") >= 96
        },
    );
}
