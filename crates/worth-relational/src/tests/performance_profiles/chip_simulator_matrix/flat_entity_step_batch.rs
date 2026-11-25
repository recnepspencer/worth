use super::*;

pub(super) fn certify_flat_entity_step_batch_compile_window(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let flat_step_batch_samples = capture_counted_samples(|| {
        let mut runtime = runtime_with_extra_entity_aspects_profile(
            RelationalRuntimeProfile::ChipSimulation,
            vec![
                crate::tests::support::entity_u64_field_aspect(
                    crate::tests::support::aspect_key("partition"),
                    crate::tests::support::field_key("partition"),
                ),
                crate::tests::support::entity_field_aspect(
                    crate::tests::support::aspect_key("lane"),
                    crate::tests::support::field_key("lane"),
                ),
                crate::tests::support::entity_u64_field_aspect(
                    crate::tests::support::aspect_key("step"),
                    crate::tests::support::field_key("step"),
                ),
            ],
        );
        runtime.configure_diagnostics_for_test(|profile| {
            profile.detailed_traces_enabled = false;
            profile.max_entries_per_artifact = 0;
        });
        let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

        let _source =
            create_entity_in_partition(execution, &runtime, "chip-batch-driver", PartitionId(7));
        let mut partition_targets = BTreeMap::new();
        for partition_offset in 0..8u32 {
            let partition_id = PartitionId(11 + partition_offset * 2);
            let targets = (0..8)
                .map(|index| {
                    create_entity_in_partition(
                        execution,
                        &runtime,
                        &format!("chip-batch-sink-{}-{index}", partition_id.0),
                        partition_id,
                    )
                })
                .collect::<Vec<_>>();
            partition_targets.insert(partition_id, targets);
        }
        let compile_partitions = std::iter::once(PartitionId(7))
            .chain(partition_targets.keys().copied())
            .collect::<Vec<_>>();

        runtime.performance_access().reset_counters();
        let (update, update_work) = measure_counted_work(execution, || {
            let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
            let mut batch = WorkerIntentBatch::new("chip-flat-entity-step-batch");
            for (partition_id, targets) in &partition_targets {
                for (index, entity) in targets.iter().enumerate().take(4) {
                    batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent {
                            entity_id: *entity,
                            fields: crate::tests::support::aspect_field_patch_from_values([
                                (
                                    crate::tests::support::aspect_key("partition"),
                                    crate::tests::support::field_key("partition"),
                                    crate::tests::support::u64_aspect_value(partition_id.0 as u64),
                                ),
                                (
                                    crate::tests::support::aspect_key("lane"),
                                    crate::tests::support::field_key("lane"),
                                    crate::tests::support::string_aspect_value("global-step"),
                                ),
                                (
                                    crate::tests::support::aspect_key("step"),
                                    crate::tests::support::field_key("step"),
                                    crate::tests::support::usize_aspect_value(index),
                                ),
                            ]),
                        },
                    )));
                }
            }
            txn.push_batch(
                batch,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("test staging stays within configured resource budgets");
            txn.commit(execution, &runtime)
                .expect("chip flat entity step batch commit")
        });
        let commit = runtime
            .history()
            .latest_commit()
            .expect("chip flat batch latest commit")
            .clone();

        let (artifact, compile_work) = measure_counted_work(execution, || {
            runtime
                .compiled_artifacts_authority()
                .compile_execution_artifact(execution, commit.commit_id, compile_partitions)
                .expect("chip flat batch compiled artifact")
        });

        let sample_targets = partition_targets
            .values()
            .flat_map(|targets| targets.iter().take(1).copied())
            .map(RecordRef::Entity)
            .collect::<Vec<_>>();
        let snapshot = runtime
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");
        let explicit_packet = explicit_query_packet(
            &runtime,
            &snapshot,
            "chip-flat-batch-explicit",
            sample_targets,
        );
        let (explicit, explicit_query_work) = measure_counted_work(execution, || {
            runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, explicit_packet)
                        .expect("planned chip flat batch explicit query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("chip flat batch explicit outcome")
        });
        assert!(runtime
            .visibility_authority()
            .release_snapshot(&snapshot)
            .is_ok());

        let counters = runtime.performance_access().counters();
        let (diagnostic_artifact_count, detailed_trace_entries) =
            fresh_diagnostics_metrics(&runtime, diagnostics_start);

        CountedPerfSample {
            charged_work: update_work + compile_work + explicit_query_work,
            metrics: perf_metrics!({
                "batch_target_count": 32,
                "batch_partition_count": partition_targets.len(),
                "update_work": update_work,
                "compile_work": compile_work,
                "explicit_query_work": explicit_query_work,
                "hot_changed_records": update.changed_records.len(),
                "explicit_result_entities": explicit.result.entities.len(),
                "diagnostic_artifact_count": diagnostic_artifact_count,
                "detailed_trace_entries": detailed_trace_entries,
                "compiled_artifact_authority_status": format!(
                    "{:?}",
                    runtime
                        .compiled_artifacts()
                        .compiled_artifact_authority_status(artifact.artifact_id)
                ),
                "counters": counters,
            }),
        }
    });
    assert!(flat_step_batch_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &flat_step_batch_samples,
        "chip flat entity step batches should stay on the widened sparse AoSoA path while remaining compile-ready",
        |metrics| {
            metrics["batch_target_count"].as_u64() == Some(32)
                && metrics["batch_partition_count"].as_u64() == Some(8)
                && metrics["hot_changed_records"].as_u64() == Some(32)
                && metrics["explicit_result_entities"].as_u64() == Some(8)
                && metrics["compiled_artifact_authority_status"].as_str()
                    == Some(&format!("{:?}", CompiledArtifactAuthorityStatus::Authoritative))
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "entity_slots_touched_by_commit") == 32
                && counter_u64(metrics, "partitions_touched_by_commit") >= 8
                && counter_u64(metrics, "aosoa_entity_chunk_slots_materialized") == 32
                && counter_u64(metrics, "aosoa_entity_chunks_published") >= 8
                && counter_u64(metrics, "aosoa_publish_soa_merge_count") == 0
        },
    );
}
