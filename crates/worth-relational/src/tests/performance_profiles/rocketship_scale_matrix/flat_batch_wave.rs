use super::*;

pub(super) fn certify_hundred_k_nodes_pseudorealistic_flat_batch_wave(
    suite: &'static str,
    node_count: usize,
    query_target_count: usize,
) {
    let flat_batch_wave_samples = capture_perf_samples(
        suite,
        "hundred_k_nodes_pseudorealistic_flat_entity_batch_wave",
        || {
            let mut runtime = runtime_with_test_schema_profile_and_chunks(
                RelationalRuntimeProfile::GeometryKernel,
                ROCKETSHIP_CHUNK_SIZE,
                ROCKETSHIP_CHUNK_SIZE,
            );
            apply_perf_diagnostics_policy(
                &mut runtime,
                PerfDiagnosticsPolicy::GeometryOperationalHotPath,
            );
            runtime.configure_for_test(|config| {
                config.publication.policy.max_patch_records_per_commit = node_count * 2
            });
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();
            let seeded =
                seed_pseudorealistic_rocketship_world(&runtime, node_count, query_target_count);

            let mut partition_targets = BTreeMap::new();
            for entity in &seeded.entities {
                let targets = partition_targets
                    .entry(entity.partition_id)
                    .or_insert_with(Vec::new);
                if targets.len() < 8 {
                    targets.push(*entity);
                }
                if partition_targets.len() >= 8
                    && partition_targets.values().all(|targets| targets.len() >= 8)
                {
                    break;
                }
            }
            let batch_targets = partition_targets
                .values()
                .flat_map(|targets| targets.iter().take(8).copied())
                .collect::<Vec<_>>();
            assert!(
                batch_targets.len() >= 64,
                "rocketship flat batch wave should gather a broad multi-partition entity batch"
            );

            runtime.performance_access().reset_counters();
            let update_started_at = Instant::now();
            let update = {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
                let mut batch = WorkerIntentBatch::new("rocketship-flat-entity-batch-wave");
                for (index, entity) in batch_targets.iter().enumerate() {
                    batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent {
                            entity_id: *entity,
                            fields: crate::tests::support::aspect_field_patch_from_values([
                                (
                                    crate::tests::support::aspect_key("section"),
                                    crate::tests::support::field_key("section"),
                                    crate::tests::support::string_aspect_value("batch-wave"),
                                ),
                                (
                                    crate::tests::support::aspect_key("tag"),
                                    crate::tests::support::field_key("tag"),
                                    crate::tests::support::string_aspect_value(&format!(
                                        "rocket.batch.{index}"
                                    )),
                                ),
                                (
                                    crate::tests::support::aspect_key("partition"),
                                    crate::tests::support::field_key("partition"),
                                    crate::tests::support::u64_aspect_value(
                                        entity.partition_id.0 as u64,
                                    ),
                                ),
                            ]),
                        },
                    )));
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
                .expect("rocketship flat entity batch wave commit")
            };
            let update_micros = update_started_at.elapsed().as_micros();
            let phase_timing = update.execution().phase_timing.clone();

            let snapshot = runtime.visibility_authority().snapshot();
            let explicit_targets = batch_targets
                .iter()
                .take(16)
                .copied()
                .map(RecordRef::Entity)
                .collect::<Vec<_>>();
            let explicit_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "rocketship-flat-entity-batch-explicit",
                explicit_targets,
            );
            let explicit_started_at = Instant::now();
            let explicit = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, explicit_packet)
                        .expect("planned rocketship flat batch explicit query"),
                )
                .expect("rocketship flat batch explicit outcome");
            let explicit_query_micros = explicit_started_at.elapsed().as_micros();
            assert!(runtime
                .visibility_authority()
                .release_snapshot(&snapshot)
                .is_ok());

            let counters = runtime.performance_access().counters();
            let (diagnostic_artifact_count, detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);

            measurement_with_elapsed(update_micros + explicit_query_micros, || {
                perf_metrics!({
                    "resident_node_count": seeded.entities.len(),
                    "resident_relation_count": seeded.relation_count,
                    "subsystem_count": seeded.subsystem_count,
                    "batch_target_count": batch_targets.len(),
                    "batch_partition_count": partition_targets.len(),
                    "update_micros": update_micros,
                    "explicit_query_micros": explicit_query_micros,
                    "hot_changed_records": update.changed_records.len(),
                    "explicit_result_entities": explicit.result.entities.len(),
                    "diagnostic_artifact_count": diagnostic_artifact_count,
                    "detailed_trace_entries": detailed_trace_entries,
                    "phase_timing": {
                        "draft_preparation_micros": phase_timing.draft_preparation_micros,
                        "draft_intent_normalization_micros": phase_timing.draft_intent_normalization_micros,
                        "draft_merge_plan_micros": phase_timing.draft_merge_plan_micros,
                        "draft_intent_validation_micros": phase_timing.draft_intent_validation_micros,
                        "draft_intent_sort_micros": phase_timing.draft_intent_sort_micros,
                        "draft_conflict_detection_micros": phase_timing.draft_conflict_detection_micros,
                        "draft_structural_summary_micros": phase_timing.draft_structural_summary_micros,
                        "draft_working_state_clone_micros": phase_timing.draft_working_state_clone_micros,
                        "invariant_pre_check_micros": phase_timing.invariant_pre_check_micros,
                        "authoritative_mutation_micros": phase_timing.authoritative_mutation_micros,
                        "history_resolution_micros": phase_timing.history_resolution_micros,
                        "invariant_post_check_micros": phase_timing.invariant_post_check_micros,
                        "durable_append_micros": phase_timing.durable_append_micros,
                        "publication_micros": phase_timing.publication_micros,
                        "publication_storage_commit_micros": phase_timing.publication_storage_commit_micros,
                    },
                    "counters": counters,
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "hundred_k_nodes_pseudorealistic_flat_entity_batch_wave",
        &flat_batch_wave_samples,
        &[
            ("batch_target_count", &["batch_target_count"]),
            ("batch_partition_count", &["batch_partition_count"]),
            ("update_micros", &["update_micros"]),
            (
                "draft_preparation_micros",
                &["phase_timing", "draft_preparation_micros"],
            ),
            (
                "draft_intent_normalization_micros",
                &["phase_timing", "draft_intent_normalization_micros"],
            ),
            (
                "draft_merge_plan_micros",
                &["phase_timing", "draft_merge_plan_micros"],
            ),
            (
                "draft_intent_validation_micros",
                &["phase_timing", "draft_intent_validation_micros"],
            ),
            (
                "draft_intent_sort_micros",
                &["phase_timing", "draft_intent_sort_micros"],
            ),
            (
                "draft_conflict_detection_micros",
                &["phase_timing", "draft_conflict_detection_micros"],
            ),
            (
                "draft_structural_summary_micros",
                &["phase_timing", "draft_structural_summary_micros"],
            ),
            (
                "draft_working_state_clone_micros",
                &["phase_timing", "draft_working_state_clone_micros"],
            ),
            (
                "invariant_pre_check_micros",
                &["phase_timing", "invariant_pre_check_micros"],
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
                "invariant_post_check_micros",
                &["phase_timing", "invariant_post_check_micros"],
            ),
            (
                "durable_append_micros",
                &["phase_timing", "durable_append_micros"],
            ),
            (
                "publication_storage_commit_micros",
                &["phase_timing", "publication_storage_commit_micros"],
            ),
            ("explicit_query_micros", &["explicit_query_micros"]),
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
    assert!(flat_batch_wave_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &flat_batch_wave_samples,
        "pseudorealistic rocketship flat entity batches should stay on the widened sparse AoSoA path across multiple touched partitions",
        |metrics| {
            let batch_target_count = metrics["batch_target_count"].as_u64().unwrap_or(0);
            let batch_partition_count = metrics["batch_partition_count"].as_u64().unwrap_or(0);
            metrics["resident_node_count"].as_u64() == Some(node_count as u64)
                && metrics["resident_relation_count"].as_u64().unwrap_or(0) >= node_count as u64
                && metrics["subsystem_count"].as_u64() == Some(12)
                && batch_target_count >= 64
                && batch_partition_count >= 8
                && metrics["hot_changed_records"].as_u64() == Some(batch_target_count)
                && metrics["explicit_result_entities"].as_u64() == Some(16)
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "entity_slots_touched_by_commit") == batch_target_count
                && counter_u64(metrics, "partitions_touched_by_commit") >= batch_partition_count
                && counter_u64(metrics, "aosoa_entity_chunk_slots_materialized")
                    == batch_target_count
                && counter_u64(metrics, "aosoa_entity_chunks_published") >= batch_partition_count
                && counter_u64(metrics, "aosoa_publish_soa_merge_count") == 0
        },
    );
}
