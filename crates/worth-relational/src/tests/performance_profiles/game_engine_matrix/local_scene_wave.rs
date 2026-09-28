use super::*;

pub(super) fn certify_local_scene_graph_propagation_wave(suite: &'static str) {
    let local_scene_wave_samples =
        capture_perf_samples(suite, "local_scene_graph_propagation_wave", || {
            let mut runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore);
            apply_perf_diagnostics_policy(
                &mut runtime,
                PerfDiagnosticsPolicy::GeometryOperationalHotPath,
            );
            let seeded = seed_game_engine_frame_world(&runtime, "scene-local", 8, 24);
            let updated = seeded.frame_targets[3];
            let explicit_targets = seeded
                .explicit_targets
                .iter()
                .take(12)
                .map(|entity| RecordRef::Entity(*entity))
                .collect::<Vec<_>>();
            let traversal_seeds = Arc::from([
                seeded.propagation_seeds[1],
                seeded.propagation_seeds[2],
                seeded.propagation_seeds[3],
                seeded.propagation_seeds[4],
            ]);
            let mut bridge_runtime = build_mock_bridge_runtime(false, 32);

            runtime.performance_access().reset_counters();
            let update_started_at = Instant::now();
            let update = update_entity(&runtime, updated, "scene-local-updated");
            let update_micros = update_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let propagation_packet = PlannedQueryPacket {
                label: "scene-local-propagation".to_string(),
                context_id: runtime
                    .read_truth()
                    .query_plan_context(&snapshot)
                    .expect("scene local query context"),
                scope: QueryScope::ConnectivityTraversal {
                    seeds: traversal_seeds,
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                    max_depth: Some(3),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(93_001),
                target_count_hint: 4,
            };
            let propagation_started_at = Instant::now();
            let propagation = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, propagation_packet)
                        .expect("scene local propagation plan"),
                )
                .expect("scene local propagation outcome");
            let propagation_micros = propagation_started_at.elapsed().as_micros();

            let explicit_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "scene-local-explicit",
                explicit_targets,
            );
            let explicit_started_at = Instant::now();
            let explicit = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, explicit_packet)
                        .expect("scene local explicit plan"),
                )
                .expect("scene local explicit outcome");
            let explicit_micros = explicit_started_at.elapsed().as_micros();
            assert!(runtime
                .visibility_authority()
                .release_snapshot(&snapshot)
                .is_ok());

            let affected_sources = (propagation.result.entities.len()
                + explicit.result.entities.len())
            .min(bridge_runtime.source_versions.len())
            .max(4);
            let bridge_before = bridge_runtime.observe();
            let bridge_started_at = Instant::now();
            bridge_runtime.apply_changes(affected_sources);
            let bridge_micros = bridge_started_at.elapsed().as_micros();
            let bridge_after = bridge_runtime.observe();

            measurement_with_elapsed(
                update_micros + propagation_micros + explicit_micros + bridge_micros,
                || {
                    perf_metrics!({
                        "region_count": seeded.region_count,
                        "resident_entities": seeded.entities.len(),
                        "resident_relations": seeded.relation_count,
                        "changed_records": update.changed_records.len(),
                        "update_micros": update_micros,
                        "propagation_micros": propagation_micros,
                        "explicit_query_micros": explicit_micros,
                        "bridge_micros": bridge_micros,
                        "propagation_result_entities": propagation.result.entities.len(),
                        "explicit_result_entities": explicit.result.entities.len(),
                        "affected_bridge_sources": affected_sources,
                        "bridge_nodes_recomputed": bridge_after.evaluation.nodes_recomputed
                            - bridge_before.evaluation.nodes_recomputed,
                        "bridge_tasks_scheduled": bridge_after.planner.tasks_scheduled
                            - bridge_before.planner.tasks_scheduled,
                        "counters": runtime.performance_access().counters(),
                    })
                },
            )
        });
    emit_metric_summaries(
        suite,
        "local_scene_graph_propagation_wave",
        &local_scene_wave_samples,
        &[
            ("update_micros", &["update_micros"]),
            ("propagation_micros", &["propagation_micros"]),
            ("explicit_query_micros", &["explicit_query_micros"]),
            ("bridge_micros", &["bridge_micros"]),
            (
                "propagation_result_entities",
                &["propagation_result_entities"],
            ),
            ("explicit_result_entities", &["explicit_result_entities"]),
            ("affected_bridge_sources", &["affected_bridge_sources"]),
            ("bridge_tasks_scheduled", &["bridge_tasks_scheduled"]),
        ],
    );
    assert_budget(
        &local_scene_wave_samples,
        "game-engine local scene waves should keep frame-local propagation and derived work region-bounded",
        |metrics| {
            let affected = metrics["affected_bridge_sources"].as_u64().unwrap_or(0);
            metrics["region_count"].as_u64() == Some(8)
                && metrics["resident_entities"].as_u64() == Some(192)
                && metrics["changed_records"].as_u64() == Some(1)
                && metrics["propagation_result_entities"].as_u64().unwrap_or(0) >= 8
                && metrics["explicit_result_entities"].as_u64().unwrap_or(0) == 12
                && (8..=32).contains(&affected)
                && metrics["bridge_tasks_scheduled"].as_u64().unwrap_or(0) >= affected
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );
}
