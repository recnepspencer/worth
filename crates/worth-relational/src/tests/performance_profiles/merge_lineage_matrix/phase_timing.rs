use super::*;

pub(super) fn certify_merge_execute_phase_timing_feature_adoption(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let merge_phase_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema();
        create_entity(execution, &runtime, "main-anchor");
        create_branch_from_main(&runtime, "feature");
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
            &runtime,
            BranchId("feature".to_string()),
        );
        txn.push_batch(
            WorkerIntentBatch::new("create-feature-only").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("feature-only"),
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "feature-only",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        let _feature_only =
            changed_entities(&txn.commit(execution, &runtime).expect("feature create"))[0];

        let prepared = runtime
            .prepare_merge_execution(
                execution,
                MergeExecutionRequest {
                    target_branch: BranchId("main".to_string()),
                    source_branch: BranchId("feature".to_string()),
                    merge_intent: MergeIntent::ReconcileIntoTarget,
                },
            )
            .expect("prepared merge");

        let log_before = runtime.durable_log().len();
        runtime.performance_access().reset_counters();
        let (outcome, execute_work) = measure_counted_work(execution, || {
            runtime
                .execute_prepared_merge(execution, prepared)
                .expect("execute merge")
        });
        let counters = runtime.performance_access().counters();
        let log_delta = runtime.durable_log().len().saturating_sub(log_before);
        let published = runtime.publication().latest_bundle().is_some();

        CountedPerfSample {
            charged_work: execute_work,
            metrics: perf_metrics!({
                "execute_work": execute_work,
                "executed_record_count": outcome.structural_summary.executed_record_count,
                "changed_entities": changed_entities(&outcome.commit).len(),
                "durable_log_delta": log_delta,
                "published_bundle_present": published,
                "counters": counters,
            }),
        }
    });
    assert!(merge_phase_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &merge_phase_samples,
        "merge execute phase budget should preserve single-record truth and counted commit phases",
        |metrics| {
            metrics["changed_entities"].as_u64() == Some(1)
                && metrics["executed_record_count"].as_u64() == Some(1)
                && metrics["execute_work"].as_u64().unwrap_or(0) > 0
                && metrics["durable_log_delta"].as_u64() == Some(1)
                && metrics["published_bundle_present"].as_bool() == Some(true)
                && metrics["counters"]["merge_execution_attempts"].as_u64() == Some(1)
        },
    );
}
