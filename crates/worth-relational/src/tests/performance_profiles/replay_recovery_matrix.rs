use super::*;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_replay_recovery_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let durable_replay_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_for_medium(&medium);
        create_entity_outcome(execution, &runtime, "source");
        let second = create_entity_outcome(execution, &runtime, "target");
        let replay_commit_id = second.commit.commit_id;
        assert_eq!(medium.committed_count(), 2);

        runtime.performance_access().reset_counters();
        let (outcome, replay_commit_work) = measure_counted_work(execution, || {
            runtime
                .replay_authority()
                .replay_commit(
                    RelationalReplayRequest {
                        commit_id: replay_commit_id,
                        branch_id: BranchId("main".to_string()),
                        execution_mode: ReplayExecutionMode::SerialDeterministic,
                        verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                    },
                    execution,
                )
                .unwrap()
        });
        let counters = runtime.performance_access().counters();

        CountedPerfSample {
            charged_work: replay_commit_work,
            metrics: perf_metrics!({
                "failure": outcome.failure.as_ref().map(|failure| format!("{failure:?}")),
                "mismatch_count": outcome.mismatches.len(),
                "compared_surface_count": outcome.compared_surfaces.len(),
                "reconstructed_commit_closure": outcome.reconstructed_commit_closure.len(),
                "lineage_authority_basis": outcome
                    .lineage_authority_basis
                    .as_ref()
                    .map(|basis: &crate::replay::data::ReplayLineageAuthorityBasis| format!("{:?}", basis.kind())),
                "phase_work": {
                    "replay_commit_work": replay_commit_work,
                },
                "counters": counters,
            }),
        }
    });
    assert_counted_budget(
        &durable_replay_samples,
        "durable replay should resolve against canonical lineage artifacts without mismatches",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && metrics["failure"].is_null()
                && metrics["mismatch_count"].as_u64() == Some(0)
                && metrics["lineage_authority_basis"].as_str() == Some("DurableLogCanonical")
                && counter_u64(metrics, "replay_lineage_authority_lookup_requests") == 1
        },
    );

    let checkpoint_recovery_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_for_medium(&medium);
        create_entity_outcome(execution, &runtime, "source");
        capture_and_file_checkpoint(execution, &runtime, &medium);
        let second = create_entity_outcome(execution, &runtime, "target");
        let tail_commit_id = second.commit.commit_id;
        assert!(medium.has_filed_checkpoint());
        assert_eq!(medium.committed_count(), 2);

        let mut recovered = durable_runtime_for_medium(&medium);
        recovered.performance_access().reset_counters();
        let (outcome, recovery_work) = measure_counted_work(execution, || {
            recover_runtime_from_medium(execution, &mut recovered, &medium)
        });
        let (replay, replay_commit_work) = measure_counted_work(execution, || {
            recovered
                .replay_authority()
                .replay_commit(
                    RelationalReplayRequest {
                        commit_id: tail_commit_id,
                        branch_id: BranchId("main".to_string()),
                        execution_mode: ReplayExecutionMode::SerialDeterministic,
                        verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                    },
                    execution,
                )
                .unwrap()
        });
        let counters = recovered.performance_access().counters();

        CountedPerfSample {
            charged_work: recovery_work + replay_commit_work,
            metrics: perf_metrics!({
                "recovered_commits": outcome.recovered_commits,
                "checkpoint_commits": outcome.coverage.checkpoint_commits,
                "replayed_tail_commits": outcome.coverage.replayed_tail_commits,
                "selected_checkpoint": outcome.coverage.checkpoint_covered_position.is_some(),
                "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                "replay_mismatch_count": replay.mismatches.len(),
                "phase_work": {
                    "recovery_work": recovery_work,
                    "replay_commit_work": replay_commit_work,
                },
                "counters": counters,
            }),
        }
    });
    assert!(checkpoint_recovery_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &checkpoint_recovery_samples,
        "checkpoint recovery plus suffix replay should recover commits cleanly without replay drift",
        |metrics| {
            metrics["recovered_commits"].as_u64().unwrap_or(0) >= 1
                && metrics["checkpoint_commits"].as_u64().unwrap_or(0) >= 1
                && metrics["replayed_tail_commits"].as_u64().unwrap_or(0) >= 1
                && metrics["selected_checkpoint"].as_bool() == Some(true)
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
        },
    );
}
