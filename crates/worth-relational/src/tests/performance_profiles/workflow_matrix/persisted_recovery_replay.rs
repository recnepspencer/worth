use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

pub(super) fn certify_persisted_recovery_replay_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let replay_recovery_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema();
        let source_created = create_entity_outcome(execution, &runtime, "recovery-source");
        let source = changed_entities(&source_created)[0];

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            runtime
                .durability_authority()
                .checkpoint(execution)
                .expect("workflow checkpoint")
        });

        let (target_created, post_checkpoint_commit_work) = measure_counted_work(execution, || {
            create_entity_outcome(execution, &runtime, "recovery-target")
        });
        let target = changed_entities(&target_created)[0];
        let recovery_plan = runtime
            .durability()
            .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
            .unwrap();
        let tail_commit_id = target_created.commit.commit_id;

        let mut recovered = runtime_with_test_schema();
        recovered.performance_access().reset_counters();
        let (recovery_outcome, recover_work) = measure_counted_work(execution, || {
            recovered
                .durability_recovery()
                .recover(execution, recovery_plan)
                .expect("workflow recovery")
        });

        let (replay_outcome, replay_commit_work) = measure_counted_work(execution, || {
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

        let recovered_snapshot = recovered
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");
        let recovered_packet = explicit_query_packet(
            &recovered,
            &recovered_snapshot,
            "recovery-round-trip-query",
            vec![RecordRef::Entity(source), RecordRef::Entity(target)],
        );
        let (query_outcome, post_recovery_query_work) = measure_counted_work(execution, || {
            recovered
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
                .expect("recovered workflow query")
        });

        let counters = recovered.performance_access().counters();
        CountedPerfSample {
            charged_work: checkpoint_work
                + post_checkpoint_commit_work
                + recover_work
                + replay_commit_work
                + post_recovery_query_work,
            metrics: perf_metrics!({
                "checkpoint_commit_count": recovery_outcome.coverage.checkpoint_commits,
                "tail_commit_count": recovery_outcome.coverage.replayed_tail_commits,
                "recovered_commits": recovery_outcome.recovered_commits,
                "selected_checkpoint": recovery_outcome
                    .coverage
                    .checkpoint_covered_position
                    .is_some(),
                "replay_failure": replay_outcome.failure.as_ref().map(|failure| format!("{failure:?}")),
                "replay_mismatch_count": replay_outcome.mismatches.len(),
                "query_entities": query_outcome.result.entities.len(),
                "query_relations": query_outcome.result.relations.len(),
                "profile_boundary": profile_boundary_metrics(
                    &recovered,
                    RelationalRuntimeProfile::CertificationCore,
                ),
                "phase_work": {
                    "checkpoint_work": checkpoint_work,
                    "post_checkpoint_commit_work": post_checkpoint_commit_work,
                    "recover_work": recover_work,
                    "replay_commit_work": replay_commit_work,
                    "post_recovery_query_work": post_recovery_query_work,
                },
                "counters": counters,
            }),
        }
    });
    assert!(replay_recovery_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
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
