use super::*;

pub(super) fn certify_geometry_commit_downstream_mixed_locality_wave(suite: &'static str) {
    let mixed_locality_samples = capture_perf_samples(
        suite,
        "geometry_commit_downstream_wave_mixed_locality_operational",
        || {
            let relational =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            let entities = seed_downstream_region_world(&relational, "downstream-mixed", 20, 5);
            let updated = entities[9];
            let query_targets = [
                "downstream-mixed-node-2",
                "downstream-mixed-node-7",
                "downstream-mixed-node-11",
                "downstream-mixed-node-16",
            ];
            let traversal_seeds = Arc::from([entities[7], entities[9]]);
            let mut downstream_runtime = build_mock_downstream_runtime(false, entities.len());

            let relational_commit_started_at = Instant::now();
            let update = update_entity(&relational, updated, "downstream-mixed-updated");
            let relational_commit_micros = relational_commit_started_at.elapsed().as_micros();

            let snapshot = relational.visibility_authority().snapshot();
            let traversal_packet = PlannedQueryPacket {
                label: "downstream-mixed-traversal".to_string(),
                context_id: relational
                    .read_truth()
                    .query_plan_context(&snapshot)
                    .expect("downstream mixed query plan context"),
                scope: QueryScope::ConnectivityTraversal {
                    seeds: traversal_seeds,
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                    max_depth: Some(2),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(92_201),
                target_count_hint: 2,
            };
            let relational_query_started_at = Instant::now();
            let traversal = relational
                .read_truth()
                .execute_query_plan(
                    relational
                        .read_truth()
                        .plan_query_packet(&snapshot, traversal_packet)
                        .expect("downstream mixed traversal plan"),
                )
                .expect("downstream mixed traversal outcome");
            let explicit_hits = query_targets
                .iter()
                .map(|name| {
                    relational
                        .read_truth()
                        .execute_query_plan(
                            relational
                                .read_truth()
                                .plan_query_packet(
                                    &snapshot,
                                    entity_name_index_packet(
                                        &relational,
                                        &snapshot,
                                        "downstream-mixed-explicit",
                                        name,
                                    ),
                                )
                                .expect("downstream mixed explicit plan"),
                        )
                        .expect("downstream mixed explicit outcome")
                        .result
                        .entities
                        .len()
                })
                .sum::<usize>();
            let relational_query_micros = relational_query_started_at.elapsed().as_micros();

            let affected_sources = (traversal.result.entities.len() + explicit_hits)
                .min(downstream_runtime.source_versions.len())
                .max(4);
            let downstream_before = downstream_runtime.observe();
            let downstream_started_at = Instant::now();
            downstream_runtime.apply_changes(affected_sources);
            let downstream_micros = downstream_started_at.elapsed().as_micros();
            let downstream_after = downstream_runtime.observe();

            PerfMeasurement {
                elapsed_micros: relational_commit_micros
                    + relational_query_micros
                    + downstream_micros,
                metrics: perf_metrics!({
                    "resident_entities": entities.len(),
                    "relational_changed_records": update.changed_records.len(),
                    "traversal_result_entities": traversal.result.entities.len(),
                    "explicit_result_entities": explicit_hits,
                    "affected_downstream_sources": affected_sources,
                    "downstream_nodes_evaluated": downstream_after.evaluation.nodes_evaluated
                        - downstream_before.evaluation.nodes_evaluated,
                    "downstream_nodes_recomputed": downstream_after.evaluation.nodes_recomputed
                        - downstream_before.evaluation.nodes_recomputed,
                    "downstream_tasks_scheduled": downstream_after.planner.tasks_scheduled
                        - downstream_before.planner.tasks_scheduled,
                    "downstream_tasks_pruned": downstream_after.planner.tasks_pruned_before_execution
                        - downstream_before.planner.tasks_pruned_before_execution,
                    "downstream_history_entries": downstream_runtime.recent_history_len(),
                    "phase_timing": {
                        "relational_commit_micros": relational_commit_micros,
                        "relational_query_micros": relational_query_micros,
                        "downstream_micros": downstream_micros,
                    },
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "geometry_commit_downstream_wave_mixed_locality_operational",
        &mixed_locality_samples,
        &[
            (
                "relational_commit_micros",
                &["phase_timing", "relational_commit_micros"],
            ),
            (
                "relational_query_micros",
                &["phase_timing", "relational_query_micros"],
            ),
            ("downstream_micros", &["phase_timing", "downstream_micros"]),
            ("traversal_result_entities", &["traversal_result_entities"]),
            ("explicit_result_entities", &["explicit_result_entities"]),
            (
                "affected_downstream_sources",
                &["affected_downstream_sources"],
            ),
            (
                "downstream_tasks_scheduled",
                &["downstream_tasks_scheduled"],
            ),
        ],
    );
    assert_budget(
        &mixed_locality_samples,
        "mixed locality downstream certification should keep explicit and traversal reads additive without exploding downstream recompute",
        |metrics| {
            let traversal = metrics["traversal_result_entities"].as_u64().unwrap_or(0);
            let explicit = metrics["explicit_result_entities"].as_u64().unwrap_or(0);
            let affected = metrics["affected_downstream_sources"].as_u64().unwrap_or(0);
            metrics["relational_changed_records"].as_u64() == Some(1)
                && traversal >= 4
                && explicit >= 4
                && affected >= explicit
                && metrics["downstream_tasks_scheduled"].as_u64().unwrap_or(0) >= affected
                && metrics["downstream_nodes_recomputed"].as_u64().unwrap_or(0) >= affected
        },
    );
}
