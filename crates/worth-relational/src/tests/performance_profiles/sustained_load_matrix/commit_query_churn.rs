use super::*;

pub(super) fn certify_commit_query_churn_stability(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let commit_query_churn_samples =
        capture_perf_samples(suite, "commit_query_churn_stability", || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore);
            const ITERATIONS: usize = 128;
            let mut total_commit_micros = 0u128;
            let mut total_query_micros = 0u128;
            let mut max_query_packets_per_iteration = 0usize;
            let mut max_query_scope_units_per_iteration = 0usize;
            let mut previous_scope_units = 0usize;

            runtime.performance_access().reset_counters();
            for index in 0..ITERATIONS {
                let commit_started_at = Instant::now();
                let outcome = create_entity_outcome(&runtime, &format!("sustained-{index}"));
                total_commit_micros += commit_started_at.elapsed().as_micros();

                let entity = changed_entities(&outcome)[0];
                let snapshot = runtime.visibility_authority().snapshot();
                let packet = explicit_query_packet(
                    &runtime,
                    &snapshot,
                    "sustained-explicit-target",
                    vec![RecordRef::Entity(entity)],
                );
                let query_started_at = Instant::now();
                let query_outcome = runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, packet)
                            .expect("planned sustained explicit query"),
                        execution,
                    )
                    .expect("request remains live")
                    .map(|read| read.outcome)
                    .expect("sustained explicit query outcome");
                total_query_micros += query_started_at.elapsed().as_micros();
                max_query_packets_per_iteration =
                    max_query_packets_per_iteration.max(query_outcome.complexity.packet_count);
                let scope_units = runtime
                    .performance_access()
                    .counters()
                    .query_scope_unit_count;
                max_query_scope_units_per_iteration = max_query_scope_units_per_iteration
                    .max(scope_units.saturating_sub(previous_scope_units));
                previous_scope_units = scope_units;
            }

            let latest_version = runtime
                .history()
                .latest_commit()
                .expect("latest sustained commit")
                .version_id;
            let final_entity_count = runtime
                .read_truth()
                .project_historical_version(execution, latest_version)
                .all_authoritative_entity_records(execution)
                .len();
            let counters = runtime.performance_access().counters();

            let elapsed_micros = total_commit_micros + total_query_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "iterations": ITERATIONS,
                    "average_commit_micros": total_commit_micros / ITERATIONS as u128,
                    "average_query_micros": total_query_micros / ITERATIONS as u128,
                    "max_query_packets_per_iteration": max_query_packets_per_iteration,
                    "max_query_scope_units_per_iteration": max_query_scope_units_per_iteration,
                    "final_entity_count": final_entity_count,
                    "counters": counters,
                })
            })
        });
    emit_metric_summaries(
        suite,
        "commit_query_churn_stability",
        &commit_query_churn_samples,
        &[
            ("average_commit_micros", &["average_commit_micros"]),
            ("average_query_micros", &["average_query_micros"]),
            (
                "max_query_packets_per_iteration",
                &["max_query_packets_per_iteration"],
            ),
            (
                "max_query_scope_units_per_iteration",
                &["max_query_scope_units_per_iteration"],
            ),
            ("final_entity_count", &["final_entity_count"]),
        ],
    );
    assert!(commit_query_churn_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &commit_query_churn_samples,
        "sustained commit/query churn should stay clone-free and packet-stable across long iteration windows",
        |metrics| {
            metrics["iterations"].as_u64() == Some(128)
                && metrics["final_entity_count"].as_u64() == Some(128)
                && metrics["max_query_packets_per_iteration"].as_u64() == Some(1)
                && metrics["max_query_scope_units_per_iteration"].as_u64() == Some(1)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "query_packet_count") == 128
                && counter_u64(metrics, "query_scope_unit_count") == 128
        },
    );
}
