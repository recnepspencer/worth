use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_replay_recovery_matrix() {
    let suite = "replay_recovery_matrix";

    let durable_replay_samples = capture_perf_samples(
        suite,
        "durable_replay_lineage_basis",
        || {
            let runtime = persisted_runtime_with_test_schema();
            create_entity_outcome(&runtime, "source");
            let second = create_entity_outcome(&runtime, "target");
            let replay_commit_id = second.commit.commit_id;

            runtime.performance_access().reset_counters();
            let replay_started_at = Instant::now();
            let outcome = runtime
                .replay_authority()
                .replay_commit(RelationalReplayRequest {
                    commit_id: replay_commit_id,
                    branch_id: BranchId("main".to_string()),
                    execution_mode: ReplayExecutionMode::SerialDeterministic,
                    verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                });
            let replay_commit_micros = replay_started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros: replay_commit_micros,
                metrics: perf_metrics!({
                    "failure": outcome.failure.as_ref().map(|failure| format!("{failure:?}")),
                    "mismatch_count": outcome.mismatches.len(),
                    "compared_surface_count": outcome.compared_surfaces.len(),
                    "reconstructed_commit_closure": outcome.reconstructed_commit_closure.len(),
                    "lineage_authority_basis": outcome
                        .lineage_authority_basis
                        .as_ref()
                        .map(|basis: &crate::replay::data::ReplayLineageAuthorityBasis| format!("{:?}", basis.kind())),
                    "phase_timing": {
                        "replay_commit_micros": replay_commit_micros,
                    },
                    "counters": counters,
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "durable_replay_lineage_basis",
        &durable_replay_samples,
        &[(
            "replay_commit_micros",
            &["phase_timing", "replay_commit_micros"],
        )],
    );
    assert!(durable_replay_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
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

    let checkpoint_recovery_samples =
        capture_perf_samples(suite, "checkpoint_recover_suffix_replay", || {
            let runtime = persisted_runtime_with_test_schema();
            create_entity_outcome(&runtime, "source");
            runtime
                .durability_authority()
                .checkpoint()
                .expect("checkpoint");
            let second = create_entity_outcome(&runtime, "target");
            let tail_commit_id = second.commit.commit_id;

            let recovery_plan = runtime.durability().recovery_plan(
                crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
            );
            let mut recovered = persisted_runtime_with_test_schema();

            recovered.performance_access().reset_counters();
            let recovery_started_at = Instant::now();
            let outcome = recovered
                .durability_recovery()
                .recover(recovery_plan)
                .expect("recover plan");
            let recovery_micros = recovery_started_at.elapsed().as_micros();
            let replay_started_at = Instant::now();
            let replay = recovered
                .replay_authority()
                .replay_commit(RelationalReplayRequest {
                    commit_id: tail_commit_id,
                    branch_id: BranchId("main".to_string()),
                    execution_mode: ReplayExecutionMode::SerialDeterministic,
                    verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                });
            let replay_commit_micros = replay_started_at.elapsed().as_micros();
            let elapsed_micros = recovery_micros + replay_commit_micros;
            let counters = recovered.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "recovered_commits": outcome.recovered_commits,
                    "checkpoint_commits": outcome.coverage.checkpoint_commits,
                    "replayed_tail_commits": outcome.coverage.replayed_tail_commits,
                    "selected_checkpoint": outcome.cursor.checkpoint_id.is_some(),
                    "replay_failure": replay.failure.as_ref().map(|failure| format!("{failure:?}")),
                    "replay_mismatch_count": replay.mismatches.len(),
                    "phase_timing": {
                        "recovery_micros": recovery_micros,
                        "replay_commit_micros": replay_commit_micros,
                    },
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "checkpoint_recover_suffix_replay",
        &checkpoint_recovery_samples,
        &[
            ("recovery_micros", &["phase_timing", "recovery_micros"]),
            (
                "replay_commit_micros",
                &["phase_timing", "replay_commit_micros"],
            ),
        ],
    );
    assert!(checkpoint_recovery_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
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
