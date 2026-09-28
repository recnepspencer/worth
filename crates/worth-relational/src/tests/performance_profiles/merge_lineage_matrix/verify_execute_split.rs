use super::*;

pub(super) fn certify_merge_verify_vs_execute_feature_adoption(suite: &'static str) {
    let merge_verify_execute_split_samples = capture_perf_samples(
        suite,
        "merge_verify_vs_execute_feature_adoption",
        || {
            let runtime = persisted_runtime_with_test_schema();
            create_entity(&runtime, "main-anchor");
            create_branch_from_main(&runtime, "feature");
            let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
                &runtime,
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
            let _feature_only = changed_entities(&txn.commit(&runtime).expect("feature create"))[0];

            let prepared = runtime
                .prepare_merge_execution(MergeExecutionRequest {
                    target_branch: BranchId("main".to_string()),
                    source_branch: BranchId("feature".to_string()),
                    merge_intent: MergeIntent::ReconcileIntoTarget,
                })
                .expect("prepared merge");

            runtime.performance_access().reset_counters();
            let verify_started_at = Instant::now();
            runtime
                .merge()
                .verify_prepared_merge_execution(&prepared)
                .expect("verify prepared merge");
            let verify_elapsed_micros = verify_started_at.elapsed().as_micros();
            let verify_counters = runtime.performance_access().counters();

            runtime.performance_access().reset_counters();
            let execute_started_at = Instant::now();
            let outcome = runtime
                .execute_prepared_merge(prepared)
                .expect("execute merge");
            let execute_elapsed_micros = execute_started_at.elapsed().as_micros();
            let execute_counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros: verify_elapsed_micros + execute_elapsed_micros,
                metrics: perf_metrics!({
                    "verify_elapsed_micros": verify_elapsed_micros,
                    "execute_elapsed_micros": execute_elapsed_micros,
                    "executed_record_count": outcome.structural_summary.executed_record_count,
                    "emitted_mutation_intent_count": outcome.structural_summary.emitted_mutation_intent_count,
                    "changed_entities": changed_entities(&outcome.commit).len(),
                    "verify_counters": verify_counters,
                    "execute_counters": execute_counters,
                }),
            }
        },
    );
    assert!(merge_verify_execute_split_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &merge_verify_execute_split_samples,
        "merge verify/execute split should show certified verification and preserve single-record execute truth",
        |metrics| {
            metrics["verify_elapsed_micros"].as_u64().unwrap_or(0) > 0
                && metrics["execute_elapsed_micros"].as_u64().unwrap_or(0) > 0
                && metrics["changed_entities"].as_u64() == Some(1)
                && metrics["verify_counters"]["merge_execution_verification_requests"].as_u64()
                    == Some(1)
                && metrics["verify_counters"]["merge_execution_branch_head_checks"].as_u64()
                    == Some(2)
                && metrics["verify_counters"]["merge_execution_merge_base_checks"].as_u64()
                    == Some(1)
                && metrics["verify_counters"]["merge_execution_compiled_plan_digest_checks"]
                    .as_u64()
                    == Some(1)
                && metrics["execute_counters"]["merge_execution_attempts"].as_u64() == Some(1)
                && metrics["execute_counters"]["merge_execution_records_admitted"].as_u64()
                    == metrics["executed_record_count"].as_u64()
                && metrics["execute_counters"]["merge_execution_mutation_intents_emitted"].as_u64()
                    == metrics["emitted_mutation_intent_count"].as_u64()
        },
    );
}
