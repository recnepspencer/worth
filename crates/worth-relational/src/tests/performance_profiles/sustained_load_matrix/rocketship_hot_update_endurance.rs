use super::*;

pub(super) fn certify_rocketship_hot_update_endurance(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let rocketship_endurance_node_count = rocketship_node_count();
    let rocketship_hot_update_endurance_samples = capture_counted_samples(|| {
        let query_target_count = rocketship_query_target_count(rocketship_endurance_node_count);
        let mut runtime = runtime_with_extra_entity_aspects_profile_and_chunks(
            RelationalRuntimeProfile::GeometryKernel,
            ROCKETSHIP_CHUNK_SIZE,
            ROCKETSHIP_CHUNK_SIZE,
            vec![
                crate::tests::support::entity_field_aspect(
                    crate::tests::support::aspect_key("section"),
                    crate::tests::support::field_key("section"),
                ),
                crate::tests::support::entity_field_aspect(
                    crate::tests::support::aspect_key("subsystem"),
                    crate::tests::support::field_key("subsystem"),
                ),
                crate::tests::support::entity_field_aspect(
                    crate::tests::support::aspect_key("aspect"),
                    crate::tests::support::field_key("aspect"),
                ),
                crate::tests::support::entity_u64_field_aspect(
                    crate::tests::support::aspect_key("ordinal"),
                    crate::tests::support::field_key("ordinal"),
                ),
            ],
        );
        apply_perf_diagnostics_policy(
            &mut runtime,
            PerfDiagnosticsPolicy::GeometryOperationalHotPath,
        );
        let seeded = seed_pseudorealistic_rocketship_world(
            execution,
            &runtime,
            rocketship_endurance_node_count,
            query_target_count,
        );

        const ITERATIONS: usize = 256;
        const WINDOW: usize = 32;
        let mut update_samples = Vec::with_capacity(ITERATIONS);
        let mut total_update_work = 0u64;
        let mut max_update_work = 0u64;
        let mut max_query_work = 0u64;

        runtime.performance_access().reset_counters();
        for index in 0..ITERATIONS {
            let target = seeded.traversal_seeds[index % seeded.traversal_seeds.len()];
            let (_, update_work) = measure_counted_work(execution, || {
                update_entity(
                    execution,
                    &runtime,
                    target,
                    &format!("rocket.endurance.hot-loop.{index}"),
                )
            });
            update_samples.push(update_work);
            total_update_work += update_work;
            max_update_work = max_update_work.max(update_work);

            if index % 16 == 0 {
                let snapshot = runtime
                    .visibility_authority()
                    .snapshot()
                    .expect("current snapshot fixture is admitted");
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
                let (_, query_work) = measure_counted_work(execution, || {
                    runtime
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
                        .expect("rocketship endurance explicit query outcome")
                });
                max_query_work = max_query_work.max(query_work);
                assert!(runtime
                    .visibility_authority()
                    .release_snapshot(&snapshot)
                    .is_ok());
            }
        }

        let first_window_average_update_work =
            update_samples.iter().take(WINDOW).copied().sum::<u64>() / WINDOW as u64;
        let last_window_average_update_work = update_samples
            .iter()
            .rev()
            .take(WINDOW)
            .copied()
            .sum::<u64>()
            / WINDOW as u64;
        CountedPerfSample {
            charged_work: total_update_work,
            metrics: perf_metrics!({
                "iterations": ITERATIONS,
                "resident_node_count": seeded.entities.len(),
                "resident_relation_count": seeded.relation_count,
                "average_update_work": total_update_work / ITERATIONS as u64,
                "max_update_work": max_update_work,
                "first_window_average_update_work": first_window_average_update_work,
                "last_window_average_update_work": last_window_average_update_work,
                "max_explicit_query_work": max_query_work,
                "counters": runtime.performance_access().counters(),
            }),
        }
    });
    assert!(rocketship_hot_update_endurance_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &rocketship_hot_update_endurance_samples,
        "rocketship hot endurance should stay region-local and resist drift across long update windows",
        |metrics| {
            let first_window = metrics["first_window_average_update_work"]
                .as_u64()
                .unwrap_or(0);
            let last_window = metrics["last_window_average_update_work"]
                .as_u64()
                .unwrap_or(0);
            metrics["iterations"].as_u64() == Some(256)
                && metrics["resident_node_count"].as_u64()
                    == Some(rocketship_endurance_node_count as u64)
                && metrics["resident_relation_count"].as_u64().unwrap_or(0)
                    >= rocketship_endurance_node_count as u64
                && first_window > 0
                && last_window <= first_window.saturating_mul(2)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_batch_count") == 256
                && counter_u64(metrics, "partitions_cloned") <= 256
        },
    );
}
