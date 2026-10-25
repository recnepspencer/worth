use super::*;

pub(super) fn certify_fintech_trade_correction_audit_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let fintech_trade_correction_samples =
        capture_perf_samples(suite, "fintech_trade_correction_audit_round_trip", || {
            let mut world = setup_trade_correction_perf_world(execution);
            let baseline_observability = perf_capture_baseline_observability(&world);
            let analysis = perf_open_analysis_branch(execution, &mut world);

            world.runtime.performance_access().reset_counters();
            let correction_started_at = Instant::now();
            let correction_commit =
                perf_correct_trade_correction(execution, &mut world, analysis.clone());
            let correction_commit_micros = correction_started_at.elapsed().as_micros();

            let audit_started_at = Instant::now();
            let audit_commit = perf_emit_trade_correction_audit(execution, &mut world, analysis);
            let audit_commit_micros = audit_started_at.elapsed().as_micros();

            let query_started_at = Instant::now();
            let probe = perf_capture_trade_correction_probe(execution, &world);
            let query_probe_micros = query_started_at.elapsed().as_micros();
            let elapsed_micros =
                correction_commit_micros + audit_commit_micros + query_probe_micros;
            let counters = world.runtime.performance_access().counters();
            let post_observability = perf_capture_post_mutation_observability(&world);

            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
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
                    "phase_timing": {
                        "correction_commit_micros": correction_commit_micros,
                        "audit_commit_micros": audit_commit_micros,
                        "query_probe_micros": query_probe_micros,
                    },
                    "shape_metrics": {
                        "packet_count": counters.query_packet_count,
                        "scope_unit_count": counters.query_scope_unit_count,
                    },
                    "counters": counters,
                })
            })
        });
    emit_metric_summaries(
        suite,
        "fintech_trade_correction_audit_round_trip",
        &fintech_trade_correction_samples,
        &[
            (
                "correction_commit_micros",
                &["phase_timing", "correction_commit_micros"],
            ),
            (
                "audit_commit_micros",
                &["phase_timing", "audit_commit_micros"],
            ),
            (
                "query_probe_micros",
                &["phase_timing", "query_probe_micros"],
            ),
            ("packet_count", &["shape_metrics", "packet_count"]),
            ("scope_unit_count", &["shape_metrics", "scope_unit_count"]),
            ("diagnostic_artifact_delta", &["diagnostic_artifact_delta"]),
            (
                "profile_execution_lane_code",
                &["profile_boundary", "execution_lane_code"],
            ),
            (
                "profile_diagnostics_boundary_code",
                &["profile_boundary", "diagnostics_boundary_code"],
            ),
            (
                "profile_matches_defaults",
                &["profile_boundary", "matches_defaults"],
            ),
        ],
    );
    assert!(fintech_trade_correction_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
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
