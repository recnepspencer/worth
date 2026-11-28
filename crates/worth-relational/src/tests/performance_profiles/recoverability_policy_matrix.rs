use super::*;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_recoverability_policy_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let geometry_policy_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::GeometryKernel, &medium);
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

        let (hot_commit, hot_commit_work) = measure_counted_work(execution, || {
            update_entity(execution, &runtime, middle_entity, "policy-middle-updated")
        });
        let hot_envelope = crate::tests::support::latest_committed_envelope(&runtime)
            .expect("policy geometry latest bundle")
            .clone();
        let hot_artifacts = runtime.publication().diagnostics_since(diagnostics_start);

        capture_and_file_checkpoint(execution, &runtime, &medium);
        assert!(medium.has_filed_checkpoint());
        assert_eq!(medium.committed_count(), 6);
        let mut recovered =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::GeometryKernel, &medium);
        recover_runtime_from_medium(execution, &mut recovered, &medium);
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
            .expect("policy recovered geometry envelope");
        let authoritative_hot_summary = crate::diagnostics::data::RelationalDiagnosticArtifact {
            scope: hot_envelope.diagnostics_summary.scope,
            kind: hot_envelope.diagnostics_summary.kind,
            determinism: hot_envelope.diagnostics_summary.determinism,
            entries: Vec::new(),
        };

        CountedPerfSample {
            charged_work: hot_commit_work + replay_commit_work,
            metrics: perf_metrics!({
                "must_be_hot_changed_records": hot_commit.changed_records.len(),
                "reconstructable_summary_entries": hot_envelope.diagnostics_summary.entries.len(),
                "deferred_trace_entries": hot_artifacts
                    .iter()
                    .filter(|artifact| artifact.kind == DiagnosticsArtifactKind::DetailedTrace)
                    .map(|artifact| artifact.entries.len())
                    .sum::<usize>(),
                "summary_reconstructed": digest_diagnostics_surface(&authoritative_hot_summary)
                    == digest_diagnostics_surface(&recovered_envelope.diagnostics_summary),
                "replay_mismatch_count": replay.mismatches.len(),
                "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                "phase_work": {
                    "hot_commit_work": hot_commit_work,
                    "replay_commit_work": replay_commit_work,
                },
            }),
        }
    });
    assert!(geometry_policy_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
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

    let chip_policy_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let mut runtime =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
        runtime.configure_diagnostics_for_test(|profile| {
            profile.detailed_traces_enabled = false;
            profile.max_entries_per_artifact = 0;
        });

        let source =
            create_entity_in_partition(execution, &runtime, "policy-chip-source", PartitionId(7));
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

        let (hot_commit, hot_commit_work) = measure_counted_work(execution, || {
            update_entity(execution, &runtime, source, "policy-chip-updated")
        });
        let latest_commit = runtime
            .history()
            .latest_commit()
            .expect("policy chip latest commit")
            .clone();

        capture_and_file_checkpoint(execution, &runtime, &medium);
        assert!(medium.has_filed_checkpoint());
        assert_eq!(medium.committed_count(), 10);
        let mut recovered =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
        recover_runtime_from_medium(execution, &mut recovered, &medium);
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
            .expect("policy recovered chip latest commit")
            .clone();
        assert_eq!(latest_commit, recovered_commit);

        CountedPerfSample {
            charged_work: hot_commit_work + replay_commit_work,
            metrics: perf_metrics!({
                "must_be_hot_changed_records": hot_commit.changed_records.len(),
                "replay_mismatch_count": replay.mismatches.len(),
                "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                "phase_work": {
                    "hot_commit_work": hot_commit_work,
                    "replay_commit_work": replay_commit_work,
                },
            }),
        }
    });
    assert!(chip_policy_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &chip_policy_samples,
        "chip policy budgets should keep commit truth hot and recover the retained snapshot",
        |metrics| {
            metrics["must_be_hot_changed_records"].as_u64() == Some(1)
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
        },
    );
}
