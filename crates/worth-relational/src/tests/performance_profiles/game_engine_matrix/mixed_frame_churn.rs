use super::*;

pub(super) fn certify_mixed_read_write_frame_churn_window(suite: &'static str) {
    let mixed_frame_churn_samples = capture_perf_samples(
        suite,
        "mixed_read_write_frame_churn_window",
        || {
            let mut runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore);
            apply_perf_diagnostics_policy(
                &mut runtime,
                PerfDiagnosticsPolicy::GeometryOperationalHotPath,
            );
            let seeded = seed_game_engine_frame_world(&runtime, "scene-frame", 8, 24);
            let mut downstream_runtime = build_mock_downstream_runtime(false, 48);

            const ITERATIONS: usize = 48;
            const WINDOW: usize = 12;
            let mut cycle_samples = Vec::with_capacity(ITERATIONS);
            let mut total_update_micros = 0u128;
            let mut total_propagation_micros = 0u128;
            let mut total_explicit_query_micros = 0u128;
            let mut total_downstream_micros = 0u128;
            let mut max_packets_per_iteration = 0usize;
            let mut max_scope_units_per_iteration = 0usize;
            let mut max_downstream_tasks_scheduled = 0u64;
            let mut previous_packets = 0usize;
            let mut previous_scope_units = 0usize;

            runtime.performance_access().reset_counters();
            for frame in 0..ITERATIONS {
                let actor = seeded.frame_targets[frame % seeded.frame_targets.len()];
                let update_started_at = Instant::now();
                let _ = update_entity(&runtime, actor, &format!("scene-frame-step-{frame}"));
                let update_micros = update_started_at.elapsed().as_micros();
                total_update_micros += update_micros;

                let snapshot = runtime.visibility_authority().snapshot();
                let propagation_packet = PlannedQueryPacket {
                    label: "scene-frame-propagation".to_string(),
                    context_id: runtime
                        .read_truth()
                        .query_plan_context(&snapshot)
                        .expect("scene frame query context"),
                    scope: QueryScope::ConnectivityTraversal {
                        seeds: Arc::from([
                            seeded.propagation_seeds[frame % seeded.propagation_seeds.len()],
                            seeded.propagation_seeds[(frame + 1) % seeded.propagation_seeds.len()],
                        ]),
                        relation_kind_scope: Some(Arc::from([KindId(2)])),
                        max_depth: Some(2),
                    },
                    locality: QueryLocalityClass::CrossPartitionTraversal,
                    ordering: QueryOrderingContract::CanonicalTraversalOrder,
                    access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                    execution_shape: QueryExecutionShape::BulkPacketized,
                    reduction: ReductionDiscipline::DeterministicMerge,
                    plan_key: DeterministicQueryPlanKey(93_101),
                    target_count_hint: 2,
                };
                let propagation_started_at = Instant::now();
                let propagation = runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, propagation_packet)
                            .expect("scene frame propagation plan"),
                    )
                    .expect("scene frame propagation outcome");
                let propagation_micros = propagation_started_at.elapsed().as_micros();
                total_propagation_micros += propagation_micros;

                let explicit_targets = seeded
                    .explicit_targets
                    .iter()
                    .cycle()
                    .skip(frame)
                    .take(8)
                    .map(|entity| RecordRef::Entity(*entity))
                    .collect::<Vec<_>>();
                let explicit_packet = explicit_query_packet(
                    &runtime,
                    &snapshot,
                    "scene-frame-explicit",
                    explicit_targets,
                );
                let explicit_started_at = Instant::now();
                let explicit = runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, explicit_packet)
                            .expect("scene frame explicit plan"),
                    )
                    .expect("scene frame explicit outcome");
                let explicit_micros = explicit_started_at.elapsed().as_micros();
                total_explicit_query_micros += explicit_micros;
                assert!(runtime
                    .visibility_authority()
                    .release_snapshot(&snapshot)
                    .is_ok());

                let affected_sources = (propagation.result.entities.len()
                    + explicit.result.entities.len())
                .min(downstream_runtime.source_versions.len())
                .max(4);
                let downstream_before = downstream_runtime.observe();
                let downstream_started_at = Instant::now();
                downstream_runtime.apply_changes(affected_sources);
                let downstream_micros = downstream_started_at.elapsed().as_micros();
                total_downstream_micros += downstream_micros;
                let downstream_after = downstream_runtime.observe();
                max_downstream_tasks_scheduled = max_downstream_tasks_scheduled.max(
                    downstream_after.planner.tasks_scheduled
                        - downstream_before.planner.tasks_scheduled,
                );

                cycle_samples
                    .push(update_micros + propagation_micros + explicit_micros + downstream_micros);
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
            let elapsed_micros = total_update_micros
                + total_propagation_micros
                + total_explicit_query_micros
                + total_downstream_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "iterations": ITERATIONS,
                    "region_count": seeded.region_count,
                    "resident_entities": seeded.entities.len(),
                    "resident_relations": seeded.relation_count,
                    "average_update_micros": total_update_micros / ITERATIONS as u128,
                    "average_propagation_micros": total_propagation_micros / ITERATIONS as u128,
                    "average_explicit_query_micros": total_explicit_query_micros / ITERATIONS as u128,
                    "average_downstream_micros": total_downstream_micros / ITERATIONS as u128,
                    "first_window_average_cycle_micros": first_window_average_cycle_micros,
                    "last_window_average_cycle_micros": last_window_average_cycle_micros,
                    "max_packets_per_iteration": max_packets_per_iteration,
                    "max_scope_units_per_iteration": max_scope_units_per_iteration,
                    "max_downstream_tasks_scheduled": max_downstream_tasks_scheduled,
                    "counters": runtime.performance_access().counters(),
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "mixed_read_write_frame_churn_window",
        &mixed_frame_churn_samples,
        &[
            ("iterations", &["iterations"]),
            ("average_update_micros", &["average_update_micros"]),
            (
                "average_propagation_micros",
                &["average_propagation_micros"],
            ),
            (
                "average_explicit_query_micros",
                &["average_explicit_query_micros"],
            ),
            ("average_downstream_micros", &["average_downstream_micros"]),
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
            (
                "max_downstream_tasks_scheduled",
                &["max_downstream_tasks_scheduled"],
            ),
        ],
    );
    assert_budget(
        &mixed_frame_churn_samples,
        "game-engine frame churn should keep repeated mixed read/write cycles local and stable across a bounded frame window",
        |metrics| {
            let first_window = metrics["first_window_average_cycle_micros"]
                .as_u64()
                .unwrap_or(0);
            let last_window = metrics["last_window_average_cycle_micros"]
                .as_u64()
                .unwrap_or(0);
            metrics["iterations"].as_u64() == Some(48)
                && metrics["region_count"].as_u64() == Some(8)
                && metrics["resident_entities"].as_u64() == Some(192)
                && metrics["max_packets_per_iteration"].as_u64().unwrap_or(0) <= 16
                && metrics["max_scope_units_per_iteration"].as_u64().unwrap_or(0) <= 16
                && metrics["max_downstream_tasks_scheduled"].as_u64().unwrap_or(0) <= 64
                && last_window <= first_window.saturating_mul(2).max(1)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_batch_count") == 48
        },
    );
}
