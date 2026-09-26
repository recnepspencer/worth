use super::*;

pub(super) fn certify_merge_execution_vs_persisted_commit_floor(suite: &'static str) {
    let merge_vs_commit_floor_samples = capture_perf_samples(
        suite,
        "merge_execution_vs_persisted_commit_floor",
        || {
            let merge_runtime = persisted_runtime_with_test_schema();
            create_entity(&merge_runtime, "main-anchor");
            create_branch_from_main(&merge_runtime, "feature");
            let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
                &merge_runtime,
                BranchId("feature".to_string()),
            );
            txn.push_batch(WorkerIntentBatch::new("create-feature-only").push(
                MutationIntent::Create(CreateIntent::Entity(
                    crate::transactions::data::EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("feature-only"),
                        fields: crate::tests::support::single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "feature-only",
                        ),
                    },
                )),
            ))
            .expect("test staging stays within configured resource budgets");
            let _feature_only =
                changed_entities(&txn.commit(&merge_runtime).expect("feature create"))[0];

            let prepared = merge_runtime
                .prepare_merge_execution(MergeExecutionRequest {
                    target_branch: BranchId("main".to_string()),
                    source_branch: BranchId("feature".to_string()),
                    merge_intent: MergeIntent::ReconcileIntoTarget,
                })
                .expect("prepared merge");

            merge_runtime.performance_access().reset_counters();
            let merge_started_at = Instant::now();
            let merge_outcome = merge_runtime
                .execute_prepared_merge(prepared)
                .expect("execute merge");
            let merge_elapsed_micros = merge_started_at.elapsed().as_micros();
            let merge_counters = merge_runtime.performance_access().counters();

            let control_runtime = persisted_runtime_with_test_schema();
            control_runtime.performance_access().reset_counters();
            let control_started_at = Instant::now();
            let control_outcome = {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(&control_runtime);
                txn.push_batch(batch_create("control-single"))
                    .expect("test staging stays within configured resource budgets");
                txn.commit(&control_runtime)
                    .expect("control persisted single create")
            };
            let control_elapsed_micros = control_started_at.elapsed().as_micros();
            let control_counters = control_runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros: merge_elapsed_micros,
                metrics: perf_metrics!({
                    "merge_elapsed_micros": merge_elapsed_micros,
                    "control_commit_elapsed_micros": control_elapsed_micros,
                    "merge_over_control_delta_micros": merge_elapsed_micros as i128 - control_elapsed_micros as i128,
                    "merge_control_ratio": merge_elapsed_micros as f64 / control_elapsed_micros.max(1) as f64,
                    "executed_record_count": merge_outcome.structural_summary.executed_record_count,
                    "emitted_mutation_intent_count": merge_outcome.structural_summary.emitted_mutation_intent_count,
                    "merge_changed_entities": changed_entities(&merge_outcome.commit).len(),
                    "control_changed_records": control_outcome.changed_records.len(),
                    "merge_counters": merge_counters,
                    "control_counters": control_counters,
                }),
            }
        },
    );
    assert!(merge_vs_commit_floor_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &merge_vs_commit_floor_samples,
        "merge execute vs persisted commit floor should preserve single-record structural truth on both paths",
        |metrics| {
            metrics["merge_elapsed_micros"].as_u64().unwrap_or(0) > 0
                && metrics["control_commit_elapsed_micros"].as_u64().unwrap_or(0) > 0
                && metrics["merge_changed_entities"].as_u64() == Some(1)
                && metrics["control_changed_records"].as_u64() == Some(1)
                && metrics["merge_counters"]["merge_execution_attempts"].as_u64() == Some(1)
                && metrics["merge_counters"]["merge_execution_records_admitted"].as_u64()
                    == metrics["executed_record_count"].as_u64()
                && metrics["merge_counters"]["merge_execution_mutation_intents_emitted"].as_u64()
                    == metrics["emitted_mutation_intent_count"].as_u64()
                && metrics["control_counters"]["full_state_clones"].as_u64() == Some(0)
                && metrics["control_counters"]["snapshot_pin_full_rebuilds"].as_u64() == Some(0)
                && metrics["control_counters"]["partitions_touched_by_commit"].as_u64() == Some(1)
        },
    );
}
