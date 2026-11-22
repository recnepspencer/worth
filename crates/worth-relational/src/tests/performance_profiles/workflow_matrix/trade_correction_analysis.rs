use super::*;

fn trade_correction_schema_registry() -> crate::facade::schema::RelationalSchemaRegistry {
    let mut registry = crate::tests::support::test_schema_registry();
    for (kind_id, kind_name, fields) in [
        (
            101,
            "workflow.trade",
            &["entity_type", "case", "status", "account"][..],
        ),
        (
            102,
            "workflow.risk_view",
            &["entity_type", "case", "status", "severity"][..],
        ),
        (
            103,
            "workflow.audit_record",
            &["entity_type", "case", "event"][..],
        ),
    ] {
        let mut aspects = fields
            .iter()
            .map(|key| {
                crate::tests::support::entity_field_aspect(
                    crate::tests::support::aspect_key(key),
                    crate::tests::support::field_key(key),
                )
            })
            .collect::<Vec<_>>();
        aspects.push(crate::tests::support::lifecycle_aspect());
        registry = registry
            .register_entity_kind(crate::facade::schema::EntityKindRegistration {
                kind_id: KindId(kind_id),
                kind_name: kind_name.to_string(),
                schema_id: crate::facade::schema::SchemaId("test".to_string()),
                schema_version_id: crate::facade::schema::SchemaVersionId(1),
                aspect_contract_declarations:
                    crate::facade::schema::KindAspectContractDeclarations::new(aspects),
            })
            .expect("workflow entity kind registration");
    }
    registry
}

pub(super) fn certify_trade_correction_analysis_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let trade_correction_samples = capture_counted_samples(|| {
        let runtime = runtime_with_schema_registry_profile(
            RelationalRuntimeProfile::CertificationCore,
            trade_correction_schema_registry(),
        );
        let account = create_entity_in_partition(
            execution,
            &runtime,
            "portfolio-account",
            PartitionId(10),
        );
        create_branch_from_main(&runtime, "analysis");

        runtime.performance_access().reset_counters();
        let (analysis_commit, analysis_commit_work) = measure_counted_work(execution, || {
            let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
                &runtime,
                BranchId("analysis".to_string()),
            );
            txn.push_batch(
                WorkerIntentBatch::new("correct-trade").push(MutationIntent::Create(
                    CreateIntent::Entity(crate::transactions::data::EntitySpec {
                        partition_id: PartitionId(10),
                        kind_id: KindId(101),
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
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("test staging stays within configured resource budgets");
            txn.push_batch(
                WorkerIntentBatch::new("refresh-risk").push(MutationIntent::Create(
                    CreateIntent::Entity(crate::transactions::data::EntitySpec {
                        partition_id: PartitionId(30),
                        kind_id: KindId(102),
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
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("test staging stays within configured resource budgets");
            txn.push_batch(
                WorkerIntentBatch::new("emit-audit").push(MutationIntent::Create(
                    CreateIntent::Entity(crate::transactions::data::EntitySpec {
                        partition_id: PartitionId(40),
                        kind_id: KindId(103),
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
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("test staging stays within configured resource budgets");
            txn.commit(execution, &runtime)
                .expect("analysis branch correction commit")
        });
        let analysis_entities = changed_entities(&analysis_commit);
        assert_eq!(analysis_entities.len(), 3);

        let prepared = runtime
            .prepare_merge_execution(
                execution,
                MergeExecutionRequest {
                    target_branch: BranchId("main".to_string()),
                    source_branch: BranchId("analysis".to_string()),
                    merge_intent: MergeIntent::ReconcileIntoTarget,
                },
            )
            .expect("prepared analysis merge");
        let (merge_outcome, merge_execute_work) = measure_counted_work(execution, || {
            runtime
                .execute_prepared_merge(execution, prepared)
                .expect("analysis merge execution")
        });
        let merged_entities = changed_entities(&merge_outcome.commit);
        assert_eq!(merged_entities.len(), 3);
        let trade = merged_entities[0];
        let risk_view = merged_entities[1];
        let audit_record = merged_entities[2];

        let snapshot = runtime
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");
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
        let (query_outcome, query_round_trip_work) = measure_counted_work(execution, || {
            runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, packet)
                        .expect("planned workflow query"),
                    execution,
                )
                .expect("request remains live")
                .map(|read| read.outcome)
                .expect("workflow query outcome")
        });

        let counters = runtime.performance_access().counters();
        CountedPerfSample {
            charged_work: analysis_commit_work + merge_execute_work + query_round_trip_work,
            metrics: perf_metrics!({
                "analysis_changed_records": analysis_commit.changed_records.len(),
                "merged_changed_records": merge_outcome.commit.changed_records.len(),
                "query_entities": query_outcome.result.entities.len(),
                "query_relations": query_outcome.result.relations.len(),
                "profile_boundary": profile_boundary_metrics(
                    &runtime,
                    RelationalRuntimeProfile::CertificationCore,
                ),
                "phase_work": {
                    "analysis_commit_work": analysis_commit_work,
                    "merge_execute_work": merge_execute_work,
                    "query_round_trip_work": query_round_trip_work,
                },
                "counters": counters,
            }),
        }
    });
    assert!(trade_correction_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
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
