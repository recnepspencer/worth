use super::*;

pub(super) fn certify_merge_execution_feature_adoption(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let merge_execution_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_for_medium(&medium);
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
                MergeExecutionRequest::new(
                    BranchId("main".to_string()),
                    BranchId("feature".to_string()),
                    MergeIntent::ReconcileIntoTarget,
                ),
            )
            .expect("prepared merge");

        runtime.performance_access().reset_counters();
        let (outcome, charged_work) = measure_counted_work(execution, || {
            runtime
                .execute_prepared_merge(execution, prepared)
                .expect("execute merge")
        });
        assert_eq!(medium.committed_count(), 3);
        assert_eq!(
            medium
                .latest_committed_artifact()
                .expect("durable merge artifact")
                .commit_id(),
            outcome.commit.commit.commit_id
        );
        let counters = runtime.performance_access().counters();

        CountedPerfSample {
            charged_work,
            metrics: perf_metrics!({
                "charged_work": charged_work,
                "executed_record_count": outcome.structural_summary.executed_record_count,
                "emitted_mutation_intent_count": outcome.structural_summary.emitted_mutation_intent_count,
                "adopted_source_record_count": outcome.structural_summary.adopted_source_record_count,
                "changed_entities": changed_entities(&outcome.commit).len(),
                "counters": counters,
            }),
        }
    });
    assert!(merge_execution_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &merge_execution_samples,
        "merge execution should admit and emit exactly the scoped merge work",
        |metrics| {
            counter_u64(metrics, "merge_execution_attempts") == 1
                && counter_u64(metrics, "merge_execution_records_admitted")
                    == metrics["executed_record_count"].as_u64().unwrap_or(0)
                && counter_u64(metrics, "merge_execution_mutation_intents_emitted")
                    == metrics["emitted_mutation_intent_count"]
                        .as_u64()
                        .unwrap_or(0)
                && metrics["adopted_source_record_count"].as_u64() == Some(1)
                && metrics["changed_entities"].as_u64() == Some(1)
        },
    );
}
