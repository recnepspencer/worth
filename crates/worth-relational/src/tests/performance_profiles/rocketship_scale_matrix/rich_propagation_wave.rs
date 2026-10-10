use super::*;

pub(super) fn certify_hundred_k_nodes_geometry_profile_propagation_wave(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
    node_count: usize,
    query_target_count: usize,
) {
    let rich_propagation_wave_samples = capture_perf_samples(
        suite,
        "hundred_k_nodes_geometry_profile_propagation_wave",
        || {
            let mut runtime = runtime_with_test_schema_profile_and_chunks(
                RelationalRuntimeProfile::GeometryKernel,
                ROCKETSHIP_CHUNK_SIZE,
                ROCKETSHIP_CHUNK_SIZE,
            );
            apply_perf_diagnostics_policy(
                &mut runtime,
                PerfDiagnosticsPolicy::GeometryRichCertification,
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
                "rocket.plumbing_and_feed.propagation_patch.rich",
            );
            let hot_update_micros = hot_update_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let context = runtime
                .read_truth()
                .query_plan_context(&snapshot)
                .expect("rocketship rich propagation context");
            let propagation_seeds = vec![
                seeded.traversal_seeds[0],
                seeded.traversal_seeds[1],
                seeded.traversal_seeds[9],
                seeded.traversal_seeds[10],
            ];
            let propagation_packet = PlannedQueryPacket {
                label: "rocketship-pseudorealistic-propagation-rich".to_string(),
                context_id: context,
                scope: QueryScope::ConnectivityTraversal {
                    seeds: Arc::from(propagation_seeds.clone()),
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                    max_depth: Some(3),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(91_003),
                target_count_hint: propagation_seeds.len(),
            };
            let propagation_plan_started_at = Instant::now();
            let propagation_plan = runtime
                .read_truth()
                .plan_query_packet(&snapshot, propagation_packet)
                .expect("planned rocketship rich propagation query");
            let propagation_planning_micros = propagation_plan_started_at.elapsed().as_micros();
            let propagation_execution_started_at = Instant::now();
            let propagation_outcome = runtime
                .read_truth()
                .execute_query_plan(
                    propagation_plan,
                    execution,
                    crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("rocketship rich propagation outcome");
            let propagation_execution_micros =
                propagation_execution_started_at.elapsed().as_micros();

            let explicit_targets = seeded
                .mixed_query_targets
                .iter()
                .take(12)
                .cloned()
                .collect::<Vec<_>>();
            let explicit_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "rocketship-pseudorealistic-propagation-explicit-rich",
                explicit_targets.clone(),
            );
            let explicit_started_at = Instant::now();
            let explicit_outcome = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, explicit_packet)
                        .expect("planned rocketship rich propagation explicit query"),
                    execution,
                    crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("rocketship rich propagation explicit query outcome");
            let explicit_query_micros = explicit_started_at.elapsed().as_micros();
            let hot_phase_timing = update.execution().phase_timing.clone();
            let counters = runtime.performance_access().counters();
            let (diagnostic_artifact_count, detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);

            let elapsed_micros = seeded.entity_commit_micros
                + seeded.relation_commit_micros
                + hot_update_micros
                + propagation_planning_micros
                + propagation_execution_micros
                + explicit_query_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "resident_node_count": seeded.entities.len(),
                    "resident_relation_count": seeded.relation_count,
                    "subsystem_count": seeded.subsystem_count,
                    "bootstrap_entity_commit_micros": seeded.entity_commit_micros,
                    "bootstrap_relation_commit_micros": seeded.relation_commit_micros,
                    "hot_update_micros": hot_update_micros,
                    "phase_timing": {
                        "draft_preparation_micros": hot_phase_timing.draft_preparation_micros,
                        "draft_bulk_admission_micros": hot_phase_timing.draft_bulk_admission_micros,
                        "draft_merge_plan_micros": hot_phase_timing.draft_merge_plan_micros,
                        "draft_structural_summary_micros": hot_phase_timing.draft_structural_summary_micros,
                        "draft_working_state_clone_micros": hot_phase_timing.draft_working_state_clone_micros,
                        "working_state_preparation_micros": hot_phase_timing.working_state_preparation_micros,
                        "invariant_pre_check_micros": hot_phase_timing.invariant_pre_check_micros,
                        "authoritative_mutation_micros": hot_phase_timing.authoritative_mutation_micros,
                        "history_resolution_micros": hot_phase_timing.history_resolution_micros,
                        "invariant_post_check_micros": hot_phase_timing.invariant_post_check_micros,
                        "artifact_assembly_micros": hot_phase_timing.artifact_assembly_micros,
                        "durable_append_micros": hot_phase_timing.durable_append_micros,
                        "publication_micros": hot_phase_timing.publication_micros,
                    },
                    "propagation_planning_micros": propagation_planning_micros,
                    "propagation_execution_micros": propagation_execution_micros,
                    "explicit_query_micros": explicit_query_micros,
                    "hot_changed_records": update.changed_records.len(),
                    "propagation_seed_count": propagation_seeds.len(),
                    "propagation_result_entities": propagation_outcome.result.entities.len(),
                    "propagation_result_relations": propagation_outcome.result.relations.len(),
                    "explicit_target_count": explicit_targets.len(),
                    "explicit_result_entities": explicit_outcome.result.entities.len(),
                    "diagnostic_artifact_count": diagnostic_artifact_count,
                    "detailed_trace_entries": detailed_trace_entries,
                    "counters": counters,
                })
            })
        },
    );
    emit_metric_summaries(
        suite,
        "hundred_k_nodes_geometry_profile_propagation_wave",
        &rich_propagation_wave_samples,
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
                "draft_preparation_micros",
                &["phase_timing", "draft_preparation_micros"],
            ),
            (
                "propagation_planning_micros",
                &["propagation_planning_micros"],
            ),
            (
                "propagation_execution_micros",
                &["propagation_execution_micros"],
            ),
            ("explicit_query_micros", &["explicit_query_micros"]),
            ("propagation_seed_count", &["propagation_seed_count"]),
            (
                "propagation_result_entities",
                &["propagation_result_entities"],
            ),
            (
                "propagation_result_relations",
                &["propagation_result_relations"],
            ),
            ("explicit_target_count", &["explicit_target_count"]),
            ("explicit_result_entities", &["explicit_result_entities"]),
            ("diagnostic_artifact_count", &["diagnostic_artifact_count"]),
            ("detailed_trace_entries", &["detailed_trace_entries"]),
        ],
    );
    assert!(rich_propagation_wave_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &rich_propagation_wave_samples,
        "rocketship geometry-profile propagation waves should preserve bounded mixed-locality execution while deferring hot detailed traces",
        |metrics| {
            let propagation_seed_count =
                metrics["propagation_seed_count"].as_u64().unwrap_or(0);
            let explicit_target_count =
                metrics["explicit_target_count"].as_u64().unwrap_or(0);
            metrics["resident_node_count"].as_u64() == Some(node_count as u64)
                && metrics["resident_relation_count"].as_u64().unwrap_or(0) >= node_count as u64
                && metrics["subsystem_count"].as_u64() == Some(12)
                && metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["propagation_result_entities"].as_u64().unwrap_or(0)
                    >= propagation_seed_count
                && metrics["propagation_result_relations"].as_u64().unwrap_or(0) >= 1
                && metrics["explicit_result_entities"].as_u64() == Some(explicit_target_count)
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "entity_slots_touched_by_commit") == 1
                && counter_u64(metrics, "query_packet_count") <= 32
                && counter_u64(metrics, "query_scope_unit_count")
                    <= propagation_seed_count + explicit_target_count
        },
    );
}
