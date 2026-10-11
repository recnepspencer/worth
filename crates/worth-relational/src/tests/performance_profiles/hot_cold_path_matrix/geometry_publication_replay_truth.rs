use super::*;

pub(super) fn certify_geometry_rich_publication_hot_vs_replay_truth(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let geometry_rich_publication_samples = capture_perf_samples(
        suite,
        "geometry_rich_publication_hot_vs_replay_truth",
        || {
            let runtime = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::GeometryKernel,
            );
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

            let source =
                create_entity_outcome(execution, &runtime, "hot-cold-geometry-rich-source");
            let middle =
                create_entity_outcome(execution, &runtime, "hot-cold-geometry-rich-middle");
            let target =
                create_entity_outcome(execution, &runtime, "hot-cold-geometry-rich-target");
            let source_entity = changed_entities(&source)[0];
            let middle_entity = changed_entities(&middle)[0];
            let target_entity = changed_entities(&target)[0];
            create_relation_outcome(
                execution,
                &runtime,
                source_entity,
                middle_entity,
                "hot-cold-geometry-rich-link-a",
            );
            create_relation_outcome(
                execution,
                &runtime,
                middle_entity,
                target_entity,
                "hot-cold-geometry-rich-link-b",
            );

            runtime.performance_access().reset_counters();
            let hot_commit_started_at = Instant::now();
            let hot_commit = update_entity(
                execution,
                &runtime,
                middle_entity,
                "hot-cold-geometry-rich-middle-updated",
            );
            let hot_commit_micros = hot_commit_started_at.elapsed().as_micros();
            let hot_phase_timing = hot_commit.execution().phase_timing.clone();

            let snapshot = runtime.visibility_authority().snapshot();
            let hot_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "hot-cold-geometry-rich-hot-query",
                vec![
                    RecordRef::Entity(source_entity),
                    RecordRef::Entity(middle_entity),
                ],
            );
            let hot_query_started_at = Instant::now();
            let hot_query = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, hot_packet)
                        .expect("planned hot rich geometry query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("hot rich geometry query outcome");
            let hot_query_micros = hot_query_started_at.elapsed().as_micros();
            let (hot_diagnostic_artifact_count, hot_detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);

            let checkpoint_started_at = Instant::now();
            runtime
                .durability_authority()
                .checkpoint()
                .expect("geometry rich hot/cold checkpoint");
            let checkpoint_micros = checkpoint_started_at.elapsed().as_micros();

            let plan = runtime.durability().recovery_plan(
                crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
            );
            let mut recovered = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::GeometryKernel,
            );
            recovered.performance_access().reset_counters();
            let recover_started_at = Instant::now();
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("geometry rich hot/cold recovery");
            let recover_micros = recover_started_at.elapsed().as_micros();

            let replay_started_at = Instant::now();
            let replay = recovered.replay_authority().replay_commit(
                RelationalReplayRequest {
                    commit_id: hot_commit.commit.commit_id,
                    branch_id: BranchId("main".to_string()),
                    execution_mode: ReplayExecutionMode::SerialDeterministic,
                    verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                },
                execution,
            );
            let replay_commit_micros = replay_started_at.elapsed().as_micros();

            let recovered_snapshot = recovered.visibility_authority().snapshot();
            let cold_packet = explicit_query_packet(
                &recovered,
                &recovered_snapshot,
                "hot-cold-geometry-rich-cold-query",
                vec![
                    RecordRef::Entity(source_entity),
                    RecordRef::Entity(middle_entity),
                ],
            );
            let cold_query_started_at = Instant::now();
            let cold_query = recovered
                .read_truth()
                .execute_query_plan(
                    recovered
                        .read_truth()
                        .plan_query_packet(&recovered_snapshot, cold_packet)
                        .expect("planned cold rich geometry query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("cold rich geometry query outcome");
            let cold_query_micros = cold_query_started_at.elapsed().as_micros();

            let elapsed_micros = hot_commit_micros
                + hot_query_micros
                + checkpoint_micros
                + recover_micros
                + replay_commit_micros
                + cold_query_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "hot_changed_records": hot_commit.changed_records.len(),
                    "hot_result_entities": hot_query.result.entities.len(),
                    "cold_result_entities": cold_query.result.entities.len(),
                    "hot_diagnostic_artifact_count": hot_diagnostic_artifact_count,
                    "hot_detailed_trace_entries": hot_detailed_trace_entries,
                    "replay_mismatch_count": replay.mismatches.len(),
                    "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                    "phase_timing": {
                        "hot_commit_micros": hot_commit_micros,
                        "hot_query_micros": hot_query_micros,
                        "artifact_assembly_micros": hot_phase_timing.artifact_assembly_micros,
                        "durable_append_micros": hot_phase_timing.durable_append_micros,
                        "publication_micros": hot_phase_timing.publication_micros,
                        "checkpoint_micros": checkpoint_micros,
                        "recover_micros": recover_micros,
                        "replay_commit_micros": replay_commit_micros,
                        "cold_query_micros": cold_query_micros,
                    },
                    "hot_counters": runtime.performance_access().counters(),
                    "cold_counters": recovered.performance_access().counters(),
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "geometry_rich_publication_hot_vs_replay_truth",
        &geometry_rich_publication_samples,
        &[
            ("hot_commit_micros", &["phase_timing", "hot_commit_micros"]),
            ("hot_query_micros", &["phase_timing", "hot_query_micros"]),
            (
                "artifact_assembly_micros",
                &["phase_timing", "artifact_assembly_micros"],
            ),
            (
                "durable_append_micros",
                &["phase_timing", "durable_append_micros"],
            ),
            (
                "publication_micros",
                &["phase_timing", "publication_micros"],
            ),
            ("checkpoint_micros", &["phase_timing", "checkpoint_micros"]),
            ("recover_micros", &["phase_timing", "recover_micros"]),
            (
                "replay_commit_micros",
                &["phase_timing", "replay_commit_micros"],
            ),
            ("cold_query_micros", &["phase_timing", "cold_query_micros"]),
            (
                "hot_diagnostic_artifact_count",
                &["hot_diagnostic_artifact_count"],
            ),
            (
                "hot_detailed_trace_entries",
                &["hot_detailed_trace_entries"],
            ),
        ],
    );
    assert!(geometry_rich_publication_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &geometry_rich_publication_samples,
        "geometry rich hot/cold certification should isolate summaries on the hot side while proving replay-recoverable truth on the cold side",
        |metrics| {
            metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["hot_result_entities"].as_u64() == Some(2)
                && metrics["cold_result_entities"].as_u64() == Some(2)
                && metrics["hot_diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["hot_detailed_trace_entries"].as_u64() == Some(0)
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
                && metrics["phase_timing"]["artifact_assembly_micros"].as_u64().unwrap_or(0) > 0
                && metrics["phase_timing"]["publication_micros"].as_u64().unwrap_or(0) > 0
                && metrics["phase_timing"]["recover_micros"].as_u64().unwrap_or(0) > 0
                && metrics["phase_timing"]["replay_commit_micros"].as_u64().unwrap_or(0) > 0
                && metrics["hot_counters"]["full_state_clones"].as_u64() == Some(0)
                && metrics["cold_counters"]["full_state_clones"].as_u64() == Some(0)
        },
    );
}
