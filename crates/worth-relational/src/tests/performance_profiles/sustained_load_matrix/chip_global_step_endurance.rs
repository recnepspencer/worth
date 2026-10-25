use super::*;

pub(super) fn certify_chip_global_step_endurance(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let chip_global_step_endurance_samples =
        capture_perf_samples(suite, "chip_global_step_endurance", || {
            let mut runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
            apply_perf_diagnostics_policy(
                &mut runtime,
                PerfDiagnosticsPolicy::ChipOperationalHotPath,
            );

            let drivers = (0..8)
                .map(|index| {
                    create_entity_in_partition(
                        execution,
                        &runtime,
                        &format!("chip-global-driver-{index}"),
                        PartitionId(930 + index as u32),
                    )
                })
                .collect::<Vec<_>>();
            let sinks = (0..64)
                .map(|index| {
                    create_entity_in_partition(
                        execution,
                        &runtime,
                        &format!("chip-global-sink-{index}"),
                        PartitionId(950 + (index % 8) as u32),
                    )
                })
                .collect::<Vec<_>>();

            for (index, driver) in drivers.iter().enumerate() {
                for fanout in 0..8 {
                    let sink = sinks[index * 8 + fanout];
                    create_relation_in_partition(
                        execution,
                        &runtime,
                        *driver,
                        sink,
                        &format!("chip-global-fanout-{index}-{fanout}"),
                        PartitionId(980 + index as u32),
                    );
                }
            }
            for index in 0..(sinks.len() - 1) {
                create_relation_in_partition(
                    execution,
                    &runtime,
                    sinks[index],
                    sinks[index + 1],
                    &format!("chip-global-chain-{index}"),
                    PartitionId(990 + (index % 4) as u32),
                );
            }

            const ITERATIONS: usize = 128;
            const WINDOW: usize = 32;
            let mut cycle_samples = Vec::with_capacity(ITERATIONS);
            let mut total_update_micros = 0u128;
            let mut total_compile_micros = 0u128;
            let mut total_adjacency_micros = 0u128;
            let mut max_compile_micros = 0u128;
            let mut max_outgoing_relation_count = 0usize;

            runtime.performance_access().reset_counters();
            for step in 0..ITERATIONS {
                let driver = drivers[step % drivers.len()];
                let update_started_at = Instant::now();
                let _ = update_entity(
                    execution,
                    &runtime,
                    driver,
                    &format!("chip-global-driver-step-{step}"),
                );
                let update_micros = update_started_at.elapsed().as_micros();
                total_update_micros += update_micros;

                let commit = runtime
                    .history()
                    .latest_commit()
                    .expect("chip global step commit")
                    .clone();
                let compile_started_at = Instant::now();
                let artifact = runtime
                    .compiled_artifacts_authority()
                    .compile_execution_artifact(
                        execution,
                        commit.commit_id,
                        vec![
                            PartitionId(930),
                            PartitionId(931),
                            PartitionId(932),
                            PartitionId(933),
                            PartitionId(934),
                            PartitionId(935),
                            PartitionId(936),
                            PartitionId(937),
                            PartitionId(950),
                            PartitionId(951),
                            PartitionId(952),
                            PartitionId(953),
                            PartitionId(954),
                            PartitionId(955),
                            PartitionId(956),
                            PartitionId(957),
                        ],
                    )
                    .expect("chip global step compiled artifact");
                let compile_micros = compile_started_at.elapsed().as_micros();
                total_compile_micros += compile_micros;
                max_compile_micros = max_compile_micros.max(compile_micros);

                let adjacency_started_at = Instant::now();
                let outgoing_relations = runtime
                    .storage_access()
                    .outgoing_relations_for_entity(driver, commit.version_id);
                let adjacency_micros = adjacency_started_at.elapsed().as_micros();
                total_adjacency_micros += adjacency_micros;
                max_outgoing_relation_count =
                    max_outgoing_relation_count.max(outgoing_relations.len());
                assert_eq!(
                    runtime
                        .compiled_artifacts()
                        .compiled_artifact_authority_status(artifact.artifact_id),
                    CompiledArtifactAuthorityStatus::Authoritative
                );

                cycle_samples.push(update_micros + compile_micros + adjacency_micros);
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
                total_update_micros + total_compile_micros + total_adjacency_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "iterations": ITERATIONS,
                    "driver_count": drivers.len(),
                    "sink_count": sinks.len(),
                    "average_update_micros": total_update_micros / ITERATIONS as u128,
                    "average_compile_micros": total_compile_micros / ITERATIONS as u128,
                    "average_adjacency_micros": total_adjacency_micros / ITERATIONS as u128,
                    "first_window_average_cycle_micros": first_window_average_cycle_micros,
                    "last_window_average_cycle_micros": last_window_average_cycle_micros,
                    "max_compile_micros": max_compile_micros,
                    "max_outgoing_relation_count": max_outgoing_relation_count,
                    "counters": runtime.performance_access().counters(),
                })
            })
        });
    emit_metric_summaries(
        suite,
        "chip_global_step_endurance",
        &chip_global_step_endurance_samples,
        &[
            ("iterations", &["iterations"]),
            ("average_update_micros", &["average_update_micros"]),
            ("average_compile_micros", &["average_compile_micros"]),
            ("average_adjacency_micros", &["average_adjacency_micros"]),
            (
                "first_window_average_cycle_micros",
                &["first_window_average_cycle_micros"],
            ),
            (
                "last_window_average_cycle_micros",
                &["last_window_average_cycle_micros"],
            ),
            ("max_compile_micros", &["max_compile_micros"]),
            (
                "max_outgoing_relation_count",
                &["max_outgoing_relation_count"],
            ),
        ],
    );
    assert_budget(
        &chip_global_step_endurance_samples,
        "chip global step endurance should keep repeated denser fanout stepping proportional across a longer sustained window",
        |metrics| {
            let first_window = metrics["first_window_average_cycle_micros"]
                .as_u64()
                .unwrap_or(0);
            let last_window = metrics["last_window_average_cycle_micros"]
                .as_u64()
                .unwrap_or(0);
            metrics["iterations"].as_u64() == Some(128)
                && metrics["driver_count"].as_u64() == Some(8)
                && metrics["sink_count"].as_u64() == Some(64)
                && metrics["max_outgoing_relation_count"].as_u64().unwrap_or(0) >= 8
                && last_window <= first_window.saturating_mul(2).max(1)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_entity_target_count") == 128
        },
    );
}
