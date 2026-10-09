use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_profile_matrix() {
    let suite = "profile_matrix";

    let certification_core_rich_samples = capture_perf_samples(
        suite,
        "certification_core_rich_commit_query_round_trip",
        || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore);
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

            runtime.performance_access().reset_counters();
            let commit_started_at = Instant::now();
            let commit_outcome = {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
                for index in 0..24 {
                    txn.push_batch(
                        batch_create(&format!("profile-certification-{index}")),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .expect("test staging stays within configured resource budgets");
                }
                txn.commit(
                    &runtime,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("certification-core commit")
            };
            let commit_micros = commit_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let targets = changed_entities(&commit_outcome)
                .into_iter()
                .map(RecordRef::Entity)
                .collect::<Vec<_>>();
            let packet = explicit_query_packet(&runtime, &snapshot, "profile-core-query", targets);
            let query_started_at = Instant::now();
            let outcome = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, packet)
                        .expect("planned certification-core profile query"),
                )
                .expect("certification-core profile query outcome");
            let query_micros = query_started_at.elapsed().as_micros();
            let elapsed_micros = commit_micros + query_micros;
            let counters = runtime.performance_access().counters();
            let publication = runtime.publication();
            let diagnostics = publication.diagnostic_artifacts();
            let fresh_artifacts = &diagnostics[diagnostics_start..];
            let detailed_trace_entries = fresh_artifacts
                .iter()
                .filter(|artifact| {
                    artifact.kind
                        == crate::facade::diagnostics::DiagnosticsArtifactKind::DetailedTrace
                })
                .map(|artifact| artifact.entries.len())
                .sum::<usize>();

            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "result_entities": outcome.result.entities.len(),
                    "result_relations": outcome.result.relations.len(),
                    "diagnostic_artifact_count": fresh_artifacts.len(),
                    "detailed_trace_entries": detailed_trace_entries,
                    "profile_boundary": profile_boundary_metrics(
                        &runtime,
                        RelationalRuntimeProfile::CertificationCore,
                    ),
                    "phase_timing": {
                        "commit_micros": commit_micros,
                        "query_micros": query_micros,
                    },
                    "shape_metrics": {
                        "packet_count": outcome.complexity.packet_count,
                        "scope_unit_count": counters.query_scope_unit_count,
                    },
                    "counters": counters,
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "certification_core_rich_commit_query_round_trip",
        &certification_core_rich_samples,
        &[
            ("commit_micros", &["phase_timing", "commit_micros"]),
            ("query_micros", &["phase_timing", "query_micros"]),
            ("packet_count", &["shape_metrics", "packet_count"]),
            ("scope_unit_count", &["shape_metrics", "scope_unit_count"]),
            ("diagnostic_artifact_count", &["diagnostic_artifact_count"]),
            ("detailed_trace_entries", &["detailed_trace_entries"]),
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
    assert!(certification_core_rich_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &certification_core_rich_samples,
        "certification-core rich diagnostics should preserve scoped truth while surfacing trace cost",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "query_packet_count") <= 2
                && metrics["result_entities"].as_u64() == Some(24)
                && metrics["result_relations"].as_u64() == Some(0)
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64().unwrap_or(0) >= 1
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(1)
        },
    );

    let geometry_kernel_rich_samples = capture_perf_samples(
        suite,
        "geometry_kernel_rich_commit_query_round_trip",
        || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

            runtime.performance_access().reset_counters();
            let commit_started_at = Instant::now();
            let commit_outcome = {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
                for index in 0..24 {
                    txn.push_batch(
                        batch_create(&format!("profile-geometry-{index}")),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .expect("test staging stays within configured resource budgets");
                }
                txn.commit(
                    &runtime,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("geometry-kernel commit")
            };
            let commit_micros = commit_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let targets = changed_entities(&commit_outcome)
                .into_iter()
                .map(RecordRef::Entity)
                .collect::<Vec<_>>();
            let packet =
                explicit_query_packet(&runtime, &snapshot, "profile-geometry-query", targets);
            let query_started_at = Instant::now();
            let outcome = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, packet)
                        .expect("planned geometry-kernel profile query"),
                )
                .expect("geometry-kernel profile query outcome");
            let query_micros = query_started_at.elapsed().as_micros();
            let elapsed_micros = commit_micros + query_micros;
            let counters = runtime.performance_access().counters();
            let publication = runtime.publication();
            let diagnostics = publication.diagnostic_artifacts();
            let fresh_artifacts = &diagnostics[diagnostics_start..];
            let detailed_trace_entries = fresh_artifacts
                .iter()
                .filter(|artifact| {
                    artifact.kind
                        == crate::facade::diagnostics::DiagnosticsArtifactKind::DetailedTrace
                })
                .map(|artifact| artifact.entries.len())
                .sum::<usize>();

            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "result_entities": outcome.result.entities.len(),
                    "result_relations": outcome.result.relations.len(),
                    "diagnostic_artifact_count": fresh_artifacts.len(),
                    "detailed_trace_entries": detailed_trace_entries,
                    "profile_boundary": profile_boundary_metrics(
                        &runtime,
                        RelationalRuntimeProfile::GeometryKernel,
                    ),
                    "phase_timing": {
                        "commit_micros": commit_micros,
                        "query_micros": query_micros,
                    },
                    "shape_metrics": {
                        "packet_count": outcome.complexity.packet_count,
                        "scope_unit_count": counters.query_scope_unit_count,
                    },
                    "counters": counters,
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "geometry_kernel_rich_commit_query_round_trip",
        &geometry_kernel_rich_samples,
        &[
            ("commit_micros", &["phase_timing", "commit_micros"]),
            ("query_micros", &["phase_timing", "query_micros"]),
            ("packet_count", &["shape_metrics", "packet_count"]),
            ("scope_unit_count", &["shape_metrics", "scope_unit_count"]),
            ("diagnostic_artifact_count", &["diagnostic_artifact_count"]),
            ("detailed_trace_entries", &["detailed_trace_entries"]),
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
    assert!(geometry_kernel_rich_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &geometry_kernel_rich_samples,
        "geometry-kernel rich diagnostics should preserve the same scoped truth envelope while deferring hot detailed traces",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "query_packet_count") <= 2
                && metrics["result_entities"].as_u64() == Some(24)
                && metrics["result_relations"].as_u64() == Some(0)
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(1)
        },
    );

    let certification_core_zero_diag_samples = capture_perf_samples(
        suite,
        "certification_core_zero_diagnostics_commit_query_round_trip",
        || {
            let mut runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore);
            runtime.configure_diagnostics_for_test(|profile| {
                profile.detailed_traces_enabled = false;
                profile.max_entries_per_artifact = 0;
            });
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

            runtime.performance_access().reset_counters();
            let commit_started_at = Instant::now();
            let commit_outcome = {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
                for index in 0..24 {
                    txn.push_batch(
                        batch_create(&format!("profile-zero-{index}")),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .expect("test staging stays within configured resource budgets");
                }
                txn.commit(
                    &runtime,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("zero-diagnostics certification-core commit")
            };
            let commit_micros = commit_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let targets = changed_entities(&commit_outcome)
                .into_iter()
                .map(RecordRef::Entity)
                .collect::<Vec<_>>();
            let packet = explicit_query_packet(&runtime, &snapshot, "profile-zero-query", targets);
            let query_started_at = Instant::now();
            let outcome = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, packet)
                        .expect("planned zero-diagnostics profile query"),
                )
                .expect("zero-diagnostics profile query outcome");
            let query_micros = query_started_at.elapsed().as_micros();
            let elapsed_micros = commit_micros + query_micros;
            let counters = runtime.performance_access().counters();
            let publication = runtime.publication();
            let diagnostics = publication.diagnostic_artifacts();
            let fresh_artifacts = &diagnostics[diagnostics_start..];
            let detailed_trace_entries = fresh_artifacts
                .iter()
                .filter(|artifact| {
                    artifact.kind
                        == crate::facade::diagnostics::DiagnosticsArtifactKind::DetailedTrace
                })
                .map(|artifact| artifact.entries.len())
                .sum::<usize>();

            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "result_entities": outcome.result.entities.len(),
                    "result_relations": outcome.result.relations.len(),
                    "diagnostic_artifact_count": fresh_artifacts.len(),
                    "detailed_trace_entries": detailed_trace_entries,
                    "profile_boundary": profile_boundary_metrics(
                        &runtime,
                        RelationalRuntimeProfile::CertificationCore,
                    ),
                    "phase_timing": {
                        "commit_micros": commit_micros,
                        "query_micros": query_micros,
                    },
                    "shape_metrics": {
                        "packet_count": outcome.complexity.packet_count,
                        "scope_unit_count": counters.query_scope_unit_count,
                    },
                    "counters": counters,
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "certification_core_zero_diagnostics_commit_query_round_trip",
        &certification_core_zero_diag_samples,
        &[
            ("commit_micros", &["phase_timing", "commit_micros"]),
            ("query_micros", &["phase_timing", "query_micros"]),
            ("packet_count", &["shape_metrics", "packet_count"]),
            ("scope_unit_count", &["shape_metrics", "scope_unit_count"]),
            ("diagnostic_artifact_count", &["diagnostic_artifact_count"]),
            ("detailed_trace_entries", &["detailed_trace_entries"]),
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
    assert!(certification_core_zero_diag_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &certification_core_zero_diag_samples,
        "zero-budget diagnostics should preserve scoped truth while eliminating trace entry pressure",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "query_packet_count") <= 2
                && metrics["result_entities"].as_u64() == Some(24)
                && metrics["result_relations"].as_u64() == Some(0)
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(0)
        },
    );
}
