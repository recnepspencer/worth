use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_artifact_recoverability_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let suite = "artifact_recoverability_matrix";

    let geometry_recoverability_samples = capture_perf_samples(
        suite,
        "geometry_diagnostics_summary_vs_trace_recoverability",
        || {
            let runtime = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::GeometryKernel,
            );
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

            let source = create_entity_outcome(execution, &runtime, "recover-geometry-source");
            let middle = create_entity_outcome(execution, &runtime, "recover-geometry-middle");
            let target = create_entity_outcome(execution, &runtime, "recover-geometry-target");
            let source_entity = changed_entities(&source)[0];
            let middle_entity = changed_entities(&middle)[0];
            let target_entity = changed_entities(&target)[0];
            create_relation_outcome(
                execution,
                &runtime,
                source_entity,
                middle_entity,
                "recover-geometry-link-a",
            );
            create_relation_outcome(
                execution,
                &runtime,
                middle_entity,
                target_entity,
                "recover-geometry-link-b",
            );

            let hot_commit_started_at = Instant::now();
            let hot_commit = update_entity(
                execution,
                &runtime,
                middle_entity,
                "recover-geometry-middle-updated",
            );
            let hot_commit_micros = hot_commit_started_at.elapsed().as_micros();
            let hot_bundle = runtime
                .publication()
                .latest_bundle()
                .expect("geometry hot publication bundle")
                .clone();
            let hot_artifacts = runtime.publication().diagnostics_since(diagnostics_start);

            let checkpoint_started_at = Instant::now();
            runtime
                .durability_authority()
                .checkpoint()
                .expect("geometry recoverability checkpoint");
            let checkpoint_micros = checkpoint_started_at.elapsed().as_micros();

            let plan = runtime.durability().recovery_plan(
                crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
            );
            let mut recovered = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::GeometryKernel,
            );
            let recover_started_at = Instant::now();
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("geometry recoverability recovery");
            let recover_micros = recover_started_at.elapsed().as_micros();

            let replay_started_at = Instant::now();
            let replay = recovered
                .replay_authority()
                .replay_commit(
                    RelationalReplayRequest {
                        commit_id: hot_commit.commit.commit_id,
                        branch_id: BranchId("main".to_string()),
                        execution_mode: ReplayExecutionMode::SerialDeterministic,
                        verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
                    },
                    execution,
                )
                .unwrap();
            let replay_commit_micros = replay_started_at.elapsed().as_micros();
            let recovered_envelope = recovered
                .replay()
                .canonical_commit_envelope(hot_commit.commit.commit_id)
                .expect("recovered canonical geometry envelope");

            PerfMeasurement {
                elapsed_micros: hot_commit_micros
                    + checkpoint_micros
                    + recover_micros
                    + replay_commit_micros,
                metrics: perf_metrics!({
                    "hot_summary_entry_count": hot_bundle.diagnostics_summary.entries.len(),
                    "hot_total_artifacts": hot_artifacts.len(),
                    "hot_total_entries": diagnostic_entry_count(&hot_artifacts),
                    "hot_detailed_trace_artifact_count": diagnostic_artifact_kind_count(
                        &hot_artifacts,
                        DiagnosticsArtifactKind::DetailedTrace,
                    ),
                    "hot_detailed_trace_entry_count": hot_artifacts
                        .iter()
                        .filter(|artifact| artifact.kind == DiagnosticsArtifactKind::DetailedTrace)
                        .map(|artifact| artifact.entries.len())
                        .sum::<usize>(),
                    "hot_history_scope_artifact_count": diagnostic_artifact_scope_count(
                        &hot_artifacts,
                        DiagnosticsScope::History,
                    ),
                    "hot_query_scope_artifact_count": diagnostic_artifact_scope_count(
                        &hot_artifacts,
                        DiagnosticsScope::QueryPlanning,
                    ),
                    "hot_commit_published_entries": diagnostic_entry_code_count(
                        &hot_artifacts,
                        DiagnosticCode::CommitPublished,
                    ),
                    "recovered_summary_entry_count": recovered_envelope.diagnostics_summary.entries.len(),
                    "summary_digest_match": digest_diagnostics_surface(&hot_bundle.diagnostics_summary)
                        == digest_diagnostics_surface(&recovered_envelope.diagnostics_summary),
                    "replay_compared_diagnostics_surface": replay
                        .compared_surfaces
                        .contains(&crate::facade::replay::ReplayObservableSurface::Diagnostics),
                    "replay_mismatch_count": replay.mismatches.len(),
                    "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                    "phase_timing": {
                        "hot_commit_micros": hot_commit_micros,
                        "checkpoint_micros": checkpoint_micros,
                        "recover_micros": recover_micros,
                        "replay_commit_micros": replay_commit_micros,
                    },
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "geometry_diagnostics_summary_vs_trace_recoverability",
        &geometry_recoverability_samples,
        &[
            ("hot_commit_micros", &["phase_timing", "hot_commit_micros"]),
            ("checkpoint_micros", &["phase_timing", "checkpoint_micros"]),
            ("recover_micros", &["phase_timing", "recover_micros"]),
            (
                "replay_commit_micros",
                &["phase_timing", "replay_commit_micros"],
            ),
            ("hot_summary_entry_count", &["hot_summary_entry_count"]),
            ("hot_total_artifacts", &["hot_total_artifacts"]),
            ("hot_total_entries", &["hot_total_entries"]),
            (
                "hot_detailed_trace_artifact_count",
                &["hot_detailed_trace_artifact_count"],
            ),
            (
                "hot_detailed_trace_entry_count",
                &["hot_detailed_trace_entry_count"],
            ),
            (
                "recovered_summary_entry_count",
                &["recovered_summary_entry_count"],
            ),
        ],
    );
    assert_budget(
        &geometry_recoverability_samples,
        "geometry diagnostics recoverability should prove canonical summary replay parity while treating detailed traces as deferred hot-path richness rather than required replay truth",
        |metrics| {
            metrics["hot_summary_entry_count"].as_u64().unwrap_or(0) >= 1
                && metrics["hot_detailed_trace_artifact_count"].as_u64() == Some(0)
                && metrics["hot_detailed_trace_entry_count"].as_u64() == Some(0)
                && metrics["summary_digest_match"].as_bool() == Some(true)
                && metrics["replay_compared_diagnostics_surface"].as_bool() == Some(true)
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
        },
    );

    let chip_recoverability_samples = capture_perf_samples(
        suite,
        "chip_compiled_artifact_recoverability",
        || {
            let mut runtime = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::ChipSimulation,
            );
            apply_perf_diagnostics_policy(
                &mut runtime,
                PerfDiagnosticsPolicy::ChipOperationalHotPath,
            );

            let source = create_entity_in_partition(
                execution,
                &runtime,
                "recover-chip-source",
                PartitionId(7),
            );
            let sinks = (0..8)
                .map(|index| {
                    create_entity_in_partition(
                        execution,
                        &runtime,
                        &format!("recover-chip-sink-{index}"),
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
                    &format!("recover-chip-link-{index}"),
                    PartitionId(19),
                );
            }

            let hot_commit_started_at = Instant::now();
            let hot_commit = update_entity(execution, &runtime, source, "recover-chip-updated");
            let hot_commit_micros = hot_commit_started_at.elapsed().as_micros();
            let latest_commit = runtime
                .history()
                .latest_commit()
                .expect("recoverability latest commit")
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
                .expect("hot recoverable compiled artifact");
            let hot_compile_micros = hot_compile_started_at.elapsed().as_micros();

            let checkpoint_started_at = Instant::now();
            runtime
                .durability_authority()
                .checkpoint()
                .expect("chip recoverability checkpoint");
            let checkpoint_micros = checkpoint_started_at.elapsed().as_micros();

            let plan = runtime.durability().recovery_plan(
                crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
            );
            let mut recovered = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::ChipSimulation,
            );
            let recover_started_at = Instant::now();
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("chip recoverability recovery");
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
                .expect("cold recoverable compiled artifact");
            let cold_compile_micros = cold_compile_started_at.elapsed().as_micros();

            PerfMeasurement {
                elapsed_micros: hot_commit_micros
                    + hot_compile_micros
                    + checkpoint_micros
                    + recover_micros
                    + replay_commit_micros
                    + cold_compile_micros,
                metrics: perf_metrics!({
                    "hot_compiled_record_count": hot_artifact.compiled_record_count,
                    "cold_compiled_record_count": cold_artifact.compiled_record_count,
                    "hot_partition_count": hot_artifact.partition_ids.len(),
                    "cold_partition_count": cold_artifact.partition_ids.len(),
                    "hot_authority_status": format!(
                        "{:?}",
                        runtime.compiled_artifacts().compiled_artifact_authority_status(hot_artifact.artifact_id)
                    ),
                    "cold_authority_status": format!(
                        "{:?}",
                        recovered.compiled_artifacts().compiled_artifact_authority_status(cold_artifact.artifact_id)
                    ),
                    "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                    "replay_mismatch_count": replay.mismatches.len(),
                    "phase_timing": {
                        "hot_commit_micros": hot_commit_micros,
                        "hot_compile_micros": hot_compile_micros,
                        "checkpoint_micros": checkpoint_micros,
                        "recover_micros": recover_micros,
                        "replay_commit_micros": replay_commit_micros,
                        "cold_compile_micros": cold_compile_micros,
                    },
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "chip_compiled_artifact_recoverability",
        &chip_recoverability_samples,
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
            ("hot_compiled_record_count", &["hot_compiled_record_count"]),
            (
                "cold_compiled_record_count",
                &["cold_compiled_record_count"],
            ),
        ],
    );
    assert_budget(
        &chip_recoverability_samples,
        "chip compiled artifacts should be safely reconstructable after recovery and replay rather than requiring hot-path persistence",
        |metrics| {
            metrics["hot_compiled_record_count"] == metrics["cold_compiled_record_count"]
                && metrics["hot_partition_count"] == metrics["cold_partition_count"]
                && metrics["hot_authority_status"].as_str()
                    == Some(&format!("{:?}", CompiledArtifactAuthorityStatus::Authoritative))
                && metrics["cold_authority_status"].as_str()
                    == Some(&format!("{:?}", CompiledArtifactAuthorityStatus::Authoritative))
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
        },
    );
}
