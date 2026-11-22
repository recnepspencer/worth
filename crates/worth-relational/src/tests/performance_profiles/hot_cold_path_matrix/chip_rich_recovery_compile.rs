use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

pub(super) fn certify_chip_rich_compile_hot_vs_recovery_compile(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let chip_rich_compile_samples = capture_counted_samples(|| {
        let mut runtime =
            runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
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
                .expect("hot rich chip compiled artifact")
        });
        let (hot_diagnostic_artifact_count, hot_detailed_trace_entries) =
            fresh_diagnostics_metrics(&runtime, diagnostics_start);

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            runtime
                .durability_authority()
                .checkpoint(execution)
                .expect("chip rich hot/cold checkpoint")
        });

        let plan = runtime
            .durability()
            .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
            .unwrap();
        let mut recovered =
            runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
        apply_perf_diagnostics_policy(
            &mut recovered,
            PerfDiagnosticsPolicy::ChipRichCertification,
        );
        recovered.performance_access().reset_counters();
        let (recovery_outcome, recover_work) = measure_counted_work(execution, || {
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("chip rich hot/cold recovery")
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
                .expect("cold rich chip compiled artifact")
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
                "hot_diagnostic_artifact_count": hot_diagnostic_artifact_count,
                "hot_detailed_trace_entries": hot_detailed_trace_entries,
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
    assert!(chip_rich_compile_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &chip_rich_compile_samples,
        "chip rich hot/cold certification should isolate compile and diagnostics on the hot side while preserving recovery-compile equivalence",
        |metrics| {
            metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["hot_diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["hot_detailed_trace_entries"].as_u64().unwrap_or(0) >= 1
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
