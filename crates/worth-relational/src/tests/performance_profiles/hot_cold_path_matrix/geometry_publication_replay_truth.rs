use super::*;

pub(super) fn certify_geometry_rich_publication_hot_vs_replay_truth(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let geometry_rich_publication_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::GeometryKernel, &medium);
        let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

        let source = create_entity_outcome(execution, &runtime, "hot-cold-geometry-rich-source");
        let middle = create_entity_outcome(execution, &runtime, "hot-cold-geometry-rich-middle");
        let target = create_entity_outcome(execution, &runtime, "hot-cold-geometry-rich-target");
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
        let (hot_commit, hot_commit_work) = measure_counted_work(execution, || {
            update_entity(
                execution,
                &runtime,
                middle_entity,
                "hot-cold-geometry-rich-middle-updated",
            )
        });

        let snapshot = runtime
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");
        let hot_packet = explicit_query_packet(
            &runtime,
            &snapshot,
            "hot-cold-geometry-rich-hot-query",
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
                        .expect("planned hot rich geometry query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("hot rich geometry query outcome")
        });
        let (hot_diagnostic_artifact_count, hot_detailed_trace_entries) =
            fresh_diagnostics_metrics(&runtime, diagnostics_start);

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            capture_and_file_checkpoint(execution, &runtime, &medium)
        });
        assert!(medium.has_filed_checkpoint());
        assert_eq!(medium.committed_count(), 6);

        let mut recovered =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::GeometryKernel, &medium);
        recovered.performance_access().reset_counters();
        let (recovery_outcome, recover_work) = measure_counted_work(execution, || {
            recover_runtime_from_medium(execution, &mut recovered, &medium)
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
            "hot-cold-geometry-rich-cold-query",
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
                        .expect("planned cold rich geometry query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("cold rich geometry query outcome")
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
                "hot_diagnostic_artifact_count": hot_diagnostic_artifact_count,
                "hot_detailed_trace_entries": hot_detailed_trace_entries,
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
    assert!(geometry_rich_publication_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
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
                && metrics["phase_work"]["hot_commit_work"].as_u64().unwrap_or(0) > 0
                && metrics["recovered_commits"].as_u64().unwrap_or(0) >= 1
                && metrics["replay_compared_surfaces"].as_u64().unwrap_or(0) >= 1
                && metrics["hot_counters"]["full_state_clones"].as_u64() == Some(0)
                && metrics["cold_counters"]["full_state_clones"].as_u64() == Some(0)
        },
    );
}
