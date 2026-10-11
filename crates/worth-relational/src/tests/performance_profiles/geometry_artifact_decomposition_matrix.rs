use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_geometry_artifact_decomposition_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let suite = "geometry_artifact_decomposition_matrix";
    let node_count = rocketship_node_count();
    let query_target_count = rocketship_query_target_count(node_count);

    let artifact_decomposition_samples = capture_perf_samples(
        suite,
        "hundred_k_nodes_pseudorealistic_rich_artifact_classes",
        || {
            let mut runtime = runtime_with_test_schema_profile_and_chunks(
                RelationalRuntimeProfile::GeometryKernel,
                ROCKETSHIP_CHUNK_SIZE,
                ROCKETSHIP_CHUNK_SIZE,
            );
            runtime.configure_for_test(|config| {
                config.publication.policy.max_patch_records_per_commit = node_count * 4
            });
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();
            let seeded = seed_pseudorealistic_rocketship_world(
                execution,
                &runtime,
                node_count,
                query_target_count,
            );

            let hot_update_started_at = Instant::now();
            let update = update_entity(
                execution,
                &runtime,
                seeded.hot_update_target,
                "rocketship-rich-artifact-update",
            );
            let hot_update_micros = hot_update_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let explicit_targets = seeded
                .mixed_query_targets
                .iter()
                .take(12)
                .cloned()
                .collect::<Vec<_>>();
            let explicit_started_at = Instant::now();
            let explicit_outcome = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(
                            &snapshot,
                            explicit_query_packet(
                                &runtime,
                                &snapshot,
                                "rocketship-rich-artifact-explicit",
                                explicit_targets.clone(),
                            ),
                        )
                        .expect("planned artifact decomposition explicit query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("artifact decomposition explicit query");
            let explicit_query_micros = explicit_started_at.elapsed().as_micros();

            let diagnostics = runtime.publication().diagnostics_since(diagnostics_start);
            let distinct_scopes = diagnostics
                .iter()
                .map(|artifact| format!("{:?}", artifact.scope))
                .collect::<BTreeSet<_>>()
                .len();

            PerfMeasurement {
                elapsed_micros: seeded.entity_commit_micros
                    + seeded.relation_commit_micros
                    + hot_update_micros
                    + explicit_query_micros,
                metrics: perf_metrics!({
                    "resident_node_count": seeded.entities.len(),
                    "resident_relation_count": seeded.relation_count,
                    "subsystem_count": seeded.subsystem_count,
                    "hot_update_micros": hot_update_micros,
                    "explicit_query_micros": explicit_query_micros,
                    "hot_changed_records": update.changed_records.len(),
                    "explicit_result_entities": explicit_outcome.result.entities.len(),
                    "artifact_count_total": diagnostics.len(),
                    "artifact_entry_count_total": diagnostic_entry_count(&diagnostics),
                    "artifact_kind_minimal_summary_count": diagnostic_artifact_kind_count(
                        &diagnostics,
                        DiagnosticsArtifactKind::MinimalSummary,
                    ),
                    "artifact_kind_detailed_trace_count": diagnostic_artifact_kind_count(
                        &diagnostics,
                        DiagnosticsArtifactKind::DetailedTrace,
                    ),
                    "artifact_scope_history_count": diagnostic_artifact_scope_count(
                        &diagnostics,
                        DiagnosticsScope::History,
                    ),
                    "artifact_scope_query_planning_count": diagnostic_artifact_scope_count(
                        &diagnostics,
                        DiagnosticsScope::QueryPlanning,
                    ),
                    "artifact_scope_snapshot_count": diagnostic_artifact_scope_count(
                        &diagnostics,
                        DiagnosticsScope::Snapshot,
                    ),
                    "artifact_scope_count_distinct": distinct_scopes,
                    "entry_code_commit_published_count": diagnostic_entry_code_count(
                        &diagnostics,
                        DiagnosticCode::CommitPublished,
                    ),
                    "entry_code_snapshot_read_path_count": diagnostic_entry_code_count(
                        &diagnostics,
                        DiagnosticCode::SnapshotReadPathInspected,
                    ),
                    "entry_code_visibility_cache_hit_count": diagnostic_entry_code_count(
                        &diagnostics,
                        DiagnosticCode::VisibilityCacheHit,
                    ),
                    "entry_code_visibility_cache_miss_count": diagnostic_entry_code_count(
                        &diagnostics,
                        DiagnosticCode::VisibilityCacheMissReconstructed,
                    ),
                    "counters": runtime.performance_access().counters(),
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "hundred_k_nodes_pseudorealistic_rich_artifact_classes",
        &artifact_decomposition_samples,
        &[
            ("resident_node_count", &["resident_node_count"]),
            ("resident_relation_count", &["resident_relation_count"]),
            ("subsystem_count", &["subsystem_count"]),
            ("hot_update_micros", &["hot_update_micros"]),
            ("explicit_query_micros", &["explicit_query_micros"]),
            ("artifact_count_total", &["artifact_count_total"]),
            (
                "artifact_entry_count_total",
                &["artifact_entry_count_total"],
            ),
            (
                "artifact_kind_minimal_summary_count",
                &["artifact_kind_minimal_summary_count"],
            ),
            (
                "artifact_kind_detailed_trace_count",
                &["artifact_kind_detailed_trace_count"],
            ),
            (
                "artifact_scope_count_distinct",
                &["artifact_scope_count_distinct"],
            ),
        ],
    );
    assert_budget(
        &artifact_decomposition_samples,
        "rich geometry scale decomposition should prove hot-path traces are deferred at rocketship size instead of hiding them behind one giant total",
        |metrics| {
            metrics["resident_node_count"].as_u64() == Some(node_count as u64)
                && metrics["subsystem_count"].as_u64() == Some(12)
                && metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["explicit_result_entities"].as_u64() == Some(12)
                && metrics["artifact_kind_minimal_summary_count"].as_u64().unwrap_or(0) >= 1
                && metrics["artifact_kind_detailed_trace_count"].as_u64() == Some(0)
                && metrics["artifact_scope_count_distinct"].as_u64().unwrap_or(0) >= 2
                && metrics["entry_code_commit_published_count"].as_u64().unwrap_or(0) >= 1
        },
    );
}
