use super::*;

pub(super) fn certify_merge_planning_divergent_update(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let merge_planning_samples =
        capture_perf_samples(suite, "merge_planning_divergent_update", || {
            let runtime = durable_runtime_with_test_schema();
            let shared = create_entity(execution, &runtime, "shared");
            create_branch_from_main(&runtime, "feature");
            let _ = update_entity(execution, &runtime, shared, "main-value");
            let _ = update_entity_on_branch(
                execution,
                &runtime,
                shared,
                "feature-value",
                BranchId("feature".to_string()),
            );

            runtime.performance_access().reset_counters();
            let started_at = Instant::now();
            let artifact = runtime
                .merge()
                .inspect_planning_scope(
                    execution,
                    crate::merge::data::MergePlanningRequest::new(
                        BranchId("main".to_string()),
                        BranchId("feature".to_string()),
                        MergeIntent::ReconcileIntoTarget,
                    ),
                )
                .expect("merge planning artifact");
            let elapsed_micros = started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "candidate_count": artifact.identity_discovery.candidate_count,
                    "classified_records": artifact.conflict_classification.classified_record_count,
                    "resolved_records": artifact.policy_resolution.resolved_record_count,
                    "decision_count": artifact.decision_log.decisions.len(),
                    "counters": counters,
                }),
            }
        });
    assert!(merge_planning_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &merge_planning_samples,
        "merge planning should stay request-shaped and artifact-accounted",
        |metrics| {
            counter_u64(metrics, "merge_planning_requests") == 1
                && counter_u64(metrics, "merge_identity_candidates_discovered")
                    == metrics["candidate_count"].as_u64().unwrap_or(0)
                && counter_u64(metrics, "merge_conflict_records_classified")
                    == metrics["classified_records"].as_u64().unwrap_or(0)
                && counter_u64(metrics, "merge_decision_log_width")
                    == metrics["decision_count"].as_u64().unwrap_or(0)
        },
    );
}
