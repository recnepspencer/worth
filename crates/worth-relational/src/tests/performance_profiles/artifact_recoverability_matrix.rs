use super::*;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_artifact_recoverability_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let geometry_recoverability_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::GeometryKernel, &medium);
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

        let (hot_commit, hot_commit_work) = measure_counted_work(execution, || {
            update_entity(
                execution,
                &runtime,
                middle_entity,
                "recover-geometry-middle-updated",
            )
        });
        let hot_envelope = crate::tests::support::latest_committed_envelope(&runtime)
            .expect("geometry hot publication bundle")
            .clone();
        let hot_artifacts = runtime.publication().diagnostics_since(diagnostics_start);

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            capture_and_file_checkpoint(execution, &runtime, &medium)
        });
        assert!(medium.has_filed_checkpoint());
        assert_eq!(medium.committed_count(), 6);

        let mut recovered =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::GeometryKernel, &medium);
        let (_, recover_work) = measure_counted_work(execution, || {
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
                        verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
                    },
                    execution,
                )
                .unwrap()
        });
        let recovered_envelope = recovered
            .replay()
            .canonical_commit_envelope(execution, hot_commit.commit.commit_id)
            .unwrap()
            .expect("recovered canonical geometry envelope");
        let authoritative_hot_summary = crate::diagnostics::data::RelationalDiagnosticArtifact {
            scope: hot_envelope.diagnostics_summary.scope,
            kind: hot_envelope.diagnostics_summary.kind,
            determinism: hot_envelope.diagnostics_summary.determinism,
            entries: Vec::new(),
        };

        CountedPerfSample {
            charged_work: hot_commit_work + checkpoint_work + recover_work + replay_commit_work,
            metrics: perf_metrics!({
                "hot_summary_entry_count": hot_envelope.diagnostics_summary.entries.len(),
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
                "summary_digest_match": digest_diagnostics_surface(&authoritative_hot_summary)
                    == digest_diagnostics_surface(&recovered_envelope.diagnostics_summary),
                "replay_compared_diagnostics_surface": replay
                    .compared_surfaces
                    .contains(&crate::facade::replay::ReplayObservableSurface::Diagnostics),
                "replay_mismatch_count": replay.mismatches.len(),
                "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                "phase_work": {
                    "hot_commit_work": hot_commit_work,
                    "checkpoint_work": checkpoint_work,
                    "recover_work": recover_work,
                    "replay_commit_work": replay_commit_work,
                },
            }),
        }
    });
    assert!(geometry_recoverability_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
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

    let chip_recoverability_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let mut runtime =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
        apply_perf_diagnostics_policy(&mut runtime, PerfDiagnosticsPolicy::ChipOperationalHotPath);

        let source =
            create_entity_in_partition(execution, &runtime, "recover-chip-source", PartitionId(7));
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

        let (hot_commit, hot_commit_work) = measure_counted_work(execution, || {
            update_entity(execution, &runtime, source, "recover-chip-updated")
        });
        let latest_commit = runtime
            .history()
            .latest_commit()
            .expect("recoverability latest commit")
            .clone();

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            capture_and_file_checkpoint(execution, &runtime, &medium)
        });
        assert!(medium.has_filed_checkpoint());
        assert_eq!(medium.committed_count(), 18);

        let mut recovered =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
        let (_, recover_work) = measure_counted_work(execution, || {
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
        assert_eq!(latest_commit, recovered_commit);

        CountedPerfSample {
            charged_work: hot_commit_work + checkpoint_work + recover_work + replay_commit_work,
            metrics: perf_metrics!({
                "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                "replay_mismatch_count": replay.mismatches.len(),
                "phase_work": {
                    "hot_commit_work": hot_commit_work,
                    "checkpoint_work": checkpoint_work,
                    "recover_work": recover_work,
                    "replay_commit_work": replay_commit_work,
                },
            }),
        }
    });
    assert!(chip_recoverability_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &chip_recoverability_samples,
        "chip truth should remain replayable after recovery",
        |metrics| {
            metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
        },
    );
}
