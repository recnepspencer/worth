use super::*;

pub(super) fn certify_merge_planning_divergent_update(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let merge_planning_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_for_medium(&medium);
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
        let (artifact, charged_work) = measure_counted_work(execution, || {
            runtime
                .merge()
                .inspect_planning_scope(
                    execution,
                    crate::merge::data::MergePlanningRequest::new(
                        BranchId("main".to_string()),
                        BranchId("feature".to_string()),
                        MergeIntent::ReconcileIntoTarget,
                    ),
                )
                .expect("merge planning artifact")
        });
        let counters = runtime.performance_access().counters();

        CountedPerfSample {
            charged_work,
            metrics: perf_metrics!({
                "charged_work": charged_work,
                "candidate_count": artifact.identity_discovery.candidate_count,
                "classified_records": artifact.conflict_classification.classified_record_count,
                "resolved_records": artifact.policy_resolution.resolved_record_count,
                "decision_count": artifact.decision_log.decisions.len(),
                "counters": counters,
            }),
        }
    });
    assert_counted_budget(
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
