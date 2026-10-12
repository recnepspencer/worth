use super::*;

pub(super) fn certify_rocketship_hot_update_endurance(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let rocketship_endurance_node_count = rocketship_node_count();
    let rocketship_hot_update_endurance_samples =
        capture_perf_samples(suite, "rocketship_hot_update_endurance", || {
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

            const ITERATIONS: usize = 256;
            const WINDOW: usize = 32;
            let mut update_samples = Vec::with_capacity(ITERATIONS);
            let mut total_update_micros = 0u128;
            let mut max_update_micros = 0u128;
            let mut max_query_micros = 0u128;

            runtime.performance_access().reset_counters();
            for index in 0..ITERATIONS {
                let target = seeded.traversal_seeds[index % seeded.traversal_seeds.len()];
                let update_started_at = Instant::now();
                let _ = update_entity(
                    execution,
                    &runtime,
                    target,
                    &format!("rocket.endurance.hot-loop.{index}"),
                );
                let update_micros = update_started_at.elapsed().as_micros();
                update_samples.push(update_micros);
                total_update_micros += update_micros;
                max_update_micros = max_update_micros.max(update_micros);

                if index % 16 == 0 {
                    let snapshot = runtime.visibility_authority().snapshot();
                    let explicit_targets = seeded
                        .mixed_query_targets
                        .iter()
                        .skip(index % seeded.mixed_query_targets.len())
                        .take(8)
                        .cloned()
                        .collect::<Vec<_>>();
                    let packet = explicit_query_packet(
                        &runtime,
                        &snapshot,
                        "rocketship-endurance-explicit",
                        explicit_targets,
                    );
                    let query_started_at = Instant::now();
                    let _ = runtime
                        .read_truth()
                        .execute_query_plan(
                            runtime
                                .read_truth()
                                .plan_query_packet(&snapshot, packet)
                                .expect("planned rocketship endurance explicit query"),
                            execution,
                        )
                        .expect("request remains live")
                        .map(|read| read.outcome)
                        .expect("rocketship endurance explicit query outcome");
                    max_query_micros = max_query_micros.max(query_started_at.elapsed().as_micros());
                    assert!(runtime
                        .visibility_authority()
                        .release_snapshot(&snapshot)
                        .is_ok());
                }
            }

            let first_window_average_update_micros =
                update_samples.iter().take(WINDOW).copied().sum::<u128>() / WINDOW as u128;
            let last_window_average_update_micros = update_samples
                .iter()
                .rev()
                .take(WINDOW)
                .copied()
                .sum::<u128>()
                / WINDOW as u128;
            measurement_with_elapsed(total_update_micros, || {
                perf_metrics!({
                    "iterations": ITERATIONS,
                    "resident_node_count": seeded.entities.len(),
                    "resident_relation_count": seeded.relation_count,
                    "average_update_micros": total_update_micros / ITERATIONS as u128,
                    "max_update_micros": max_update_micros,
                    "first_window_average_update_micros": first_window_average_update_micros,
                    "last_window_average_update_micros": last_window_average_update_micros,
                    "max_explicit_query_micros": max_query_micros,
                    "counters": runtime.performance_access().counters(),
                })
            })
        });
    emit_metric_summaries(
        suite,
        "rocketship_hot_update_endurance",
        &rocketship_hot_update_endurance_samples,
        &[
            ("iterations", &["iterations"]),
            ("resident_node_count", &["resident_node_count"]),
            ("resident_relation_count", &["resident_relation_count"]),
            ("average_update_micros", &["average_update_micros"]),
            ("max_update_micros", &["max_update_micros"]),
            (
                "first_window_average_update_micros",
                &["first_window_average_update_micros"],
            ),
            (
                "last_window_average_update_micros",
                &["last_window_average_update_micros"],
            ),
            ("max_explicit_query_micros", &["max_explicit_query_micros"]),
        ],
    );
    assert_budget(
        &rocketship_hot_update_endurance_samples,
        "rocketship hot endurance should stay region-local and resist drift across long update windows",
        |metrics| {
            let first_window = metrics["first_window_average_update_micros"]
                .as_u64()
                .unwrap_or(0);
            let last_window = metrics["last_window_average_update_micros"]
                .as_u64()
                .unwrap_or(0);
            metrics["iterations"].as_u64() == Some(256)
                && metrics["resident_node_count"]
                    .as_u64()
                    == Some(rocketship_endurance_node_count as u64)
                && metrics["resident_relation_count"].as_u64().unwrap_or(0)
                    >= rocketship_endurance_node_count as u64
                && last_window <= first_window.saturating_mul(2).max(1)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_batch_count") == 256
                && counter_u64(metrics, "partitions_cloned") <= 256
        },
    );
}
