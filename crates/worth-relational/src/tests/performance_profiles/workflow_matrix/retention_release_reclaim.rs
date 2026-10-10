use super::*;

pub(super) fn certify_retention_release_reclaim_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let retention_samples =
        capture_perf_samples(suite, "retention_release_reclaim_round_trip", || {
            let runtime = runtime_with_test_schema();
            let survivor =
                create_entity_in_partition(&runtime, "retention-survivor", PartitionId(10));
            let deleted_created = create_entity_outcome(&runtime, "retention-deleted");
            let created_snapshot = runtime.visibility_authority().snapshot();
            let deleted_entity = changed_entities(&deleted_created)[0];
            let deleted_commit = delete_entity(&runtime, deleted_entity);
            let deleted_snapshot = runtime.visibility_authority().snapshot();

            assert!(runtime
                .visibility_authority()
                .release_snapshot(&created_snapshot)
                .is_ok());
            assert!(runtime
                .visibility_authority()
                .release_snapshot(&deleted_snapshot)
                .is_ok());

            runtime.performance_access().reset_counters();
            let inspect_started_at = Instant::now();
            let inspect_plan = runtime.retention().inspect_plan();
            let inspect_plan_micros = inspect_started_at.elapsed().as_micros();
            let reclaim_started_at = Instant::now();
            let reclaim_pass = runtime.retention().run_pass();
            let run_pass_micros = reclaim_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "retention-reclaim-round-trip",
                vec![
                    RecordRef::Entity(survivor),
                    RecordRef::Entity(deleted_entity),
                ],
            );
            let query_started_at = Instant::now();
            let query_outcome = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, packet)
                        .expect("planned retention workflow query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("retention workflow query");
            let post_reclaim_query_micros = query_started_at.elapsed().as_micros();

            let elapsed_micros = inspect_plan_micros + run_pass_micros + post_reclaim_query_micros;
            let counters = runtime.performance_access().counters();

            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "deleted_commit_records": deleted_commit.changed_records.len(),
                    "active_snapshot_count": inspect_plan.active_snapshot_count,
                    "reclaimable_entities": inspect_plan.reclaimable_entities,
                    "entity_reclaimable": reclaim_pass.entity_reclaimable,
                    "entity_reclaimed": reclaim_pass.entity_reclaimed,
                    "query_entities": query_outcome.result.entities.len(),
                    "query_relations": query_outcome.result.relations.len(),
                    "profile_boundary": profile_boundary_metrics(
                        &runtime,
                        RelationalRuntimeProfile::CertificationCore,
                    ),
                    "phase_timing": {
                        "inspect_plan_micros": inspect_plan_micros,
                        "run_pass_micros": run_pass_micros,
                        "post_reclaim_query_micros": post_reclaim_query_micros,
                    },
                    "counters": counters,
                })
            })
        });
    emit_metric_summaries(
        suite,
        "retention_release_reclaim_round_trip",
        &retention_samples,
        &[
            (
                "inspect_plan_micros",
                &["phase_timing", "inspect_plan_micros"],
            ),
            ("run_pass_micros", &["phase_timing", "run_pass_micros"]),
            (
                "post_reclaim_query_micros",
                &["phase_timing", "post_reclaim_query_micros"],
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
    assert!(retention_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &retention_samples,
        "retention release round trips should expose reclaimability and keep the survivor queryable without clone-heavy reclaim work",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && metrics["deleted_commit_records"].as_u64() == Some(1)
                && metrics["active_snapshot_count"].as_u64() == Some(0)
                && metrics["reclaimable_entities"].as_u64().unwrap_or(0) >= 1
                && metrics["entity_reclaimable"].as_u64().unwrap_or(0) >= 1
                && metrics["entity_reclaimed"].as_u64().unwrap_or(0)
                    <= metrics["entity_reclaimable"].as_u64().unwrap_or(0)
                && counter_u64(metrics, "query_packet_count") <= 2
                && metrics["query_entities"].as_u64() == Some(1)
                && metrics["query_relations"].as_u64() == Some(0)
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(1)
        },
    );
}
