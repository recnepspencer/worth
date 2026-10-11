use super::*;

pub(super) fn certify_chip_hot_compile_vs_recovery_compile(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let chip_hot_vs_recovery_samples = capture_perf_samples(
        suite,
        "chip_hot_compile_vs_recovery_compile",
        || {
            let mut runtime = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::ChipSimulation,
            );
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
            let hot_commit_started_at = Instant::now();
            let hot_commit = update_entity(execution, &runtime, source, "chip-hot-cold-updated");
            let hot_commit_micros = hot_commit_started_at.elapsed().as_micros();
            let latest_commit = runtime
                .history()
                .latest_commit()
                .expect("chip hot/cold latest commit")
                .clone();
            let hot_compile_started_at = Instant::now();
            let hot_artifact = runtime
                .compiled_artifacts_authority()
                .compile_execution_artifact(
                    latest_commit.commit_id,
                    vec![
                        PartitionId(7),
                        PartitionId(11),
                        PartitionId(12),
                        PartitionId(19),
                    ],
                )
                .expect("hot chip compiled artifact");
            let hot_compile_micros = hot_compile_started_at.elapsed().as_micros();

            let checkpoint_started_at = Instant::now();
            runtime
                .durability_authority()
                .checkpoint()
                .expect("chip hot/cold checkpoint");
            let checkpoint_micros = checkpoint_started_at.elapsed().as_micros();

            let plan = runtime.durability().recovery_plan(
                crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
            );
            let mut recovered = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::ChipSimulation,
            );
            recovered.performance_access().reset_counters();
            let recover_started_at = Instant::now();
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("chip hot/cold recovery");
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

            let recovered_commit = recovered
                .history()
                .latest_commit()
                .expect("recovered chip latest commit")
                .clone();
            let cold_compile_started_at = Instant::now();
            let cold_artifact = recovered
                .compiled_artifacts_authority()
                .compile_execution_artifact(
                    recovered_commit.commit_id,
                    vec![
                        PartitionId(7),
                        PartitionId(11),
                        PartitionId(12),
                        PartitionId(19),
                    ],
                )
                .expect("cold chip compiled artifact");
            let cold_compile_micros = cold_compile_started_at.elapsed().as_micros();

            let elapsed_micros = hot_commit_micros
                + hot_compile_micros
                + checkpoint_micros
                + recover_micros
                + replay_commit_micros
                + cold_compile_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "hot_changed_records": hot_commit.changed_records.len(),
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
                    "phase_timing": {
                        "hot_commit_micros": hot_commit_micros,
                        "hot_compile_micros": hot_compile_micros,
                        "checkpoint_micros": checkpoint_micros,
                        "recover_micros": recover_micros,
                        "replay_commit_micros": replay_commit_micros,
                        "cold_compile_micros": cold_compile_micros,
                    },
                    "hot_counters": runtime.performance_access().counters(),
                    "cold_counters": recovered.performance_access().counters(),
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "chip_hot_compile_vs_recovery_compile",
        &chip_hot_vs_recovery_samples,
        &[
            ("hot_commit_micros", &["phase_timing", "hot_commit_micros"]),
            (
                "hot_compile_micros",
                &["phase_timing", "hot_compile_micros"],
            ),
            ("checkpoint_micros", &["phase_timing", "checkpoint_micros"]),
            ("recover_micros", &["phase_timing", "recover_micros"]),
            (
                "replay_commit_micros",
                &["phase_timing", "replay_commit_micros"],
            ),
            (
                "cold_compile_micros",
                &["phase_timing", "cold_compile_micros"],
            ),
        ],
    );
    assert!(chip_hot_vs_recovery_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
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
                && metrics["phase_timing"]["hot_compile_micros"].as_u64().unwrap_or(0) > 0
                && metrics["phase_timing"]["recover_micros"].as_u64().unwrap_or(0) > 0
                && metrics["phase_timing"]["cold_compile_micros"].as_u64().unwrap_or(0) > 0
                && metrics["hot_counters"]["full_state_clones"].as_u64() == Some(0)
                && metrics["cold_counters"]["full_state_clones"].as_u64() == Some(0)
        },
    );
}
