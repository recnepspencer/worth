use super::*;

pub(super) fn certify_hundred_k_nodes_zero_diagnostics_narrow_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
    node_count: usize,
    query_target_count: usize,
) {
    let zero_diagnostics_samples = capture_perf_samples(
        suite,
        "hundred_k_nodes_zero_diagnostics_narrow_round_trip",
        || {
            let mut runtime = runtime_with_test_schema_profile_and_chunks(
                RelationalRuntimeProfile::GeometryKernel,
                ROCKETSHIP_CHUNK_SIZE,
                ROCKETSHIP_CHUNK_SIZE,
            );
            runtime.configure_for_test(|config| {
                config.publication.policy.max_patch_records_per_commit = node_count * 2
            });
            runtime.configure_diagnostics_for_test(|profile| {
                profile.detailed_traces_enabled = false;
                profile.max_entries_per_artifact = 0;
            });
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();
            let seeded = seed_rocketship_world(&runtime, node_count);

            runtime.performance_access().reset_counters();
            let target_index = seeded.entities.len() / 2;
            let hot_update_started_at = Instant::now();
            let update = update_entity(
                &runtime,
                seeded.entities[target_index],
                "rocket-node-hot-update",
            );
            let hot_update_micros = hot_update_started_at.elapsed().as_micros();
            let hot_phase_timing = update.execution().phase_timing.clone();
            let snapshot = runtime.visibility_authority().snapshot();
            let half_window = query_target_count / 2;
            let window_start = target_index.saturating_sub(half_window);
            let window_end = (window_start + query_target_count).min(seeded.entities.len());
            let targets = seeded.entities[window_start..window_end]
                .iter()
                .copied()
                .map(RecordRef::Entity)
                .collect::<Vec<_>>();
            let packet = explicit_query_packet(&runtime, &snapshot, "rocketship-explicit", targets);

            let hot_query_plan_started_at = Instant::now();
            let planned = runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet)
                .expect("planned rocketship explicit query");
            let hot_query_planning_micros = hot_query_plan_started_at.elapsed().as_micros();
            let hot_query_execution_started_at = Instant::now();
            let outcome = runtime
                .read_truth()
                .execute_query_plan(
                    planned,
                    execution,
                    crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("rocketship explicit query outcome");
            let hot_query_execution_micros = hot_query_execution_started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();
            let (diagnostic_artifact_count, detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);

            let elapsed_micros = seeded.entity_commit_micros
                + seeded.relation_commit_micros
                + hot_update_micros
                + hot_query_planning_micros
                + hot_query_execution_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "resident_node_count": seeded.entities.len(),
                    "resident_relation_count": seeded.relation_count,
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
                    "hot_query_planning_micros": hot_query_planning_micros,
                    "hot_query_execution_micros": hot_query_execution_micros,
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
                        "publication_storage_commit_micros": hot_phase_timing.publication_storage_commit_micros,
                        "publication_index_refresh_micros": hot_phase_timing.publication_index_refresh_micros,
                        "publication_history_publish_micros": hot_phase_timing.publication_history_publish_micros,
                        "publication_visibility_pin_micros": hot_phase_timing.publication_visibility_pin_micros,
                        "publication_bundle_publish_micros": hot_phase_timing.publication_bundle_publish_micros,
                        "publication_post_commit_consumer_micros": hot_phase_timing.publication_post_commit_consumer_micros,
                    },
                    "hot_changed_records": update.changed_records.len(),
                    "query_target_count": window_end - window_start,
                    "query_result_entities": outcome.result.entities.len(),
                    "query_result_relations": outcome.result.relations.len(),
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
        "hundred_k_nodes_zero_diagnostics_narrow_round_trip",
        &zero_diagnostics_samples,
        &[
            ("resident_node_count", &["resident_node_count"]),
            ("resident_relation_count", &["resident_relation_count"]),
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
                "publication_storage_commit_micros",
                &["phase_timing", "publication_storage_commit_micros"],
            ),
            (
                "publication_index_refresh_micros",
                &["phase_timing", "publication_index_refresh_micros"],
            ),
            (
                "publication_history_publish_micros",
                &["phase_timing", "publication_history_publish_micros"],
            ),
            (
                "publication_visibility_pin_micros",
                &["phase_timing", "publication_visibility_pin_micros"],
            ),
            (
                "publication_bundle_publish_micros",
                &["phase_timing", "publication_bundle_publish_micros"],
            ),
            (
                "publication_post_commit_consumer_micros",
                &["phase_timing", "publication_post_commit_consumer_micros"],
            ),
            ("hot_query_planning_micros", &["hot_query_planning_micros"]),
            (
                "hot_query_execution_micros",
                &["hot_query_execution_micros"],
            ),
            ("query_target_count", &["query_target_count"]),
            ("query_result_entities", &["query_result_entities"]),
            ("diagnostic_artifact_count", &["diagnostic_artifact_count"]),
            ("detailed_trace_entries", &["detailed_trace_entries"]),
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
    assert!(zero_diagnostics_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &zero_diagnostics_samples,
        "rocketship zero-diagnostics should preserve a 100k-node resident world while keeping the hot path narrow and clone-free",
        |metrics| {
            let resident_node_count = metrics["resident_node_count"].as_u64().unwrap_or(0) as usize;
            let resident_relation_count =
                metrics["resident_relation_count"].as_u64().unwrap_or(0) as usize;
            let query_target_count = metrics["query_target_count"].as_u64().unwrap_or(0);
            metrics["resident_node_count"].as_u64() == Some(node_count as u64)
                && resident_relation_count >= resident_node_count.saturating_sub(1)
                && metrics["hot_changed_records"].as_u64() == Some(1)
                && metrics["query_result_entities"].as_u64() == Some(query_target_count)
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(0)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "entity_slots_touched_by_commit") == 1
                && counter_u64(metrics, "query_packet_count") <= 8
                && counter_u64(metrics, "query_scope_unit_count") <= query_target_count
        },
    );
}
