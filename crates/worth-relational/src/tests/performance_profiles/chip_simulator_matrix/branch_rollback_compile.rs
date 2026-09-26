use super::*;

pub(super) fn certify_branch_rollback_compile_step_window(suite: &'static str) {
    let branch_rollback_compile_samples =
        capture_perf_samples(suite, "branch_rollback_compile_step_window", || {
            let feature_branch = BranchId("feature".to_string());
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();
            let source = create_entity_in_partition(&runtime, "rollback-driver", PartitionId(7));
            let stable_targets = (0..8)
                .map(|index| {
                    let partition_id = match index % 4 {
                        0 => PartitionId(11),
                        1 => PartitionId(13),
                        2 => PartitionId(17),
                        _ => PartitionId(19),
                    };
                    create_entity_in_partition(
                        &runtime,
                        &format!("rollback-stable-sink-{index}"),
                        partition_id,
                    )
                })
                .collect::<Vec<_>>();
            let transient_targets = (0..8)
                .map(|index| {
                    let partition_id = match index % 4 {
                        0 => PartitionId(23),
                        1 => PartitionId(31),
                        2 => PartitionId(37),
                        _ => PartitionId(41),
                    };
                    create_entity_in_partition(
                        &runtime,
                        &format!("rollback-transient-sink-{index}"),
                        partition_id,
                    )
                })
                .collect::<Vec<_>>();
            create_branch_from_main(&runtime, "feature");
            for (index, target) in stable_targets.iter().enumerate() {
                create_relation_in_partition_on_branch(
                    &runtime,
                    source,
                    *target,
                    &format!("rollback-stable-edge-{index}"),
                    "stable",
                    PartitionId(29),
                    feature_branch.clone(),
                );
            }

            runtime.performance_access().reset_counters();
            let mut txn = {
                let transaction_validation_input =
                    crate::tests::support::test_owner_transaction_validation_input_for_branch(
                        &runtime,
                        feature_branch.clone(),
                    );
                runtime
                    .begin_branch_transaction(
                        transaction_validation_input.basis(),
                        transaction_validation_input.intent().clone(),
                    )
                    .expect("owner-admitted transaction context")
            };
            let savepoint = txn.create_savepoint().unwrap();
            let mut transient_batch = WorkerIntentBatch::new("chip-transient-fanout");
            for (index, target) in transient_targets.iter().enumerate() {
                transient_batch = transient_batch.push(MutationIntent::Create(
                    CreateIntent::Relation(crate::transactions::data::RelationSpec {
                        partition_id: PartitionId(43),
                        kind_id: KindId(2),
                        client_key: crate::symbols::data::ClientKey::raw(format!(
                            "rollback-transient-edge-{index}"
                        )),
                        source: crate::transactions::data::EntityReference::Existing(source),
                        target: crate::transactions::data::EntityReference::Existing(*target),
                        fields: crate::transactions::data::AspectFieldPatch::default(),
                    }),
                ));
            }
            txn.push_batch(transient_batch)
                .expect("test staging stays within configured resource budgets");

            let rollback_started_at = Instant::now();
            let rollback = txn
                .rollback_to_savepoint(savepoint)
                .expect("chip savepoint rollback");
            let rollback_micros = rollback_started_at.elapsed().as_micros();

            txn.push_batch(WorkerIntentBatch::new("chip-committed-step").push(
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: source,
                        fields: crate::tests::support::aspect_field_patch_from_values([
                            (
                                crate::tests::support::aspect_key("name"),
                                crate::tests::support::field_key("name"),
                                crate::tests::support::string_aspect_value("rollback-driver"),
                            ),
                            (
                                crate::tests::support::aspect_key("step"),
                                crate::tests::support::field_key("step"),
                                crate::tests::support::u64_aspect_value(1),
                            ),
                            (
                                crate::tests::support::aspect_key("branch"),
                                crate::tests::support::field_key("branch"),
                                crate::tests::support::string_aspect_value("feature"),
                            ),
                        ]),
                    },
                )),
            ))
            .expect("test staging stays within configured resource budgets");
            let commit_started_at = Instant::now();
            let commit_outcome = txn.commit(&runtime).expect("chip branch step commit");
            let commit_micros = commit_started_at.elapsed().as_micros();

            let feature_commit = runtime
                .history()
                .branch_head(&feature_branch)
                .expect("feature branch head")
                .clone();
            let compile_started_at = Instant::now();
            let artifact = runtime
                .compiled_artifacts_authority()
                .compile_execution_artifact(
                    feature_commit.commit_id,
                    vec![
                        PartitionId(7),
                        PartitionId(11),
                        PartitionId(13),
                        PartitionId(17),
                        PartitionId(19),
                        PartitionId(29),
                    ],
                )
                .expect("feature branch compiled artifact");
            let compile_micros = compile_started_at.elapsed().as_micros();

            let adjacency_started_at = Instant::now();
            let outgoing_relations = runtime
                .storage_access()
                .outgoing_relations_for_entity(source, feature_commit.version_id);
            let adjacency_micros = adjacency_started_at.elapsed().as_micros();
            let (diagnostic_artifact_count, detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);

            PerfMeasurement {
                elapsed_micros: rollback_micros + commit_micros + compile_micros + adjacency_micros,
                metrics: perf_metrics!({
                    "rollback_micros": rollback_micros,
                    "commit_micros": commit_micros,
                    "compile_micros": compile_micros,
                    "adjacency_micros": adjacency_micros,
                    "rollback_effect_count": rollback.effect_count(),
                    "rollback_discarded_creations": rollback.summary.discarded_creation_count(),
                    "rollback_restored_records": rollback.summary.restored_record_count(),
                    "committed_changed_records": commit_outcome.changed_records.len(),
                    "outgoing_relation_count": outgoing_relations.len(),
                    "diagnostic_artifact_count": diagnostic_artifact_count,
                    "detailed_trace_entries": detailed_trace_entries,
                    "compiled_artifact_authority_status": format!(
                        "{:?}",
                        runtime
                            .compiled_artifacts()
                            .compiled_artifact_authority_status(artifact.artifact_id)
                    ),
                    "counters": runtime.performance_access().counters(),
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "branch_rollback_compile_step_window",
        &branch_rollback_compile_samples,
        &[
            ("rollback_micros", &["rollback_micros"]),
            ("commit_micros", &["commit_micros"]),
            ("compile_micros", &["compile_micros"]),
            ("adjacency_micros", &["adjacency_micros"]),
            ("rollback_effect_count", &["rollback_effect_count"]),
            (
                "rollback_discarded_creations",
                &["rollback_discarded_creations"],
            ),
            ("committed_changed_records", &["committed_changed_records"]),
            ("outgoing_relation_count", &["outgoing_relation_count"]),
        ],
    );
    assert!(branch_rollback_compile_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &branch_rollback_compile_samples,
        "chip branch rollback compile windows should discard abandoned fanout work and keep feature truth narrow",
        |metrics| {
            metrics["rollback_effect_count"].as_u64() == Some(8)
                && metrics["rollback_discarded_creations"].as_u64() == Some(8)
                && metrics["rollback_restored_records"].as_u64() == Some(0)
                && metrics["committed_changed_records"].as_u64() == Some(1)
                && metrics["outgoing_relation_count"].as_u64() == Some(8)
                && metrics["compiled_artifact_authority_status"].as_str()
                    == Some(&format!("{:?}", CompiledArtifactAuthorityStatus::Authoritative))
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );
}
