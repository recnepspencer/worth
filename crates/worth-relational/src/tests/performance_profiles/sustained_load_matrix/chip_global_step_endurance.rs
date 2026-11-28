use super::*;

pub(super) fn certify_chip_global_step_endurance(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let chip_global_step_endurance_samples = capture_counted_samples(|| {
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
        let mut total_update_work = 0u64;
        let mut total_adjacency_work = 0u64;
        let mut max_outgoing_relation_count = 0usize;

        runtime.performance_access().reset_counters();
        for step in 0..ITERATIONS {
            let driver = drivers[step % drivers.len()];
            let (_, update_work) = measure_counted_work(execution, || {
                update_entity(
                    execution,
                    &runtime,
                    driver,
                    &format!("chip-global-driver-step-{step}"),
                )
            });
            total_update_work += update_work;

            let commit = runtime
                .history()
                .latest_commit()
                .expect("chip global step commit")
                .clone();

            let (outgoing_relations, adjacency_work) = measure_counted_work(execution, || {
                runtime
                    .storage_access()
                    .outgoing_relations_for_entity(execution, driver, commit.version_id)
                    .expect("funded adjacency read succeeds")
            });
            total_adjacency_work += adjacency_work;
            max_outgoing_relation_count = max_outgoing_relation_count.max(outgoing_relations.len());

            cycle_samples.push(update_work + adjacency_work);
        }

        let first_window_average_cycle_work =
            cycle_samples.iter().take(WINDOW).copied().sum::<u64>() / WINDOW as u64;
        let last_window_average_cycle_work = cycle_samples
            .iter()
            .rev()
            .take(WINDOW)
            .copied()
            .sum::<u64>()
            / WINDOW as u64;
        let charged_work = total_update_work + total_adjacency_work;
        CountedPerfSample {
            charged_work,
            metrics: perf_metrics!({
                "iterations": ITERATIONS,
                "driver_count": drivers.len(),
                "sink_count": sinks.len(),
                "average_update_work": total_update_work / ITERATIONS as u64,
                "average_adjacency_work": total_adjacency_work / ITERATIONS as u64,
                "first_window_average_cycle_work": first_window_average_cycle_work,
                "last_window_average_cycle_work": last_window_average_cycle_work,
                "max_outgoing_relation_count": max_outgoing_relation_count,
                "counters": runtime.performance_access().counters(),
            }),
        }
    });
    assert!(chip_global_step_endurance_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &chip_global_step_endurance_samples,
        "chip global step endurance should keep repeated denser fanout stepping proportional across a longer sustained window",
        |metrics| {
            let first_window = metrics["first_window_average_cycle_work"]
                .as_u64()
                .unwrap_or(0);
            let last_window = metrics["last_window_average_cycle_work"]
                .as_u64()
                .unwrap_or(0);
            metrics["iterations"].as_u64() == Some(128)
                && metrics["driver_count"].as_u64() == Some(8)
                && metrics["sink_count"].as_u64() == Some(64)
                && metrics["max_outgoing_relation_count"].as_u64().unwrap_or(0) >= 8
                && first_window > 0
                && last_window <= first_window.saturating_mul(2)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_entity_target_count") == 128
        },
    );
}
