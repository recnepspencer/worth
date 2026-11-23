use super::*;

pub(super) fn certify_retention_release_reclaim_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let retention_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema();
        let survivor = create_entity_in_partition(
            execution,
            &runtime,
            "retention-survivor",
            PartitionId(10),
        );
        let deleted_created = create_entity_outcome(execution, &runtime, "retention-deleted");
        let created_snapshot = runtime
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");
        let deleted_entity = changed_entities(&deleted_created)[0];
        let deleted_commit = delete_entity(execution, &runtime, deleted_entity);
        let deleted_snapshot = runtime
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");

        assert!(runtime
            .visibility_authority()
            .release_snapshot(&created_snapshot)
            .is_ok());
        assert!(runtime
            .visibility_authority()
            .release_snapshot(&deleted_snapshot)
            .is_ok());

        runtime.performance_access().reset_counters();
        let (inspect_plan, inspect_plan_work) = measure_counted_work(execution, || {
            runtime.retention().inspect_plan(execution).unwrap()
        });
        let (reclaim_pass, run_pass_work) = measure_counted_work(execution, || {
            runtime.retention().run_pass(execution).unwrap()
        });

        let snapshot = runtime
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");
        let packet = explicit_query_packet(
            &runtime,
            &snapshot,
            "retention-reclaim-round-trip",
            vec![
                RecordRef::Entity(survivor),
                RecordRef::Entity(deleted_entity),
            ],
        );
        let (query_outcome, post_reclaim_query_work) = measure_counted_work(execution, || {
            runtime
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
                .expect("retention workflow query")
        });

        let counters = runtime.performance_access().counters();
        CountedPerfSample {
            charged_work: inspect_plan_work + run_pass_work + post_reclaim_query_work,
            metrics: perf_metrics!({
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
                "phase_work": {
                    "inspect_plan_work": inspect_plan_work,
                    "run_pass_work": run_pass_work,
                    "post_reclaim_query_work": post_reclaim_query_work,
                },
                "counters": counters,
            }),
        }
    });
    assert_counted_budget(
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
