use super::*;

pub(super) fn certify_merge_execute_phase_timing_feature_adoption(suite: &'static str) {
    let merge_phase_timing_samples = capture_perf_samples(
        suite,
        "merge_execute_phase_timing_feature_adoption",
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
            let started_at = Instant::now();
            let outcome = runtime
                .execute_prepared_merge(prepared)
                .expect("execute merge");
            let elapsed_micros = started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();
            let phase_timing = outcome.commit.execution().phase_timing.clone();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "executed_record_count": outcome.structural_summary.executed_record_count,
                    "changed_entities": changed_entities(&outcome.commit).len(),
                    "phase_timing": {
                        "working_state_preparation_micros": phase_timing.working_state_preparation_micros,
                        "invariant_pre_check_micros": phase_timing.invariant_pre_check_micros,
                        "authoritative_mutation_micros": phase_timing.authoritative_mutation_micros,
                        "history_resolution_micros": phase_timing.history_resolution_micros,
                        "invariant_post_check_micros": phase_timing.invariant_post_check_micros,
                        "artifact_assembly_micros": phase_timing.artifact_assembly_micros,
                        "durable_append_micros": phase_timing.durable_append_micros,
                        "publication_micros": phase_timing.publication_micros
                    },
                    "counters": counters,
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "merge_execute_phase_timing_feature_adoption",
        &merge_phase_timing_samples,
        &[
            (
                "working_state_preparation_micros",
                &["phase_timing", "working_state_preparation_micros"],
            ),
            (
                "authoritative_mutation_micros",
                &["phase_timing", "authoritative_mutation_micros"],
            ),
            (
                "history_resolution_micros",
                &["phase_timing", "history_resolution_micros"],
            ),
            (
                "artifact_assembly_micros",
                &["phase_timing", "artifact_assembly_micros"],
            ),
            (
                "durable_append_micros",
                &["phase_timing", "durable_append_micros"],
            ),
            (
                "publication_micros",
                &["phase_timing", "publication_micros"],
            ),
        ],
    );
    assert!(merge_phase_timing_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &merge_phase_timing_samples,
        "merge execute phase timing should preserve single-record truth and expose nonzero tail-phase timings",
        |metrics| {
            metrics["changed_entities"].as_u64() == Some(1)
                && metrics["executed_record_count"].as_u64() == Some(1)
                && metrics["phase_timing"]["authoritative_mutation_micros"]
                    .as_u64()
                    .unwrap_or(0)
                    > 0
                && metrics["phase_timing"]["artifact_assembly_micros"]
                    .as_u64()
                    .unwrap_or(0)
                    > 0
                && metrics["phase_timing"]["durable_append_micros"]
                    .as_u64()
                    .unwrap_or(0)
                    > 0
                && metrics["phase_timing"]["publication_micros"]
                    .as_u64()
                    .unwrap_or(0)
                    > 0
                && metrics["counters"]["merge_execution_attempts"].as_u64() == Some(1)
        },
    );
}
