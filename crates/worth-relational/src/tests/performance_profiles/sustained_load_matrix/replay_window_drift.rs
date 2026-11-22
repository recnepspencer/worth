use super::*;

pub(super) fn certify_replay_window_drift_stability(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let replay_window_drift_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_profile_for_medium(
            RelationalRuntimeProfile::CertificationCore,
            &medium,
        );
        const HISTORY_DEPTH: usize = 48;
        const REPLAY_WINDOW: usize = 32;
        let mut commit_ids = Vec::with_capacity(HISTORY_DEPTH);
        for index in 0..HISTORY_DEPTH {
            let outcome =
                create_entity_outcome(execution, &runtime, &format!("replay-window-{index}"));
            commit_ids.push(outcome.commit.commit_id);
        }
        assert_eq!(medium.committed_count(), HISTORY_DEPTH);

        runtime.performance_access().reset_counters();
        let mut total_replay_work = 0u64;
        let mut total_compared_surface_count = 0usize;
        let mut total_reconstructed_commit_closure = 0usize;
        let mut total_mismatch_count = 0usize;
        let mut replayed_commit_count = 0usize;

        for commit_id in commit_ids.iter().rev().take(REPLAY_WINDOW) {
            let (outcome, replay_work) = measure_counted_work(execution, || {
                runtime
                    .replay_authority()
                    .replay_commit(
                        RelationalReplayRequest {
                            commit_id: *commit_id,
                            branch_id: BranchId("main".to_string()),
                            execution_mode: ReplayExecutionMode::SerialDeterministic,
                            verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                        },
                        execution,
                    )
                    .unwrap()
            });
            assert!(
                outcome.failure.is_none(),
                "replay window drift sample should not fail: {:?}",
                outcome.failure
            );
            total_replay_work += replay_work;
            total_compared_surface_count += outcome.compared_surfaces.len();
            total_reconstructed_commit_closure += outcome.reconstructed_commit_closure.len();
            total_mismatch_count += outcome.mismatches.len();
            replayed_commit_count += 1;
        }

        CountedPerfSample {
            charged_work: total_replay_work,
            metrics: perf_metrics!({
                "history_depth": HISTORY_DEPTH,
                "replay_window": REPLAY_WINDOW,
                "total_replay_work": total_replay_work,
                "replayed_commit_count": replayed_commit_count,
                "total_compared_surface_count": total_compared_surface_count,
                "total_reconstructed_commit_closure": total_reconstructed_commit_closure,
                "total_mismatch_count": total_mismatch_count,
                "counters": runtime.performance_access().counters(),
            }),
        }
    });
    assert_counted_budget(
        &replay_window_drift_samples,
        "replay drift windows should stay mismatch-free while replaying a bounded recent history slice",
        |metrics| {
            metrics["history_depth"].as_u64() == Some(48)
                && metrics["replay_window"].as_u64() == Some(32)
                && metrics["replayed_commit_count"].as_u64() == Some(32)
                && metrics["total_mismatch_count"].as_u64() == Some(0)
                && metrics["total_compared_surface_count"].as_u64().unwrap_or(0) >= 32
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "replay_lineage_authority_lookup_requests") == 32
        },
    );
}
