use super::*;

pub(super) fn certify_geometry_commit_downstream_medium_region_wave(suite: &'static str) {
    for (case, development_profile) in [
        (
            "geometry_commit_downstream_wave_medium_region_operational",
            false,
        ),
        (
            "geometry_commit_downstream_wave_medium_region_development",
            true,
        ),
    ] {
        let samples = capture_perf_samples(suite, case, || {
            let mut relational =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            relational.configure_for_test(|config| {
                config.diagnostics.profile.detailed_traces_enabled = development_profile
            });
            relational.configure_for_test(|config| {
                config.diagnostics.profile.max_entries_per_artifact =
                    if development_profile { 256 } else { 0 }
            });

            let entities = seed_downstream_region_world(&relational, "downstream-medium", 24, 4);
            let updated = entities[10];
            let seeds = Arc::from([entities[8], entities[10], entities[12], entities[14]]);
            let mut downstream_runtime =
                build_mock_downstream_runtime(development_profile, entities.len());

            let relational_commit_started_at = Instant::now();
            let update = update_entity(&relational, updated, "downstream-medium-updated");
            let relational_commit_micros = relational_commit_started_at.elapsed().as_micros();

            let snapshot = relational.visibility_authority().snapshot();
            let traversal_packet = PlannedQueryPacket {
                label: "downstream-medium-traversal".to_string(),
                context_id: relational
                    .read_truth()
                    .query_plan_context(&snapshot)
                    .expect("downstream medium query plan context"),
                scope: QueryScope::ConnectivityTraversal {
                    seeds,
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                    max_depth: Some(3),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(92_101),
                target_count_hint: 4,
            };
            let relational_query_started_at = Instant::now();
            let traversal = relational
                .read_truth()
                .execute_query_plan(
                    relational
                        .read_truth()
                        .plan_query_packet(&snapshot, traversal_packet)
                        .expect("downstream medium traversal plan"),
                )
                .expect("downstream medium traversal outcome");
            let relational_query_micros = relational_query_started_at.elapsed().as_micros();

            let affected_sources = traversal
                .result
                .entities
                .len()
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
                    "relational_result_entities": traversal.result.entities.len(),
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
        });
        emit_metric_summaries(
            suite,
            case,
            &samples,
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
                ("resident_entities", &["resident_entities"]),
                (
                    "affected_downstream_sources",
                    &["affected_downstream_sources"],
                ),
                (
                    "downstream_nodes_recomputed",
                    &["downstream_nodes_recomputed"],
                ),
                (
                    "downstream_tasks_scheduled",
                    &["downstream_tasks_scheduled"],
                ),
            ],
        );
        assert_budget(
            &samples,
            "medium downstream region certification should scale recompute with the affected region instead of the whole resident world",
            |metrics| {
                let affected = metrics["affected_downstream_sources"].as_u64().unwrap_or(0);
                let resident = metrics["resident_entities"].as_u64().unwrap_or(0);
                metrics["relational_changed_records"].as_u64() == Some(1)
                    && metrics["relational_result_entities"].as_u64().unwrap_or(0) >= 8
                    && affected >= 8
                    && affected < resident
                    && metrics["downstream_nodes_recomputed"].as_u64().unwrap_or(0) >= affected
                    && metrics["downstream_nodes_recomputed"].as_u64().unwrap_or(0) <= affected * 4
                    && metrics["downstream_tasks_scheduled"].as_u64().unwrap_or(0) >= affected
                    && metrics["downstream_tasks_scheduled"].as_u64().unwrap_or(0) <= affected * 3
            },
        );
    }
}
