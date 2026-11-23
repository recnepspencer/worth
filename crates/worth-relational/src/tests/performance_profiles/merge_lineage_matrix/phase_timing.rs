use super::*;

pub(super) fn certify_merge_execute_phase_timing_feature_adoption(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let merge_phase_samples = capture_counted_samples(|| {
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
                MergeExecutionRequest {
                    target_branch: BranchId("main".to_string()),
                    source_branch: BranchId("feature".to_string()),
                    merge_intent: MergeIntent::ReconcileIntoTarget,
                },
            )
            .expect("prepared merge");

        let pre_merge_bundle_commit = runtime
            .publication()
            .latest_bundle()
            .expect("pre-merge publication bundle")
            .commit
            .commit_id;
        let medium_before = medium.committed_count();
        let log_before = runtime.durable_log().len();
        runtime.performance_access().reset_counters();
        let (outcome, execute_work) = measure_counted_work(execution, || {
            runtime
                .execute_prepared_merge(execution, prepared)
                .expect("execute merge")
        });
        let counters = runtime.performance_access().counters();
        let merge_commit_id = outcome.commit.commit.commit_id;
        let merge_position = outcome.commit.publication().patch_position;
        let log_delta = runtime.durable_log().len().saturating_sub(log_before);
        let medium_delta = medium.committed_count().saturating_sub(medium_before);
        let latest_medium_artifact = medium
            .latest_committed_artifact()
            .expect("merge commit appended to owned durable medium");
        let latest_bundle = runtime
            .publication()
            .latest_bundle()
            .expect("fresh merge publication bundle");

        CountedPerfSample {
            charged_work: execute_work,
            metrics: perf_metrics!({
                "execute_work": execute_work,
                "executed_record_count": outcome.structural_summary.executed_record_count,
                "changed_entities": changed_entities(&outcome.commit).len(),
                "assembled_patch_len": outcome.commit.patch().len(),
                "envelope_matches_commit": outcome.commit.publication().envelope.commit.commit_id
                    == merge_commit_id,
                "medium_artifact_bytes": latest_medium_artifact.as_bytes().len(),
                "medium_commit_delta": medium_delta,
                "medium_artifact_matches_commit": latest_medium_artifact.commit_id()
                    == merge_commit_id
                    && latest_medium_artifact.position() == merge_position,
                "durable_log_delta": log_delta,
                "fresh_bundle_matches_commit": latest_bundle.commit.commit_id == merge_commit_id
                    && latest_bundle.commit.commit_id != pre_merge_bundle_commit
                    && latest_bundle.patch.position == merge_position,
                "counters": counters,
            }),
        }
    });
    assert!(merge_phase_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &merge_phase_samples,
        "merge execute phase budget should witness authoritative mutation, artifact assembly, durable medium append, and fresh commit-identified publication",
        |metrics| {
            metrics["changed_entities"].as_u64() == Some(1)
                && metrics["executed_record_count"].as_u64() == Some(1)
                && metrics["execute_work"].as_u64().unwrap_or(0) > 0
                && counter_u64(metrics, "entity_slots_touched_by_commit") == 1
                && counter_u64(metrics, "merge_execution_attempts") == 1
                && counter_u64(metrics, "merge_execution_records_admitted") == 1
                && counter_u64(metrics, "merge_execution_mutation_intents_emitted") == 1
                && metrics["assembled_patch_len"].as_u64() == Some(1)
                && metrics["envelope_matches_commit"].as_bool() == Some(true)
                && metrics["medium_artifact_bytes"].as_u64().unwrap_or(0) > 0
                && metrics["medium_commit_delta"].as_u64() == Some(1)
                && metrics["medium_artifact_matches_commit"].as_bool() == Some(true)
                && metrics["durable_log_delta"].as_u64() == Some(1)
                && metrics["fresh_bundle_matches_commit"].as_bool() == Some(true)
        },
    );
}
