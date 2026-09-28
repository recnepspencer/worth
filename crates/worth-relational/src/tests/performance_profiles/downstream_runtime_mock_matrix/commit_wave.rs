use super::*;

pub(super) fn certify_geometry_commit_downstream_wave(suite: &'static str) {
    for (case, development_profile) in [
        ("geometry_commit_downstream_wave_operational", false),
        ("geometry_commit_downstream_wave_development", true),
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

            let source = create_entity_outcome(&relational, "merged-geometry-source");
            let middle = create_entity_outcome(&relational, "merged-geometry-middle");
            let target = create_entity_outcome(&relational, "merged-geometry-target");
            let source_entity = changed_entities(&source)[0];
            let middle_entity = changed_entities(&middle)[0];
            let target_entity = changed_entities(&target)[0];
            create_relation_outcome(
                &relational,
                source_entity,
                middle_entity,
                "merged-geometry-link-a",
            );
            create_relation_outcome(
                &relational,
                middle_entity,
                target_entity,
                "merged-geometry-link-b",
            );

            let mut downstream_runtime = build_mock_downstream_runtime(development_profile, 4);

            let relational_commit_started_at = Instant::now();
            let update =
                update_entity(&relational, middle_entity, "merged-geometry-middle-updated");
            let relational_commit_micros = relational_commit_started_at.elapsed().as_micros();

            let snapshot = relational.visibility_authority().snapshot();
            let traversal_packet = PlannedQueryPacket {
                label: "merged-relational-signal-traversal".to_string(),
                context_id: relational
                    .read_truth()
                    .query_plan_context(&snapshot)
                    .expect("merged query plan context"),
                scope: QueryScope::ConnectivityTraversal {
                    seeds: Arc::from([source_entity, middle_entity]),
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                    max_depth: Some(2),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(92_001),
                target_count_hint: 2,
            };
            let relational_query_started_at = Instant::now();
            let traversal = relational
                .read_truth()
                .execute_query_plan(
                    relational
                        .read_truth()
                        .plan_query_packet(&snapshot, traversal_packet)
                        .expect("merged traversal plan"),
                )
                .expect("merged traversal outcome");
            let relational_query_micros = relational_query_started_at.elapsed().as_micros();

            let affected_sources = traversal
                .result
                .entities
                .len()
                .min(downstream_runtime.source_versions.len())
                .max(1);
            let downstream_before = downstream_runtime.observe();
            let downstream_started_at = Instant::now();
            downstream_runtime.apply_changes(affected_sources);
            let downstream_micros = downstream_started_at.elapsed().as_micros();
            let downstream_after = downstream_runtime.observe();
            let downstream_history_entries = downstream_runtime.recent_history_len();

            PerfMeasurement {
                elapsed_micros: relational_commit_micros
                    + relational_query_micros
                    + downstream_micros,
                metrics: perf_metrics!({
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
                    "downstream_suppressed_propagations": downstream_after.evaluation.suppressed_downstream_propagations
                        - downstream_before.evaluation.suppressed_downstream_propagations,
                    "downstream_history_entries": downstream_history_entries,
                    "downstream_has_latest_flow": downstream_runtime.latest_flow_diagnostics().is_some(),
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
                (
                    "affected_downstream_sources",
                    &["affected_downstream_sources"],
                ),
                (
                    "downstream_nodes_evaluated",
                    &["downstream_nodes_evaluated"],
                ),
                (
                    "downstream_nodes_recomputed",
                    &["downstream_nodes_recomputed"],
                ),
                (
                    "downstream_tasks_scheduled",
                    &["downstream_tasks_scheduled"],
                ),
                (
                    "downstream_history_entries",
                    &["downstream_history_entries"],
                ),
            ],
        );
        assert_budget(
            &samples,
            "mocked downstream certification should keep truth updates narrow while surfacing downstream invalidation and recomputation work without crossing the crate boundary",
            |metrics| {
                let affected = metrics["affected_downstream_sources"].as_u64().unwrap_or(0);
                metrics["relational_changed_records"].as_u64() == Some(1)
                    && metrics["relational_result_entities"].as_u64().unwrap_or(0) >= 2
                    && affected >= 1
                    && metrics["downstream_nodes_recomputed"].as_u64().unwrap_or(0) >= affected
                    && metrics["downstream_tasks_scheduled"].as_u64().unwrap_or(0) >= affected
                    && metrics["downstream_history_entries"].as_u64().unwrap_or(0) >= 1
                    && metrics["downstream_has_latest_flow"].as_bool() == Some(true)
            },
        );
    }
}
