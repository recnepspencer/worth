use super::*;

pub(super) fn certify_trade_correction_analysis_round_trip(suite: &'static str) {
    let trade_correction_samples =
        capture_perf_samples(suite, "trade_correction_analysis_round_trip", || {
            let runtime = persisted_runtime_with_test_schema();
            let account =
                create_entity_in_partition(&runtime, "portfolio-account", PartitionId(10));
            create_branch_from_main(&runtime, "analysis");

            runtime.performance_access().reset_counters();
            let analysis_commit_started_at = Instant::now();
            let analysis_commit = {
                let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
                    &runtime,
                    BranchId("analysis".to_string()),
                );
                txn.push_batch(WorkerIntentBatch::new("correct-trade").push(
                    MutationIntent::Create(CreateIntent::Entity(
                        crate::transactions::data::EntitySpec {
                            partition_id: PartitionId(10),
                            kind_id: KindId(1),
                            client_key: crate::symbols::data::ClientKey::raw(
                                "analysis-trade-correction".to_string(),
                            ),
                            fields: crate::tests::support::string_aspect_field_patch([
                                (
                                    crate::tests::support::aspect_key("entity_type"),
                                    crate::tests::support::field_key("entity_type"),
                                    "trade",
                                ),
                                (
                                    crate::tests::support::aspect_key("case"),
                                    crate::tests::support::field_key("case"),
                                    "trade-correction",
                                ),
                                (
                                    crate::tests::support::aspect_key("status"),
                                    crate::tests::support::field_key("status"),
                                    "corrected",
                                ),
                                (
                                    crate::tests::support::aspect_key("account"),
                                    crate::tests::support::field_key("account"),
                                    "portfolio-account",
                                ),
                            ]),
                        },
                    )),
                ))
                .expect("test staging stays within configured resource budgets");
                txn.push_batch(WorkerIntentBatch::new("refresh-risk").push(
                    MutationIntent::Create(CreateIntent::Entity(
                        crate::transactions::data::EntitySpec {
                            partition_id: PartitionId(30),
                            kind_id: KindId(1),
                            client_key: crate::symbols::data::ClientKey::raw(
                                "analysis-risk-refresh".to_string(),
                            ),
                            fields: crate::tests::support::string_aspect_field_patch([
                                (
                                    crate::tests::support::aspect_key("entity_type"),
                                    crate::tests::support::field_key("entity_type"),
                                    "risk_view",
                                ),
                                (
                                    crate::tests::support::aspect_key("case"),
                                    crate::tests::support::field_key("case"),
                                    "trade-correction",
                                ),
                                (
                                    crate::tests::support::aspect_key("status"),
                                    crate::tests::support::field_key("status"),
                                    "refreshed",
                                ),
                                (
                                    crate::tests::support::aspect_key("severity"),
                                    crate::tests::support::field_key("severity"),
                                    "medium",
                                ),
                            ]),
                        },
                    )),
                ))
                .expect("test staging stays within configured resource budgets");
                txn.push_batch(
                    WorkerIntentBatch::new("emit-audit").push(MutationIntent::Create(
                        CreateIntent::Entity(crate::transactions::data::EntitySpec {
                            partition_id: PartitionId(40),
                            kind_id: KindId(1),
                            client_key: crate::symbols::data::ClientKey::raw(
                                "analysis-audit-record".to_string(),
                            ),
                            fields: crate::tests::support::string_aspect_field_patch([
                                (
                                    crate::tests::support::aspect_key("entity_type"),
                                    crate::tests::support::field_key("entity_type"),
                                    "audit_record",
                                ),
                                (
                                    crate::tests::support::aspect_key("case"),
                                    crate::tests::support::field_key("case"),
                                    "trade-correction",
                                ),
                                (
                                    crate::tests::support::aspect_key("event"),
                                    crate::tests::support::field_key("event"),
                                    "analysis-reviewed",
                                ),
                            ]),
                        }),
                    )),
                )
                .expect("test staging stays within configured resource budgets");
                txn.commit(&runtime)
                    .expect("analysis branch correction commit")
            };
            let analysis_commit_micros = analysis_commit_started_at.elapsed().as_micros();
            let analysis_entities = changed_entities(&analysis_commit);
            let trade = analysis_entities[0];
            let risk_view = analysis_entities[1];
            let audit_record = analysis_entities[2];

            let prepared = runtime
                .prepare_merge_execution(MergeExecutionRequest {
                    target_branch: BranchId("main".to_string()),
                    source_branch: BranchId("analysis".to_string()),
                    merge_intent: MergeIntent::ReconcileIntoTarget,
                })
                .expect("prepared analysis merge");
            let merge_started_at = Instant::now();
            let merge_outcome = runtime
                .execute_prepared_merge(prepared)
                .expect("analysis merge execution");
            let merge_execute_micros = merge_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "trade-correction-round-trip",
                vec![
                    RecordRef::Entity(account),
                    RecordRef::Entity(trade),
                    RecordRef::Entity(risk_view),
                    RecordRef::Entity(audit_record),
                ],
            );
            let query_started_at = Instant::now();
            let query_outcome = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, packet)
                        .expect("planned workflow query"),
                )
                .expect("workflow query outcome");
            let query_round_trip_micros = query_started_at.elapsed().as_micros();

            let elapsed_micros =
                analysis_commit_micros + merge_execute_micros + query_round_trip_micros;
            let counters = runtime.performance_access().counters();

            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "analysis_changed_records": analysis_commit.changed_records.len(),
                    "merged_changed_records": merge_outcome.commit.changed_records.len(),
                    "query_entities": query_outcome.result.entities.len(),
                    "query_relations": query_outcome.result.relations.len(),
                    "profile_boundary": profile_boundary_metrics(
                        &runtime,
                        RelationalRuntimeProfile::CertificationCore,
                    ),
                    "phase_timing": {
                        "analysis_commit_micros": analysis_commit_micros,
                        "merge_execute_micros": merge_execute_micros,
                        "query_round_trip_micros": query_round_trip_micros,
                    },
                    "counters": counters,
                })
            })
        });
    emit_metric_summaries(
        suite,
        "trade_correction_analysis_round_trip",
        &trade_correction_samples,
        &[
            (
                "analysis_commit_micros",
                &["phase_timing", "analysis_commit_micros"],
            ),
            (
                "merge_execute_micros",
                &["phase_timing", "merge_execute_micros"],
            ),
            (
                "query_round_trip_micros",
                &["phase_timing", "query_round_trip_micros"],
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
    assert!(trade_correction_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &trade_correction_samples,
        "workflow round trips should stay branch-local, merge one analysis patch, and query a narrow case surface",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "merge_execution_attempts") == 1
                && counter_u64(metrics, "partitions_touched_by_commit") <= 3
                && counter_u64(metrics, "query_packet_count") <= 3
                && metrics["analysis_changed_records"].as_u64() == Some(3)
                && metrics["merged_changed_records"].as_u64() == Some(3)
                && metrics["query_entities"].as_u64() == Some(4)
                && metrics["query_relations"].as_u64() == Some(0)
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(2)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(1)
        },
    );
}
