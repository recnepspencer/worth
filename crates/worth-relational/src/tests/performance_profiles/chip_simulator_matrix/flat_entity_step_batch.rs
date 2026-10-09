use super::*;

pub(super) fn certify_flat_entity_step_batch_compile_window(suite: &'static str) {
    let flat_step_batch_samples = capture_perf_samples(
        suite,
        "flat_entity_step_batch_compile_window",
        || {
            let mut runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
            runtime.configure_diagnostics_for_test(|profile| {
                profile.detailed_traces_enabled = false;
                profile.max_entries_per_artifact = 0;
            });
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();

            let _source = create_entity_in_partition(&runtime, "chip-batch-driver", PartitionId(7));
            let mut partition_targets = BTreeMap::new();
            for partition_offset in 0..8u32 {
                let partition_id = PartitionId(11 + partition_offset * 2);
                let targets = (0..8)
                    .map(|index| {
                        create_entity_in_partition(
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
            let update_started_at = Instant::now();
            let update = {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
                let mut batch = WorkerIntentBatch::new("chip-flat-entity-step-batch");
                for (partition_id, targets) in &partition_targets {
                    for (index, entity) in targets.iter().enumerate().take(4) {
                        batch = batch.push(MutationIntent::Entity(
                            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                                entity_id: *entity,
                                fields: crate::tests::support::aspect_field_patch_from_values([
                                    (
                                        crate::tests::support::aspect_key("partition"),
                                        crate::tests::support::field_key("partition"),
                                        crate::tests::support::u64_aspect_value(
                                            partition_id.0 as u64,
                                        ),
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
                            }),
                        ));
                    }
                }
                txn.push_batch(
                    batch,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("test staging stays within configured resource budgets");
                txn.commit(
                    &runtime,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("chip flat entity step batch commit")
            };
            let update_micros = update_started_at.elapsed().as_micros();
            let phase_timing = update.execution().phase_timing.clone();
            let commit = runtime
                .history()
                .latest_commit()
                .expect("chip flat batch latest commit")
                .clone();

            let compile_started_at = Instant::now();
            let artifact = runtime
                .compiled_artifacts_authority()
                .compile_execution_artifact(commit.commit_id, compile_partitions)
                .expect("chip flat batch compiled artifact");
            let compile_micros = compile_started_at.elapsed().as_micros();

            let sample_targets = partition_targets
                .values()
                .flat_map(|targets| targets.iter().take(1).copied())
                .map(RecordRef::Entity)
                .collect::<Vec<_>>();
            let snapshot = runtime.visibility_authority().snapshot();
            let explicit_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "chip-flat-batch-explicit",
                sample_targets,
            );
            let explicit_started_at = Instant::now();
            let explicit = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, explicit_packet)
                        .expect("planned chip flat batch explicit query"),
                )
                .expect("chip flat batch explicit outcome");
            let explicit_query_micros = explicit_started_at.elapsed().as_micros();
            assert!(runtime
                .visibility_authority()
                .release_snapshot(&snapshot)
                .is_ok());

            let counters = runtime.performance_access().counters();
            let (diagnostic_artifact_count, detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);

            measurement_with_elapsed(
                update_micros + compile_micros + explicit_query_micros,
                || {
                    perf_metrics!({
                        "batch_target_count": 32,
                        "batch_partition_count": partition_targets.len(),
                        "update_micros": update_micros,
                        "compile_micros": compile_micros,
                        "explicit_query_micros": explicit_query_micros,
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
                        "phase_timing": {
                            "draft_preparation_micros": phase_timing.draft_preparation_micros,
                            "draft_working_state_clone_micros": phase_timing.draft_working_state_clone_micros,
                            "publication_storage_commit_micros": phase_timing.publication_storage_commit_micros,
                        },
                        "counters": counters,
                    })
                },
            )
        },
    );
    emit_metric_summaries(
        suite,
        "flat_entity_step_batch_compile_window",
        &flat_step_batch_samples,
        &[
            ("batch_target_count", &["batch_target_count"]),
            ("batch_partition_count", &["batch_partition_count"]),
            ("update_micros", &["update_micros"]),
            ("compile_micros", &["compile_micros"]),
            ("explicit_query_micros", &["explicit_query_micros"]),
            (
                "draft_preparation_micros",
                &["phase_timing", "draft_preparation_micros"],
            ),
            (
                "draft_working_state_clone_micros",
                &["phase_timing", "draft_working_state_clone_micros"],
            ),
            (
                "publication_storage_commit_micros",
                &["phase_timing", "publication_storage_commit_micros"],
            ),
            (
                "aosoa_entity_chunk_slots_materialized",
                &["counters", "aosoa_entity_chunk_slots_materialized"],
            ),
            (
                "aosoa_entity_chunks_published",
                &["counters", "aosoa_entity_chunks_published"],
            ),
        ],
    );
    assert!(flat_step_batch_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
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
