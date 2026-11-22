use super::*;

pub(super) fn certify_fintech_trade_correction_audit_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let fintech_trade_correction_samples = capture_counted_samples(|| {
        let mut world = setup_trade_correction_perf_world(execution);
        let baseline_observability = perf_capture_baseline_observability(&world);
        let analysis = perf_open_analysis_branch(execution, &mut world);

        world.runtime.performance_access().reset_counters();
        let (correction_commit, correction_commit_work) = measure_counted_work(execution, || {
            perf_correct_trade_correction(execution, &mut world, analysis.clone())
        });

        let (audit_commit, audit_commit_work) = measure_counted_work(execution, || {
            perf_emit_trade_correction_audit(execution, &mut world, analysis)
        });

        let (probe, query_probe_work) = measure_counted_work(execution, || {
            perf_capture_trade_correction_probe(execution, &world)
        });
        let counters = world.runtime.performance_access().counters();
        let post_observability = perf_capture_post_mutation_observability(&world);

        CountedPerfSample {
            charged_work: correction_commit_work + audit_commit_work + query_probe_work,
            metrics: perf_metrics!({
                "correction_records": correction_commit.changed_records.len(),
                "audit_records": audit_commit.changed_records.len(),
                "query_entities": probe.entity_count,
                "query_relations": probe.relation_count,
                "corrected_trade_count": probe.corrected_trade_count,
                "audit_record_count": probe.audit_record_count,
                "diagnostic_artifact_delta": post_observability
                    .diagnostics_artifact_count
                    .saturating_sub(baseline_observability.diagnostics_artifact_count),
                "profile_boundary": profile_boundary_metrics(
                    &world.runtime,
                    RelationalRuntimeProfile::AiWorkflow,
                ),
                "phase_work": {
                    "correction_commit_work": correction_commit_work,
                    "audit_commit_work": audit_commit_work,
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
    assert!(fintech_trade_correction_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &fintech_trade_correction_samples,
        "fintech trade correction should surface one corrected trade and one audit record without broadening the case probe",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && metric_u64(metrics, "correction_records") == 1
                && metric_u64(metrics, "audit_records") == 1
                && metric_u64(metrics, "query_entities") == 3
                && metric_u64(metrics, "query_relations") == 0
                && metric_u64(metrics, "corrected_trade_count") == 1
                && metric_u64(metrics, "audit_record_count") == 1
                && metric_u64(metrics, "diagnostic_artifact_delta") >= 2
                && counter_u64(metrics, "query_packet_count") <= 3
                && counter_u64(metrics, "query_scope_unit_count") <= 3
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(3)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(3)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(1)
        },
    );
}
