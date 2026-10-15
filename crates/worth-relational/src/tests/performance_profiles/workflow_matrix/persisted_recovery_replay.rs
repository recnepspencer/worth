use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

pub(super) fn certify_persisted_recovery_replay_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let replay_recovery_samples = capture_perf_samples(
        suite,
        "persisted_recovery_replay_round_trip",
        || {
            let runtime = persisted_runtime_with_test_schema();
            let source_created = create_entity_outcome(execution, &runtime, "recovery-source");
            let source = changed_entities(&source_created)[0];

            let checkpoint_started_at = Instant::now();
            runtime
                .durability_authority()
                .checkpoint(execution)
                .expect("workflow checkpoint");
            let checkpoint_micros = checkpoint_started_at.elapsed().as_micros();

            let post_checkpoint_commit_started_at = Instant::now();
            let target_created = create_entity_outcome(execution, &runtime, "recovery-target");
            let target = changed_entities(&target_created)[0];
            let post_checkpoint_commit_micros =
                post_checkpoint_commit_started_at.elapsed().as_micros();
            let recovery_plan = runtime
                .durability()
                .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
                .unwrap();
            let tail_commit_id = target_created.commit.commit_id;

            let mut recovered = persisted_runtime_with_test_schema();
            recovered.performance_access().reset_counters();
            let recover_started_at = Instant::now();
            let recovery_outcome = recovered
                .durability_recovery()
                .recover(execution, recovery_plan)
                .expect("workflow recovery");
            let recover_micros = recover_started_at.elapsed().as_micros();

            let replay_started_at = Instant::now();
            let replay_outcome = recovered
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
                .unwrap();
            let replay_commit_micros = replay_started_at.elapsed().as_micros();

            let recovered_snapshot = recovered.visibility_authority().snapshot();
            let recovered_packet = explicit_query_packet(
                &recovered,
                &recovered_snapshot,
                "recovery-round-trip-query",
                vec![RecordRef::Entity(source), RecordRef::Entity(target)],
            );
            let query_started_at = Instant::now();
            let query_outcome = recovered
                .read_truth()
                .execute_query_plan(
                    recovered
                        .read_truth()
                        .plan_query_packet(&recovered_snapshot, recovered_packet)
                        .expect("planned recovered workflow query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("recovered workflow query");
            let post_recovery_query_micros = query_started_at.elapsed().as_micros();

            let elapsed_micros = checkpoint_micros
                + post_checkpoint_commit_micros
                + recover_micros
                + replay_commit_micros
                + post_recovery_query_micros;
            let counters = recovered.performance_access().counters();

            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "checkpoint_commit_count": recovery_outcome.coverage.checkpoint_commits,
                    "tail_commit_count": recovery_outcome.coverage.replayed_tail_commits,
                    "recovered_commits": recovery_outcome.recovered_commits,
                    "selected_checkpoint": recovery_outcome.cursor.checkpoint_id.is_some(),
                    "replay_failure": replay_outcome.failure.as_ref().map(|failure| format!("{failure:?}")),
                    "replay_mismatch_count": replay_outcome.mismatches.len(),
                    "query_entities": query_outcome.result.entities.len(),
                    "query_relations": query_outcome.result.relations.len(),
                    "profile_boundary": profile_boundary_metrics(
                        &recovered,
                        RelationalRuntimeProfile::CertificationCore,
                    ),
                    "phase_timing": {
                        "checkpoint_micros": checkpoint_micros,
                        "post_checkpoint_commit_micros": post_checkpoint_commit_micros,
                        "recover_micros": recover_micros,
                        "replay_commit_micros": replay_commit_micros,
                        "post_recovery_query_micros": post_recovery_query_micros,
                    },
                    "counters": counters,
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "persisted_recovery_replay_round_trip",
        &replay_recovery_samples,
        &[
            ("checkpoint_micros", &["phase_timing", "checkpoint_micros"]),
            (
                "post_checkpoint_commit_micros",
                &["phase_timing", "post_checkpoint_commit_micros"],
            ),
            ("recover_micros", &["phase_timing", "recover_micros"]),
            (
                "replay_commit_micros",
                &["phase_timing", "replay_commit_micros"],
            ),
            (
                "post_recovery_query_micros",
                &["phase_timing", "post_recovery_query_micros"],
            ),
            (
                "profile_execution_lane_code",
                &["profile_boundary", "execution_lane_code"],
            ),
            (
                "profile_diagnostics_boundary_code",
                &["profile_boundary", "diagnostics_boundary_code"],
            ),
            (
                "profile_matches_defaults",
                &["profile_boundary", "matches_defaults"],
            ),
        ],
    );
    assert!(replay_recovery_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &replay_recovery_samples,
        "persisted recovery round trips should select a checkpoint, replay cleanly, and query the recovered tail surface",
        |metrics| {
            metrics["selected_checkpoint"].as_bool() == Some(true)
                && metrics["replay_failure"].is_null()
                && metrics["replay_mismatch_count"].as_u64() == Some(0)
                && metrics["checkpoint_commit_count"].as_u64().unwrap_or(0) >= 1
                && metrics["tail_commit_count"].as_u64().unwrap_or(0) >= 1
                && counter_u64(metrics, "replay_lineage_authority_lookup_requests") == 1
                && counter_u64(metrics, "query_packet_count") <= 3
                && metrics["query_entities"].as_u64() == Some(2)
                && metrics["query_relations"].as_u64() == Some(0)
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(1)
        },
    );
}
