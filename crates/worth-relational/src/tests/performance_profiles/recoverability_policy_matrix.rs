use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_recoverability_policy_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let suite = "recoverability_policy_matrix";

    let geometry_policy_samples = capture_perf_samples(
        suite,
        "geometry_hot_truth_vs_deferred_trace_policy",
        || {
            let runtime = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::GeometryKernel,
            );
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

            let source = create_entity_outcome(execution, &runtime, "policy-geometry-source");
            let middle = create_entity_outcome(execution, &runtime, "policy-geometry-middle");
            let target = create_entity_outcome(execution, &runtime, "policy-geometry-target");
            let source_entity = changed_entities(&source)[0];
            let middle_entity = changed_entities(&middle)[0];
            let target_entity = changed_entities(&target)[0];
            create_relation_outcome(
                execution,
                &runtime,
                source_entity,
                middle_entity,
                "policy-link-a",
            );
            create_relation_outcome(
                execution,
                &runtime,
                middle_entity,
                target_entity,
                "policy-link-b",
            );

            let hot_commit_started_at = Instant::now();
            let hot_commit =
                update_entity(execution, &runtime, middle_entity, "policy-middle-updated");
            let hot_commit_micros = hot_commit_started_at.elapsed().as_micros();
            let hot_bundle = runtime
                .publication()
                .latest_bundle()
                .expect("policy geometry latest bundle")
                .clone();
            let hot_artifacts = runtime.publication().diagnostics_since(diagnostics_start);

            runtime
                .durability_authority()
                .checkpoint()
                .expect("policy geometry checkpoint");
            let plan = runtime.durability().recovery_plan(
                crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
            );
            let mut recovered = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::GeometryKernel,
            );
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("policy geometry recovery");
            let replay_started_at = Instant::now();
            let replay = recovered.replay_authority().replay_commit(
                RelationalReplayRequest {
                    commit_id: hot_commit.commit.commit_id,
                    branch_id: BranchId("main".to_string()),
                    execution_mode: ReplayExecutionMode::SerialDeterministic,
                    verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
                },
                execution,
            );
            let replay_commit_micros = replay_started_at.elapsed().as_micros();
            let recovered_envelope = recovered
                .replay()
                .canonical_commit_envelope(hot_commit.commit.commit_id)
                .expect("policy recovered geometry envelope");

            PerfMeasurement {
                elapsed_micros: hot_commit_micros + replay_commit_micros,
                metrics: perf_metrics!({
                    "must_be_hot_changed_records": hot_commit.changed_records.len(),
                    "reconstructable_summary_entries": hot_bundle.diagnostics_summary.entries.len(),
                    "deferred_trace_entries": hot_artifacts
                        .iter()
                        .filter(|artifact| artifact.kind == DiagnosticsArtifactKind::DetailedTrace)
                        .map(|artifact| artifact.entries.len())
                        .sum::<usize>(),
                    "summary_reconstructed": digest_diagnostics_surface(&hot_bundle.diagnostics_summary)
                        == digest_diagnostics_surface(&recovered_envelope.diagnostics_summary),
                    "replay_mismatch_count": replay.mismatches.len(),
                    "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                    "phase_timing": {
                        "hot_commit_micros": hot_commit_micros,
                        "replay_commit_micros": replay_commit_micros,
                    },
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "geometry_hot_truth_vs_deferred_trace_policy",
        &geometry_policy_samples,
        &[
            ("hot_commit_micros", &["phase_timing", "hot_commit_micros"]),
            (
                "replay_commit_micros",
                &["phase_timing", "replay_commit_micros"],
            ),
            (
                "must_be_hot_changed_records",
                &["must_be_hot_changed_records"],
            ),
            (
                "reconstructable_summary_entries",
                &["reconstructable_summary_entries"],
            ),
            ("deferred_trace_entries", &["deferred_trace_entries"]),
        ],
    );
    assert_budget(
        &geometry_policy_samples,
        "geometry policy budgets should keep truth updates hot, canonical summaries reconstructable, and detailed traces explicitly deferrable",
        |metrics| {
            metrics["must_be_hot_changed_records"].as_u64() == Some(1)
                && metrics["reconstructable_summary_entries"].as_u64().unwrap_or(0) >= 1
                && metrics["deferred_trace_entries"].as_u64() == Some(0)
                && metrics["summary_reconstructed"].as_bool() == Some(true)
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
        },
    );

    let chip_policy_samples = capture_perf_samples(
        suite,
        "chip_compile_reconstructable_policy",
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
                "policy-chip-source",
                PartitionId(7),
            );
            let sinks = (0..4)
                .map(|index| {
                    create_entity_in_partition(
                        execution,
                        &runtime,
                        &format!("policy-chip-sink-{index}"),
                        PartitionId(11 + index as u32),
                    )
                })
                .collect::<Vec<_>>();
            for (index, sink) in sinks.iter().enumerate() {
                create_relation_in_partition(
                    execution,
                    &runtime,
                    source,
                    *sink,
                    &format!("policy-chip-link-{index}"),
                    PartitionId(19),
                );
            }

            let hot_commit_started_at = Instant::now();
            let hot_commit = update_entity(execution, &runtime, source, "policy-chip-updated");
            let hot_commit_micros = hot_commit_started_at.elapsed().as_micros();
            let latest_commit = runtime
                .history()
                .latest_commit()
                .expect("policy chip latest commit")
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
                        PartitionId(13),
                        PartitionId(14),
                        PartitionId(19),
                    ],
                )
                .expect("policy hot chip compile");
            let hot_compile_micros = hot_compile_started_at.elapsed().as_micros();

            runtime
                .durability_authority()
                .checkpoint()
                .expect("policy chip checkpoint");
            let plan = runtime.durability().recovery_plan(
                crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
            );
            let mut recovered = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::ChipSimulation,
            );
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("policy chip recovery");
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
            let recovered_commit = recovered
                .history()
                .latest_commit()
                .expect("policy recovered chip latest commit")
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
                        PartitionId(13),
                        PartitionId(14),
                        PartitionId(19),
                    ],
                )
                .expect("policy cold chip compile");
            let cold_compile_micros = cold_compile_started_at.elapsed().as_micros();

            PerfMeasurement {
                elapsed_micros: hot_commit_micros
                    + hot_compile_micros
                    + replay_commit_micros
                    + cold_compile_micros,
                metrics: perf_metrics!({
                    "must_be_hot_changed_records": hot_commit.changed_records.len(),
                    "reconstructable_compiled_record_count": cold_artifact.compiled_record_count,
                    "hot_compiled_record_count": hot_artifact.compiled_record_count,
                    "hot_authority_status": format!(
                        "{:?}",
                        runtime.compiled_artifacts().compiled_artifact_authority_status(hot_artifact.artifact_id)
                    ),
                    "cold_authority_status": format!(
                        "{:?}",
                        recovered.compiled_artifacts().compiled_artifact_authority_status(cold_artifact.artifact_id)
                    ),
                    "replay_mismatch_count": replay.mismatches.len(),
                    "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                    "phase_timing": {
                        "hot_commit_micros": hot_commit_micros,
                        "hot_compile_micros": hot_compile_micros,
                        "replay_commit_micros": replay_commit_micros,
                        "cold_compile_micros": cold_compile_micros,
                    },
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "chip_compile_reconstructable_policy",
        &chip_policy_samples,
        &[
            ("hot_commit_micros", &["phase_timing", "hot_commit_micros"]),
            (
                "hot_compile_micros",
                &["phase_timing", "hot_compile_micros"],
            ),
            (
                "replay_commit_micros",
                &["phase_timing", "replay_commit_micros"],
            ),
            (
                "cold_compile_micros",
                &["phase_timing", "cold_compile_micros"],
            ),
            (
                "must_be_hot_changed_records",
                &["must_be_hot_changed_records"],
            ),
            (
                "reconstructable_compiled_record_count",
                &["reconstructable_compiled_record_count"],
            ),
        ],
    );
    assert_budget(
        &chip_policy_samples,
        "chip policy budgets should keep commit truth hot while treating compiled execution artifacts as reconstructable cold-path products",
        |metrics| {
            metrics["must_be_hot_changed_records"].as_u64() == Some(1)
                && metrics["hot_compiled_record_count"] == metrics["reconstructable_compiled_record_count"]
                && metrics["hot_authority_status"].as_str()
                    == Some(&format!("{:?}", CompiledArtifactAuthorityStatus::Authoritative))
                && metrics["cold_authority_status"].as_str()
                    == Some(&format!("{:?}", CompiledArtifactAuthorityStatus::Authoritative))
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
        },
    );
}
