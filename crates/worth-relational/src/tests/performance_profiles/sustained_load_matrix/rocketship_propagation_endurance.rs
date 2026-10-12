use super::*;

pub(super) fn certify_rocketship_propagation_endurance(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let rocketship_endurance_node_count = rocketship_node_count();
    let rocketship_propagation_endurance_samples = capture_perf_samples(
        suite,
        "rocketship_propagation_endurance",
        || {
            let query_target_count = rocketship_query_target_count(rocketship_endurance_node_count);
            let mut runtime = runtime_with_test_schema_profile_and_chunks(
                RelationalRuntimeProfile::GeometryKernel,
                ROCKETSHIP_CHUNK_SIZE,
                ROCKETSHIP_CHUNK_SIZE,
            );
            apply_perf_diagnostics_policy(
                &mut runtime,
                PerfDiagnosticsPolicy::GeometryOperationalHotPath,
            );
            runtime.configure_for_test(|config| {
                config.publication.policy.max_patch_records_per_commit =
                    rocketship_endurance_node_count * 2
            });
            let seeded = seed_pseudorealistic_rocketship_world(
                execution,
                &runtime,
                rocketship_endurance_node_count,
                query_target_count,
            );

            const ITERATIONS: usize = 96;
            const WINDOW: usize = 16;
            let mut cycle_samples = Vec::with_capacity(ITERATIONS);
            let mut total_update_micros = 0u128;
            let mut total_propagation_micros = 0u128;
            let mut total_explicit_query_micros = 0u128;
            let mut max_packets_per_iteration = 0usize;
            let mut max_scope_units_per_iteration = 0usize;
            let mut previous_packets = 0usize;
            let mut previous_scope_units = 0usize;

            runtime.performance_access().reset_counters();
            for index in 0..ITERATIONS {
                let target = seeded.traversal_seeds[index % seeded.traversal_seeds.len()];
                let update_started_at = Instant::now();
                let _ = update_entity(
                    execution,
                    &runtime,
                    target,
                    &format!("rocket.endurance.propagation.{index}"),
                );
                let update_micros = update_started_at.elapsed().as_micros();
                total_update_micros += update_micros;

                let snapshot = runtime.visibility_authority().snapshot();
                let context = runtime
                    .read_truth()
                    .query_plan_context(&snapshot)
                    .expect("rocketship endurance propagation context");
                let propagation_seeds = vec![
                    seeded.traversal_seeds[index % seeded.traversal_seeds.len()],
                    seeded.traversal_seeds[(index + 1) % seeded.traversal_seeds.len()],
                    seeded.traversal_seeds[(index + 9) % seeded.traversal_seeds.len()],
                    seeded.traversal_seeds[(index + 10) % seeded.traversal_seeds.len()],
                ];
                let propagation_packet = PlannedQueryPacket {
                    label: "rocketship-endurance-propagation".to_string(),
                    context_id: context,
                    scope: QueryScope::ConnectivityTraversal {
                        seeds: Arc::from(propagation_seeds),
                        relation_kind_scope: Some(Arc::from([KindId(2)])),
                        max_depth: Some(3),
                    },
                    locality: QueryLocalityClass::CrossPartitionTraversal,
                    ordering: QueryOrderingContract::CanonicalTraversalOrder,
                    access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                    execution_shape: QueryExecutionShape::BulkPacketized,
                    reduction: ReductionDiscipline::DeterministicMerge,
                    plan_key: DeterministicQueryPlanKey(92_250),
                    target_count_hint: 4,
                };
                let propagation_started_at = Instant::now();
                let _ = runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, propagation_packet)
                            .expect("planned rocketship endurance propagation query"),
                        execution,
                    )
                    .expect("request remains live")
                    .map(|read| read.outcome)
                    .expect("rocketship endurance propagation outcome");
                let propagation_micros = propagation_started_at.elapsed().as_micros();
                total_propagation_micros += propagation_micros;

                let explicit_targets = seeded
                    .mixed_query_targets
                    .iter()
                    .cycle()
                    .skip(index)
                    .take(12)
                    .cloned()
                    .collect::<Vec<_>>();
                let explicit_packet = explicit_query_packet(
                    &runtime,
                    &snapshot,
                    "rocketship-endurance-explicit-broad",
                    explicit_targets,
                );
                let explicit_started_at = Instant::now();
                let _ = runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, explicit_packet)
                            .expect("planned rocketship endurance explicit broad query"),
                        execution,
                    )
                    .expect("request remains live")
                    .map(|read| read.outcome)
                    .expect("rocketship endurance explicit broad outcome");
                let explicit_query_micros = explicit_started_at.elapsed().as_micros();
                total_explicit_query_micros += explicit_query_micros;
                assert!(runtime
                    .visibility_authority()
                    .release_snapshot(&snapshot)
                    .is_ok());

                let cycle_micros = update_micros + propagation_micros + explicit_query_micros;
                cycle_samples.push(cycle_micros);

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

            let first_window_average_cycle_micros =
                cycle_samples.iter().take(WINDOW).copied().sum::<u128>() / WINDOW as u128;
            let last_window_average_cycle_micros = cycle_samples
                .iter()
                .rev()
                .take(WINDOW)
                .copied()
                .sum::<u128>()
                / WINDOW as u128;
            let elapsed_micros =
                total_update_micros + total_propagation_micros + total_explicit_query_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "iterations": ITERATIONS,
                    "resident_node_count": seeded.entities.len(),
                    "resident_relation_count": seeded.relation_count,
                    "average_update_micros": total_update_micros / ITERATIONS as u128,
                    "average_propagation_micros": total_propagation_micros / ITERATIONS as u128,
                    "average_explicit_query_micros": total_explicit_query_micros / ITERATIONS as u128,
                    "first_window_average_cycle_micros": first_window_average_cycle_micros,
                    "last_window_average_cycle_micros": last_window_average_cycle_micros,
                    "max_packets_per_iteration": max_packets_per_iteration,
                    "max_scope_units_per_iteration": max_scope_units_per_iteration,
                    "counters": runtime.performance_access().counters(),
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "rocketship_propagation_endurance",
        &rocketship_propagation_endurance_samples,
        &[
            ("iterations", &["iterations"]),
            ("resident_node_count", &["resident_node_count"]),
            ("resident_relation_count", &["resident_relation_count"]),
            ("average_update_micros", &["average_update_micros"]),
            (
                "average_propagation_micros",
                &["average_propagation_micros"],
            ),
            (
                "average_explicit_query_micros",
                &["average_explicit_query_micros"],
            ),
            (
                "first_window_average_cycle_micros",
                &["first_window_average_cycle_micros"],
            ),
            (
                "last_window_average_cycle_micros",
                &["last_window_average_cycle_micros"],
            ),
            ("max_packets_per_iteration", &["max_packets_per_iteration"]),
            (
                "max_scope_units_per_iteration",
                &["max_scope_units_per_iteration"],
            ),
        ],
    );
    assert_budget(
        &rocketship_propagation_endurance_samples,
        "rocketship propagation endurance should keep broad-wave cycles bounded across extended 100k-node operation",
        |metrics| {
            let first_window = metrics["first_window_average_cycle_micros"]
                .as_u64()
                .unwrap_or(0);
            let last_window = metrics["last_window_average_cycle_micros"]
                .as_u64()
                .unwrap_or(0);
            metrics["iterations"].as_u64() == Some(96)
                && metrics["resident_node_count"]
                    .as_u64()
                    == Some(rocketship_endurance_node_count as u64)
                && metrics["resident_relation_count"].as_u64().unwrap_or(0)
                    >= rocketship_endurance_node_count as u64
                && last_window <= first_window.saturating_mul(2).max(1)
                && metrics["max_packets_per_iteration"].as_u64().unwrap_or(0) <= 24
                && metrics["max_scope_units_per_iteration"].as_u64().unwrap_or(0) <= 24
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_batch_count") == 96
        },
    );
}
