use super::*;

pub(super) fn certify_merge_execution_zero_diagnostics_budget(suite: &'static str) {
    let merge_execution_zero_diag_samples = capture_perf_samples(
        suite,
        "merge_execution_feature_adoption_zero_diagnostics_budget",
        || {
            let mut runtime = persisted_runtime_with_test_schema();
            runtime.configure_diagnostics_for_test(|profile| {
                profile.max_entries_per_artifact = 0;
            });
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
            let started_at = Instant::now();
            let outcome = runtime
                .execute_prepared_merge(prepared)
                .expect("execute merge");
            let elapsed_micros = started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "executed_record_count": outcome.structural_summary.executed_record_count,
                    "emitted_mutation_intent_count": outcome.structural_summary.emitted_mutation_intent_count,
                    "adopted_source_record_count": outcome.structural_summary.adopted_source_record_count,
                    "changed_entities": changed_entities(&outcome.commit).len(),
                    "diagnostic_artifact_entries": runtime
                        .publication()
                        .diagnostics()
                        .artifacts()
                        .iter()
                        .rev()
                        .find(|artifact| {
                            artifact.scope == crate::facade::diagnostics::DiagnosticsScope::History
                                && artifact.kind
                                    == crate::facade::diagnostics::DiagnosticsArtifactKind::DetailedTrace
                        })
                        .map(|artifact| artifact.entries.len())
                        .unwrap_or(0),
                    "counters": counters,
                }),
            }
        },
    );
    assert!(merge_execution_zero_diag_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &merge_execution_zero_diag_samples,
        "merge execution should preserve structural truth even when detailed diagnostics are budget-zero",
        |metrics| {
            counter_u64(metrics, "merge_execution_attempts") == 1
                && counter_u64(metrics, "merge_execution_records_admitted")
                    == metrics["executed_record_count"].as_u64().unwrap_or(0)
                && counter_u64(metrics, "merge_execution_mutation_intents_emitted")
                    == metrics["emitted_mutation_intent_count"].as_u64().unwrap_or(0)
                && metrics["adopted_source_record_count"].as_u64() == Some(1)
                && metrics["changed_entities"].as_u64() == Some(1)
                && metrics["diagnostic_artifact_entries"].as_u64() == Some(0)
        },
    );
}
