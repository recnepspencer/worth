use super::*;

pub(super) fn certify_chip_hot_compile_vs_recovery_compile(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let chip_hot_vs_recovery_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let mut runtime =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
        runtime.configure_diagnostics_for_test(|profile| {
            profile.detailed_traces_enabled = false;
            profile.max_entries_per_artifact = 0;
        });

        let source = create_entity_in_partition(
            execution,
            &runtime,
            "chip-hot-cold-source",
            PartitionId(7),
        );
        let sinks = (0..8)
            .map(|index| {
                create_entity_in_partition(
                    execution,
                    &runtime,
                    &format!("chip-hot-cold-sink-{index}"),
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
                &format!("chip-hot-cold-link-{index}"),
                PartitionId(19),
            );
        }

        runtime.performance_access().reset_counters();
        let (hot_commit, hot_commit_work) = measure_counted_work(execution, || {
            update_entity(execution, &runtime, source, "chip-hot-cold-updated")
        });
        let latest_commit = runtime
            .history()
            .latest_commit()
            .expect("chip hot/cold latest commit")
            .clone();
        let (hot_artifact, hot_compile_work) = measure_counted_work(execution, || {
            runtime
                .compiled_artifacts_authority()
                .compile_execution_artifact(
                    execution,
                    latest_commit.commit_id,
                    vec![
                        PartitionId(7),
                        PartitionId(11),
                        PartitionId(12),
                        PartitionId(19),
                    ],
                )
                .expect("hot chip compiled artifact")
        });

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            capture_and_file_checkpoint(execution, &runtime, &medium)
        });
        assert!(medium.has_filed_checkpoint());
        assert_eq!(medium.committed_count(), 18);

        let mut recovered =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
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
            .expect("recovered chip latest commit")
            .clone();
        let (cold_artifact, cold_compile_work) = measure_counted_work(execution, || {
            recovered
                .compiled_artifacts_authority()
                .compile_execution_artifact(
                    execution,
                    recovered_commit.commit_id,
                    vec![
                        PartitionId(7),
                        PartitionId(11),
                        PartitionId(12),
                        PartitionId(19),
                    ],
                )
                .expect("cold chip compiled artifact")
        });

        CountedPerfSample {
            charged_work: hot_commit_work
                + hot_compile_work
                + checkpoint_work
                + recover_work
                + replay_commit_work
                + cold_compile_work,
            metrics: perf_metrics!({
                "hot_changed_records": hot_commit.changed_records.len(),
                "hot_compiled_record_count": hot_artifact.compiled_record_count,
                "cold_compiled_record_count": cold_artifact.compiled_record_count,
                "recovered_commits": recovery_outcome.recovered_commits,
                "replay_mismatch_count": replay.mismatches.len(),
                "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                "hot_authority_status": format!(
                    "{:?}",
                    runtime.compiled_artifacts().compiled_artifact_authority_status(hot_artifact.artifact_id)
                ),
                "cold_authority_status": format!(
                    "{:?}",
                    recovered.compiled_artifacts().compiled_artifact_authority_status(cold_artifact.artifact_id)
                ),
                "phase_work": {
                    "hot_commit_work": hot_commit_work,
                    "hot_compile_work": hot_compile_work,
                    "checkpoint_work": checkpoint_work,
                    "recover_work": recover_work,
                    "replay_commit_work": replay_commit_work,
                    "cold_compile_work": cold_compile_work,
                },
                "hot_counters": runtime.performance_access().counters(),
                "cold_counters": recovered.performance_access().counters(),
            }),
        }
    });
    assert!(chip_hot_vs_recovery_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &chip_hot_vs_recovery_samples,
        "chip hot/cold certification should keep compile-ready stepping narrow while preserving recovery-compile equivalence on the cold path",
        |metrics| {
            metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
                && metrics["hot_authority_status"].as_str()
                    == Some(&format!("{:?}", CompiledArtifactAuthorityStatus::Authoritative))
                && metrics["cold_authority_status"].as_str()
                    == Some(&format!("{:?}", CompiledArtifactAuthorityStatus::Authoritative))
                && metrics["hot_compiled_record_count"].as_u64().unwrap_or(0) > 0
                && metrics["hot_compiled_record_count"] == metrics["cold_compiled_record_count"]
                && metrics["recovered_commits"].as_u64().unwrap_or(0) >= 1
                && metrics["hot_counters"]["full_state_clones"].as_u64() == Some(0)
                && metrics["cold_counters"]["full_state_clones"].as_u64() == Some(0)
        },
    );
}
