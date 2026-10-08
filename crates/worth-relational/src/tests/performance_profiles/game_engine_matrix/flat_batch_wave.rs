use super::*;

pub(super) fn certify_flat_entity_batch_region_wave(suite: &'static str) {
    let flat_batch_wave_samples =
        capture_perf_samples(suite, "flat_entity_batch_region_wave", || {
            let mut runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore);
            apply_perf_diagnostics_policy(
                &mut runtime,
                PerfDiagnosticsPolicy::GeometryOperationalHotPath,
            );
            let seeded = seed_game_engine_frame_world(&runtime, "scene-batch", 8, 24);

            let mut partition_targets = BTreeMap::new();
            for entity in &seeded.entities {
                let targets = partition_targets
                    .entry(entity.partition_id)
                    .or_insert_with(Vec::new);
                if targets.len() < 8 {
                    targets.push(*entity);
                }
                if partition_targets.len() >= 4
                    && partition_targets.values().all(|targets| targets.len() >= 6)
                {
                    break;
                }
            }
            let batch_targets = partition_targets
                .values()
                .flat_map(|targets| targets.iter().take(6).copied())
                .collect::<Vec<_>>();
            assert!(
                batch_targets.len() >= 24,
                "batch wave should gather a multi-partition entity batch"
            );

            runtime.performance_access().reset_counters();
            let update_started_at = Instant::now();
            let update = {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
                let mut batch = WorkerIntentBatch::new("scene-batch-flat-entity-wave");
                for (index, entity) in batch_targets.iter().enumerate() {
                    batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent {
                            entity_id: *entity,
                            fields: crate::tests::support::aspect_field_patch_from_values([
                                (
                                    crate::tests::support::aspect_key("name"),
                                    crate::tests::support::field_key("name"),
                                    crate::tests::support::string_aspect_value(&format!(
                                        "scene-batch-updated-{index}"
                                    )),
                                ),
                                (
                                    crate::tests::support::aspect_key("phase"),
                                    crate::tests::support::field_key("phase"),
                                    crate::tests::support::string_aspect_value("batch-wave"),
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
                .expect("scene batch flat entity wave commit")
            };
            let update_micros = update_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let explicit_targets = batch_targets
                .iter()
                .take(12)
                .map(|entity| RecordRef::Entity(*entity))
                .collect::<Vec<_>>();
            let explicit_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "scene-batch-explicit",
                explicit_targets,
            );
            let explicit_started_at = Instant::now();
            let explicit = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, explicit_packet)
                        .expect("scene batch explicit plan"),
                )
                .expect("scene batch explicit outcome");
            let explicit_micros = explicit_started_at.elapsed().as_micros();
            assert!(runtime
                .visibility_authority()
                .release_snapshot(&snapshot)
                .is_ok());

            measurement_with_elapsed(update_micros + explicit_micros, || {
                perf_metrics!({
                    "region_count": seeded.region_count,
                    "resident_entities": seeded.entities.len(),
                    "resident_relations": seeded.relation_count,
                    "batch_target_count": batch_targets.len(),
                    "batch_partition_count": partition_targets.len(),
                    "changed_records": update.changed_records.len(),
                    "update_micros": update_micros,
                    "explicit_query_micros": explicit_micros,
                    "explicit_result_entities": explicit.result.entities.len(),
                    "counters": runtime.performance_access().counters(),
                })
            })
        });
    emit_metric_summaries(
        suite,
        "flat_entity_batch_region_wave",
        &flat_batch_wave_samples,
        &[
            ("update_micros", &["update_micros"]),
            ("explicit_query_micros", &["explicit_query_micros"]),
            ("batch_target_count", &["batch_target_count"]),
            ("batch_partition_count", &["batch_partition_count"]),
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
    assert_budget(
        &flat_batch_wave_samples,
        "game-engine flat entity batches should stay on the sparse AoSoA path across a few touched partitions",
        |metrics| {
            let batch_target_count = metrics["batch_target_count"].as_u64().unwrap_or(0);
            let batch_partition_count = metrics["batch_partition_count"].as_u64().unwrap_or(0);
            metrics["region_count"].as_u64() == Some(8)
                && batch_target_count >= 24
                && batch_partition_count >= 4
                && metrics["changed_records"].as_u64() == Some(batch_target_count)
                && counter_u64(metrics, "entity_slots_touched_by_commit") == batch_target_count
                && counter_u64(metrics, "partitions_touched_by_commit") >= batch_partition_count
                && counter_u64(metrics, "aosoa_entity_chunk_slots_materialized")
                    == batch_target_count
                && counter_u64(metrics, "aosoa_entity_chunks_published") >= batch_partition_count
                && counter_u64(metrics, "aosoa_publish_soa_merge_count") == 0
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );
}
