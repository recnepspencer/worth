use super::*;

pub(super) fn certify_fintech_intraday_risk_branch_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let fintech_intraday_risk_samples = capture_counted_samples(|| {
        let mut world = setup_intraday_risk_perf_world(execution);
        let baseline_observability = perf_capture_baseline_observability(&world);
        let analysis = perf_open_analysis_branch(execution, &mut world);

        world.runtime.performance_access().reset_counters();
        let (stress_commit, stress_commit_work) = measure_counted_work(execution, || {
            perf_stress_intraday_risk(execution, &mut world, analysis)
        });

        let (probe, query_probe_work) = measure_counted_work(execution, || {
            perf_capture_intraday_risk_probe(execution, &world)
        });
        let counters = world.runtime.performance_access().counters();
        let post_observability = perf_capture_post_mutation_observability(&world);

        CountedPerfSample {
            charged_work: stress_commit_work + query_probe_work,
            metrics: perf_metrics!({
                "changed_records": stress_commit.changed_records.len(),
                "query_entities": probe.entity_count,
                "query_relations": probe.relation_count,
                "open_breach_count": probe.open_breach_count,
                "diagnostic_artifact_delta": post_observability
                    .diagnostics_artifact_count
                    .saturating_sub(baseline_observability.diagnostics_artifact_count),
                "latest_patch_present": post_observability.latest_patch_present,
                "profile_boundary": profile_boundary_metrics(
                    &world.runtime,
                    RelationalRuntimeProfile::AiWorkflow,
                ),
                "phase_work": {
                    "stress_commit_work": stress_commit_work,
                    "query_probe_work": query_probe_work,
                },
                "shape_metrics": {
                    "packet_count": counters.query_packet_count,
                    "scope_unit_count": counters.query_scope_unit_count,
                },
                "counters": counters,
            }),
        }
    });
    assert!(fintech_intraday_risk_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &fintech_intraday_risk_samples,
        "fintech intraday risk should expose one open breach without widening beyond the case probe",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && metric_u64(metrics, "changed_records") == 4
                && metric_u64(metrics, "query_entities") == 4
                && metric_u64(metrics, "query_relations") == 0
                && metric_u64(metrics, "open_breach_count") == 1
                && metric_u64(metrics, "diagnostic_artifact_delta") >= 1
                && metrics["latest_patch_present"].as_bool() == Some(true)
                && counter_u64(metrics, "query_packet_count") <= 4
                && counter_u64(metrics, "query_scope_unit_count") <= 4
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(3)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(3)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(1)
        },
    );
}
