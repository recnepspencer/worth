use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

pub(super) fn certify_geometry_hot_commit_vs_replay_reconstruction(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let geometry_hot_vs_replay_samples = capture_perf_samples(
        suite,
        "geometry_hot_commit_vs_replay_reconstruction",
        || {
            let mut runtime =
                durable_runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            runtime.configure_diagnostics_for_test(|profile| {
                profile.detailed_traces_enabled = false;
                profile.max_entries_per_artifact = 0;
            });

            let source = create_entity_outcome(execution, &runtime, "hot-cold-geometry-source");
            let middle = create_entity_outcome(execution, &runtime, "hot-cold-geometry-middle");
            let target = create_entity_outcome(execution, &runtime, "hot-cold-geometry-target");
            let source_entity = changed_entities(&source)[0];
            let middle_entity = changed_entities(&middle)[0];
            let target_entity = changed_entities(&target)[0];
            create_relation_outcome(
                execution,
                &runtime,
                source_entity,
                middle_entity,
                "hot-cold-geometry-link-a",
            );
            create_relation_outcome(
                execution,
                &runtime,
                middle_entity,
                target_entity,
                "hot-cold-geometry-link-b",
            );

            runtime.performance_access().reset_counters();
            let hot_commit_started_at = Instant::now();
            let hot_commit = update_entity(
                execution,
                &runtime,
                middle_entity,
                "hot-cold-geometry-middle-updated",
            );
            let hot_commit_micros = hot_commit_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let hot_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "hot-cold-geometry-hot-query",
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
                        .expect("planned hot geometry query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("hot geometry query outcome");
            let hot_query_micros = hot_query_started_at.elapsed().as_micros();

            let checkpoint_started_at = Instant::now();
            crate::tests::support::checkpoint_durably(execution, &runtime)
                .expect("geometry hot/cold checkpoint");
            let checkpoint_micros = checkpoint_started_at.elapsed().as_micros();

            let plan = crate::tests::support::recovery_plan_for(
                execution,
                &runtime,
                RecoveryMode::NormalRecoveryVerification,
            )
            .unwrap();
            let mut recovered =
                durable_runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            recovered.performance_access().reset_counters();
            let recover_started_at = Instant::now();
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("geometry hot/cold recovery");
            let recover_micros = recover_started_at.elapsed().as_micros();

            let replay_started_at = Instant::now();
            let replay = recovered
                .replay_authority()
                .replay_commit(
                    RelationalReplayRequest {
                        commit_id: hot_commit.commit.commit_id,
                        branch_id: BranchId("main".to_string()),
                        execution_mode: ReplayExecutionMode::SerialDeterministic,
                        verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                    },
                    execution,
                )
                .unwrap();
            let replay_commit_micros = replay_started_at.elapsed().as_micros();

            let recovered_snapshot = recovered.visibility_authority().snapshot();
            let cold_packet = explicit_query_packet(
                &recovered,
                &recovered_snapshot,
                "hot-cold-geometry-cold-query",
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
                        .expect("planned cold geometry query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("cold geometry query outcome");
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
                    "replay_mismatch_count": replay.mismatches.len(),
                    "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                    "phase_timing": {
                        "hot_commit_micros": hot_commit_micros,
                        "hot_query_micros": hot_query_micros,
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
        "geometry_hot_commit_vs_replay_reconstruction",
        &geometry_hot_vs_replay_samples,
        &[
            ("hot_commit_micros", &["phase_timing", "hot_commit_micros"]),
            ("hot_query_micros", &["phase_timing", "hot_query_micros"]),
            ("checkpoint_micros", &["phase_timing", "checkpoint_micros"]),
            ("recover_micros", &["phase_timing", "recover_micros"]),
            (
                "replay_commit_micros",
                &["phase_timing", "replay_commit_micros"],
            ),
            ("cold_query_micros", &["phase_timing", "cold_query_micros"]),
        ],
    );
    assert!(geometry_hot_vs_replay_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &geometry_hot_vs_replay_samples,
        "geometry hot/cold certification should keep hot updates narrow while proving truth is replay-recoverable on the cold path",
        |metrics| {
            metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["hot_result_entities"].as_u64() == Some(2)
                && metrics["cold_result_entities"].as_u64() == Some(2)
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
                && metrics["phase_timing"]["hot_commit_micros"].as_u64().unwrap_or(0) > 0
                && metrics["phase_timing"]["recover_micros"].as_u64().unwrap_or(0) > 0
                && metrics["phase_timing"]["replay_commit_micros"].as_u64().unwrap_or(0) > 0
                && metrics["hot_counters"]["full_state_clones"].as_u64() == Some(0)
                && metrics["cold_counters"]["full_state_clones"].as_u64() == Some(0)
        },
    );
}
