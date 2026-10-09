use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_retention_reclaim_matrix() {
    let suite = "retention_reclaim_matrix";

    let snapshot_pin_samples =
        capture_perf_samples(suite, "snapshot_release_to_reclaimable_entity", || {
            let runtime = runtime_with_test_schema();
            let created = create_entity_outcome(&runtime, "retained");
            let created_snapshot = runtime.visibility_authority().snapshot();
            let entity = changed_entities(&created)[0];
            let _deleted = delete_entity(&runtime, entity);
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
            let plan = runtime.retention().inspect_plan();
            let inspect_plan_micros = inspect_started_at.elapsed().as_micros();
            let pass_started_at = Instant::now();
            let pass = runtime.retention().run_pass();
            let run_pass_micros = pass_started_at.elapsed().as_micros();
            let elapsed_micros = inspect_plan_micros + run_pass_micros;
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "active_snapshot_count": plan.active_snapshot_count,
                    "reclaimable_entities": plan.reclaimable_entities,
                    "entity_reclaimable": pass.entity_reclaimable,
                    "entity_reclaimed": pass.entity_reclaimed,
                    "phase_timing": {
                        "inspect_plan_micros": inspect_plan_micros,
                        "run_pass_micros": run_pass_micros,
                    },
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "snapshot_release_to_reclaimable_entity",
        &snapshot_pin_samples,
        &[
            (
                "inspect_plan_micros",
                &["phase_timing", "inspect_plan_micros"],
            ),
            ("run_pass_micros", &["phase_timing", "run_pass_micros"]),
        ],
    );
    assert!(snapshot_pin_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &snapshot_pin_samples,
        "retention reclaim should become reclaimable after snapshot release without full rebuilds",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && metrics["active_snapshot_count"].as_u64() == Some(0)
                && metrics["reclaimable_entities"].as_u64().unwrap_or(0) >= 1
                && metrics["entity_reclaimable"].as_u64().unwrap_or(0) >= 1
        },
    );

    let replay_pin_samples =
        capture_perf_samples(suite, "replay_pin_release_deleted_relation", || {
            let runtime = runtime_with_test_schema();
            let source = create_entity(&runtime, "replay-left");
            let target = create_entity(&runtime, "replay-right");
            let created = create_relation_outcome(&runtime, source, target, "replay-r1");
            let relation = changed_relations(&created)[0];
            let deleted = {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
                txn.push_batch(
                    WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
                        RelationMutationIntent::Delete(DeleteRelationIntent {
                            relation_id: relation,
                        }),
                    )),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("test staging stays within configured resource budgets");
                txn.commit(
                    &runtime,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("delete relation")
            };

            assert!(runtime
                .visibility_authority()
                .release_snapshot(&created.snapshot)
                .is_ok());
            assert!(runtime
                .visibility_authority()
                .release_snapshot(&deleted.snapshot)
                .is_ok());
            assert!(runtime
                .history_authority()
                .retain_version_for_replay(created.version_id));

            runtime.performance_access().reset_counters();
            let inspect_started_at = Instant::now();
            let pinned = runtime.retention().inspect_plan();
            let inspect_pinned_micros = inspect_started_at.elapsed().as_micros();
            let release_started_at = Instant::now();
            assert!(runtime
                .history_authority()
                .release_version_replay_retention(created.version_id));
            let release_replay_pin_micros = release_started_at.elapsed().as_micros();
            let inspect_released_started_at = Instant::now();
            let released = runtime.retention().inspect_plan();
            let inspect_released_micros = inspect_released_started_at.elapsed().as_micros();
            let elapsed_micros =
                inspect_pinned_micros + release_replay_pin_micros + inspect_released_micros;
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "pinned_replay_relations": pinned.replay_pinned_relations,
                    "pinned_reclaimable_relations": pinned.reclaimable_relations,
                    "released_branch_pinned_relations": released.branch_pinned_relations,
                    "released_replay_relations": released.replay_pinned_relations,
                    "released_reclaimable_relations": released.reclaimable_relations,
                    "phase_timing": {
                        "inspect_pinned_micros": inspect_pinned_micros,
                        "release_replay_pin_micros": release_replay_pin_micros,
                        "inspect_released_micros": inspect_released_micros,
                    },
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "replay_pin_release_deleted_relation",
        &replay_pin_samples,
        &[
            (
                "inspect_pinned_micros",
                &["phase_timing", "inspect_pinned_micros"],
            ),
            (
                "release_replay_pin_micros",
                &["phase_timing", "release_replay_pin_micros"],
            ),
            (
                "inspect_released_micros",
                &["phase_timing", "inspect_released_micros"],
            ),
        ],
    );
    assert!(replay_pin_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &replay_pin_samples,
        "replay retention should pin relations until release and then expose reclaimability",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && metrics["pinned_replay_relations"].as_u64().unwrap_or(0) >= 1
                && metrics["pinned_reclaimable_relations"].as_u64() == Some(0)
                && metrics["released_replay_relations"].as_u64() == Some(0)
                && metrics["released_branch_pinned_relations"]
                    .as_u64()
                    .unwrap_or(0)
                    >= 1
                && metrics["released_reclaimable_relations"].as_u64() == Some(0)
        },
    );
}
