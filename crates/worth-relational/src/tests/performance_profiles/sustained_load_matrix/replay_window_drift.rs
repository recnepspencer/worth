use super::*;

pub(super) fn certify_replay_window_drift_stability(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let replay_window_drift_samples =
        capture_perf_samples(suite, "replay_window_drift_stability", || {
            let runtime = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::CertificationCore,
            );
            const HISTORY_DEPTH: usize = 48;
            const REPLAY_WINDOW: usize = 32;
            let mut commit_ids = Vec::with_capacity(HISTORY_DEPTH);
            for index in 0..HISTORY_DEPTH {
                let outcome =
                    create_entity_outcome(execution, &runtime, &format!("replay-window-{index}"));
                commit_ids.push(outcome.commit.commit_id);
            }

            runtime.performance_access().reset_counters();
            let mut total_replay_micros = 0u128;
            let mut max_replay_micros = 0u128;
            let mut total_compared_surface_count = 0usize;
            let mut total_reconstructed_commit_closure = 0usize;
            let mut total_mismatch_count = 0usize;
            let mut replayed_commit_count = 0usize;

            for commit_id in commit_ids.iter().rev().take(REPLAY_WINDOW) {
                let replay_started_at = Instant::now();
                let outcome = runtime
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
                    .unwrap();
                let replay_micros = replay_started_at.elapsed().as_micros();
                assert!(
                    outcome.failure.is_none(),
                    "replay window drift sample should not fail: {:?}",
                    outcome.failure
                );
                total_replay_micros += replay_micros;
                max_replay_micros = max_replay_micros.max(replay_micros);
                total_compared_surface_count += outcome.compared_surfaces.len();
                total_reconstructed_commit_closure += outcome.reconstructed_commit_closure.len();
                total_mismatch_count += outcome.mismatches.len();
                replayed_commit_count += 1;
            }

            measurement_with_elapsed(total_replay_micros, || {
                perf_metrics!({
                    "history_depth": HISTORY_DEPTH,
                    "replay_window": REPLAY_WINDOW,
                    "average_replay_micros": total_replay_micros / REPLAY_WINDOW as u128,
                    "max_replay_micros": max_replay_micros,
                    "replayed_commit_count": replayed_commit_count,
                    "total_compared_surface_count": total_compared_surface_count,
                    "total_reconstructed_commit_closure": total_reconstructed_commit_closure,
                    "total_mismatch_count": total_mismatch_count,
                    "counters": runtime.performance_access().counters(),
                })
            })
        });
    emit_metric_summaries(
        suite,
        "replay_window_drift_stability",
        &replay_window_drift_samples,
        &[
            ("average_replay_micros", &["average_replay_micros"]),
            ("max_replay_micros", &["max_replay_micros"]),
            ("replayed_commit_count", &["replayed_commit_count"]),
            (
                "total_compared_surface_count",
                &["total_compared_surface_count"],
            ),
            (
                "total_reconstructed_commit_closure",
                &["total_reconstructed_commit_closure"],
            ),
        ],
    );
    assert!(replay_window_drift_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
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
