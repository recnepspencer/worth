use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

pub(super) fn certify_geometry_hot_commit_vs_replay_reconstruction(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let geometry_hot_vs_replay_samples = capture_counted_samples(|| {
        let mut runtime =
            runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
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
        let (hot_commit, hot_commit_work) = measure_counted_work(execution, || {
            update_entity(
                execution,
                &runtime,
                middle_entity,
                "hot-cold-geometry-middle-updated",
            )
        });

        let snapshot = runtime
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");
        let hot_packet = explicit_query_packet(
            &runtime,
            &snapshot,
            "hot-cold-geometry-hot-query",
            vec![
                RecordRef::Entity(source_entity),
                RecordRef::Entity(middle_entity),
            ],
        );
        let (hot_query, hot_query_work) = measure_counted_work(execution, || {
            runtime
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
                .expect("hot geometry query outcome")
        });

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            runtime
                .durability_authority()
                .checkpoint(execution)
                .expect("geometry hot/cold checkpoint")
        });

        let plan = runtime
            .durability()
            .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
            .unwrap();
        let mut recovered =
            runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
        recovered.performance_access().reset_counters();
        let (recovery_outcome, recover_work) = measure_counted_work(execution, || {
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("geometry hot/cold recovery")
        });

        let (replay, replay_commit_work) = measure_counted_work(execution, || {
            recovered
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
                .unwrap()
        });

        let recovered_snapshot = recovered
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");
        let cold_packet = explicit_query_packet(
            &recovered,
            &recovered_snapshot,
            "hot-cold-geometry-cold-query",
            vec![
                RecordRef::Entity(source_entity),
                RecordRef::Entity(middle_entity),
            ],
        );
        let (cold_query, cold_query_work) = measure_counted_work(execution, || {
            recovered
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
                .expect("cold geometry query outcome")
        });

        CountedPerfSample {
            charged_work: hot_commit_work
                + hot_query_work
                + checkpoint_work
                + recover_work
                + replay_commit_work
                + cold_query_work,
            metrics: perf_metrics!({
                "hot_changed_records": hot_commit.changed_records.len(),
                "hot_result_entities": hot_query.result.entities.len(),
                "cold_result_entities": cold_query.result.entities.len(),
                "recovered_commits": recovery_outcome.recovered_commits,
                "replay_compared_surfaces": replay.compared_surfaces.len(),
                "replay_mismatch_count": replay.mismatches.len(),
                "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                "phase_work": {
                    "hot_commit_work": hot_commit_work,
                    "hot_query_work": hot_query_work,
                    "checkpoint_work": checkpoint_work,
                    "recover_work": recover_work,
                    "replay_commit_work": replay_commit_work,
                    "cold_query_work": cold_query_work,
                },
                "hot_counters": runtime.performance_access().counters(),
                "cold_counters": recovered.performance_access().counters(),
            }),
        }
    });
    assert!(geometry_hot_vs_replay_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &geometry_hot_vs_replay_samples,
        "geometry hot/cold certification should keep hot updates narrow while proving truth is replay-recoverable on the cold path",
        |metrics| {
            metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["hot_result_entities"].as_u64() == Some(2)
                && metrics["cold_result_entities"].as_u64() == Some(2)
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
                && metrics["phase_work"]["hot_commit_work"].as_u64().unwrap_or(0) > 0
                && metrics["recovered_commits"].as_u64().unwrap_or(0) >= 1
                && metrics["replay_compared_surfaces"].as_u64().unwrap_or(0) >= 1
                && metrics["hot_counters"]["full_state_clones"].as_u64() == Some(0)
                && metrics["cold_counters"]["full_state_clones"].as_u64() == Some(0)
        },
    );
}
