use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::basis_reads::VisibilityProjectionViewFixturePort as _;
use super::*;

pub(super) fn certify_commit_query_churn_stability(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let commit_query_churn_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore);
        const ITERATIONS: usize = 128;
        let mut total_commit_work = 0u64;
        let mut total_query_work = 0u64;
        let mut max_query_packets_per_iteration = 0usize;
        let mut max_query_scope_units_per_iteration = 0usize;
        let mut previous_scope_units = 0usize;

        runtime.performance_access().reset_counters();
        for index in 0..ITERATIONS {
            let (outcome, commit_work) = measure_counted_work(execution, || {
                create_entity_outcome(execution, &runtime, &format!("sustained-{index}"))
            });
            total_commit_work += commit_work;

            let entity = changed_entities(&outcome)[0];
            let snapshot = runtime
                .visibility_authority()
                .snapshot()
                .expect("current snapshot fixture is admitted");
            let packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "sustained-explicit-target",
                vec![RecordRef::Entity(entity)],
            );
            let (query_outcome, query_work) = measure_counted_work(execution, || {
                runtime
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
                    .expect("sustained explicit query outcome")
            });
            total_query_work += query_work;
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
            .try_project_historical_version(latest_version)
            .unwrap()
            .all_authoritative_entity_records_for_test(execution)
            .expect("read request remains live")
            .len();
        let counters = runtime.performance_access().counters();

        CountedPerfSample {
            charged_work: total_commit_work + total_query_work,
            metrics: perf_metrics!({
                "iterations": ITERATIONS,
                "average_commit_work": total_commit_work / ITERATIONS as u64,
                "average_query_work": total_query_work / ITERATIONS as u64,
                "max_query_packets_per_iteration": max_query_packets_per_iteration,
                "max_query_scope_units_per_iteration": max_query_scope_units_per_iteration,
                "final_entity_count": final_entity_count,
                "counters": counters,
            }),
        }
    });
    assert!(commit_query_churn_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
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
