use super::*;

pub(super) fn certify_chip_rich_hot_commit_and_recovery(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let chip_rich_recovery_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let mut runtime =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
        apply_perf_diagnostics_policy(
            &mut runtime,
            PerfDiagnosticsPolicy::ChipRichCertification,
        );
        let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

        let source = create_entity_in_partition(
            execution,
            &runtime,
            "chip-rich-hot-cold-source",
            PartitionId(7),
        );
        let sinks = (0..8)
            .map(|index| {
                create_entity_in_partition(
                    execution,
                    &runtime,
                    &format!("chip-rich-hot-cold-sink-{index}"),
                    if index % 2 == 0 {
                        PartitionId(11)
                    } else {
                        PartitionId(12)
                    },
                )
            })
            .collect::<Vec<_>>();
        for (index, sink) in sinks.iter().enumerate() {
            create_relation_in_partition(
                execution,
                &runtime,
                source,
                *sink,
                &format!("chip-rich-hot-cold-link-{index}"),
                PartitionId(19),
            );
        }

        runtime.performance_access().reset_counters();
        let (hot_commit, hot_commit_work) = measure_counted_work(execution, || {
            update_entity(execution, &runtime, source, "chip-rich-hot-cold-updated")
        });
        let latest_commit = runtime
            .history()
            .latest_commit()
            .expect("chip rich hot/cold latest commit")
            .clone();
        let (hot_diagnostic_artifact_count, hot_detailed_trace_entries) =
            fresh_diagnostics_metrics(&runtime, diagnostics_start);

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            capture_and_file_checkpoint(execution, &runtime, &medium)
        });
        assert!(medium.has_filed_checkpoint());
        assert_eq!(medium.committed_count(), 18);

        let mut recovered =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
        apply_perf_diagnostics_policy(
            &mut recovered,
            PerfDiagnosticsPolicy::ChipRichCertification,
        );
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

        let recovered_commit = recovered
            .history()
            .latest_commit()
            .expect("recovered rich chip latest commit")
            .clone();
        assert_eq!(latest_commit, recovered_commit);

        CountedPerfSample {
            charged_work: hot_commit_work + checkpoint_work + recover_work + replay_commit_work,
            metrics: perf_metrics!({
                "hot_changed_records": hot_commit.changed_records.len(),
                "recovered_commits": recovery_outcome.recovered_commits,
                "hot_diagnostic_artifact_count": hot_diagnostic_artifact_count,
                "hot_detailed_trace_entries": hot_detailed_trace_entries,
                "replay_mismatch_count": replay.mismatches.len(),
                "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                "phase_work": {
                    "hot_commit_work": hot_commit_work,
                    "checkpoint_work": checkpoint_work,
                    "recover_work": recover_work,
                    "replay_commit_work": replay_commit_work,
                },
                "hot_counters": runtime.performance_access().counters(),
                "cold_counters": recovered.performance_access().counters(),
            }),
        }
    });
    assert!(chip_rich_recovery_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &chip_rich_recovery_samples,
        "chip rich hot/cold certification should preserve recovery and separate diagnostic work",
        |metrics| {
            metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["hot_diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["hot_detailed_trace_entries"].as_u64().unwrap_or(0) >= 1
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
                && metrics["recovered_commits"].as_u64().unwrap_or(0) >= 1
                && metrics["hot_counters"]["full_state_clones"].as_u64() == Some(0)
                && metrics["cold_counters"]["full_state_clones"].as_u64() == Some(0)
        },
    );
}
