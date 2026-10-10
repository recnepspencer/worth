use super::*;

pub(super) fn certify_hundred_k_nodes_pseudorealistic_narrow_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
    node_count: usize,
    query_target_count: usize,
) {
    let pseudorealistic_samples = capture_perf_samples(
        suite,
        "hundred_k_nodes_pseudorealistic_subsystem_round_trip",
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
            let seeded = seed_pseudorealistic_rocketship_world(
                execution,
                &runtime,
                node_count,
                query_target_count,
            );

            runtime.performance_access().reset_counters();
            let hot_update_started_at = Instant::now();
            let update = update_entity(
                &runtime,
                seeded.hot_update_target,
                "rocket.engine_cluster.hot_patch",
            );
            let hot_update_micros = hot_update_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let explicit_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "rocketship-pseudorealistic-explicit",
                seeded.mixed_query_targets.clone(),
            );
            let explicit_plan_started_at = Instant::now();
            let explicit_plan = runtime
                .read_truth()
                .plan_query_packet(&snapshot, explicit_packet)
                .expect("planned pseudorealistic explicit query");
            let explicit_query_planning_micros = explicit_plan_started_at.elapsed().as_micros();
            let explicit_execution_started_at = Instant::now();
            let explicit_outcome = runtime
                .read_truth()
                .execute_query_plan(
                    explicit_plan,
                    execution,
                    crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("pseudorealistic explicit query outcome");
            let explicit_query_execution_micros =
                explicit_execution_started_at.elapsed().as_micros();

            let traversal_context = runtime
                .read_truth()
                .query_plan_context(&snapshot)
                .expect("pseudorealistic traversal context");
            let traversal_packet = PlannedQueryPacket {
                label: "rocketship-pseudorealistic-traversal".to_string(),
                context_id: traversal_context,
                scope: QueryScope::ConnectivityTraversal {
                    seeds: Arc::from(seeded.traversal_seeds.clone()),
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                    max_depth: Some(2),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(91_001),
                target_count_hint: seeded.traversal_seeds.len(),
            };
            let traversal_plan_started_at = Instant::now();
            let traversal_plan = runtime
                .read_truth()
                .plan_query_packet(&snapshot, traversal_packet)
                .expect("planned pseudorealistic traversal query");
            let traversal_planning_micros = traversal_plan_started_at.elapsed().as_micros();
            let traversal_execution_started_at = Instant::now();
            let traversal_outcome = runtime
                .read_truth()
                .execute_query_plan(
                    traversal_plan,
                    execution,
                    crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("pseudorealistic traversal outcome");
            let traversal_execution_micros = traversal_execution_started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();
            let (diagnostic_artifact_count, detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);

            let elapsed_micros = seeded.entity_commit_micros
                + seeded.relation_commit_micros
                + hot_update_micros
                + explicit_query_planning_micros
                + explicit_query_execution_micros
                + traversal_planning_micros
                + traversal_execution_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "resident_node_count": seeded.entities.len(),
                    "resident_relation_count": seeded.relation_count,
                    "subsystem_count": seeded.subsystem_count,
                    "bootstrap_entity_commit_micros": seeded.entity_commit_micros,
                    "bootstrap_relation_commit_micros": seeded.relation_commit_micros,
                    "bootstrap_relation_phase_timing": {
                        "draft_preparation_micros": seeded.relation_commit_phase_timing.draft_preparation_micros,
                        "draft_bulk_admission_micros": seeded.relation_commit_phase_timing.draft_bulk_admission_micros,
                        "draft_merge_plan_micros": seeded.relation_commit_phase_timing.draft_merge_plan_micros,
                        "draft_structural_summary_micros": seeded.relation_commit_phase_timing.draft_structural_summary_micros,
                        "draft_working_state_clone_micros": seeded.relation_commit_phase_timing.draft_working_state_clone_micros,
                        "invariant_pre_check_micros": seeded.relation_commit_phase_timing.invariant_pre_check_micros,
                        "authoritative_mutation_micros": seeded.relation_commit_phase_timing.authoritative_mutation_micros,
                        "history_resolution_micros": seeded.relation_commit_phase_timing.history_resolution_micros,
                        "invariant_post_check_micros": seeded.relation_commit_phase_timing.invariant_post_check_micros,
                        "artifact_assembly_micros": seeded.relation_commit_phase_timing.artifact_assembly_micros,
                        "durable_append_micros": seeded.relation_commit_phase_timing.durable_append_micros,
                        "publication_micros": seeded.relation_commit_phase_timing.publication_micros,
                        "publication_storage_commit_micros": seeded.relation_commit_phase_timing.publication_storage_commit_micros,
                    },
                    "hot_update_micros": hot_update_micros,
                    "explicit_query_planning_micros": explicit_query_planning_micros,
                    "explicit_query_execution_micros": explicit_query_execution_micros,
                    "traversal_planning_micros": traversal_planning_micros,
                    "traversal_execution_micros": traversal_execution_micros,
                    "hot_changed_records": update.changed_records.len(),
                    "mixed_query_target_count": seeded.mixed_query_targets.len(),
                    "explicit_query_result_entities": explicit_outcome.result.entities.len(),
                    "traversal_seed_count": seeded.traversal_seeds.len(),
                    "traversal_result_entities": traversal_outcome.result.entities.len(),
                    "traversal_result_relations": traversal_outcome.result.relations.len(),
                    "diagnostic_artifact_count": diagnostic_artifact_count,
                    "detailed_trace_entries": detailed_trace_entries,
                    "profile_boundary": profile_boundary_metrics(
                        &runtime,
                        RelationalRuntimeProfile::GeometryKernel,
                    ),
                    "counters": counters,
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "hundred_k_nodes_pseudorealistic_subsystem_round_trip",
        &pseudorealistic_samples,
        &[
            ("resident_node_count", &["resident_node_count"]),
            ("resident_relation_count", &["resident_relation_count"]),
            ("subsystem_count", &["subsystem_count"]),
            (
                "bootstrap_entity_commit_micros",
                &["bootstrap_entity_commit_micros"],
            ),
            (
                "bootstrap_relation_commit_micros",
                &["bootstrap_relation_commit_micros"],
            ),
            ("hot_update_micros", &["hot_update_micros"]),
            (
                "explicit_query_planning_micros",
                &["explicit_query_planning_micros"],
            ),
            (
                "explicit_query_execution_micros",
                &["explicit_query_execution_micros"],
            ),
            ("traversal_planning_micros", &["traversal_planning_micros"]),
            (
                "traversal_execution_micros",
                &["traversal_execution_micros"],
            ),
            ("mixed_query_target_count", &["mixed_query_target_count"]),
            (
                "explicit_query_result_entities",
                &["explicit_query_result_entities"],
            ),
            ("traversal_seed_count", &["traversal_seed_count"]),
            ("traversal_result_entities", &["traversal_result_entities"]),
            (
                "traversal_result_relations",
                &["traversal_result_relations"],
            ),
            (
                "profile_execution_lane_code",
                &["profile_boundary", "execution_lane_code"],
            ),
            (
                "profile_diagnostics_boundary_code",
                &["profile_boundary", "diagnostics_boundary_code"],
            ),
            (
                "profile_matches_defaults",
                &["profile_boundary", "matches_defaults"],
            ),
        ],
    );
    assert!(pseudorealistic_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &pseudorealistic_samples,
        "pseudorealistic rocketship should preserve mixed subsystem truth, narrow hot updates, and bounded mixed-locality query work",
        |metrics| {
            let mixed_query_target_count =
                metrics["mixed_query_target_count"].as_u64().unwrap_or(0);
            let traversal_seed_count = metrics["traversal_seed_count"].as_u64().unwrap_or(0);
            metrics["resident_node_count"].as_u64() == Some(node_count as u64)
                && metrics["resident_relation_count"].as_u64().unwrap_or(0) >= node_count as u64
                && metrics["subsystem_count"].as_u64() == Some(12)
                && metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["explicit_query_result_entities"].as_u64()
                    == Some(mixed_query_target_count)
                && metrics["traversal_result_entities"].as_u64().unwrap_or(0)
                    >= traversal_seed_count
                && metrics["traversal_result_relations"].as_u64().unwrap_or(0) >= 1
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(0)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "entity_slots_touched_by_commit") == 1
                && counter_u64(metrics, "query_packet_count")
                    <= mixed_query_target_count + traversal_seed_count
                && counter_u64(metrics, "query_scope_unit_count")
                    <= mixed_query_target_count + traversal_seed_count
        },
    );
}
