use super::*;

pub(super) fn certify_dense_fanout_wave(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let fanout_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
        let diagnostics_start = runtime.publication().diagnostic_artifacts().len();
        let source = create_entity_in_partition(execution, &runtime, "net-driver", PartitionId(7));
        let targets = (0..24)
            .map(|index| {
                let partition_id = match index % 4 {
                    0 => PartitionId(11),
                    1 => PartitionId(13),
                    2 => PartitionId(17),
                    _ => PartitionId(19),
                };
                create_entity_in_partition(
                    execution,
                    &runtime,
                    &format!("net-sink-{index}"),
                    partition_id,
                )
            })
            .collect::<Vec<_>>();

        runtime.performance_access().reset_counters();
        let (commit_outcome, commit_work) = measure_counted_work(execution, || {
            let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
            let mut batch = WorkerIntentBatch::new("chip-fanout-wave");
            for (index, target) in targets.iter().enumerate() {
                batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
                    crate::transactions::data::RelationSpec {
                        partition_id: PartitionId(29),
                        kind_id: KindId(2),
                        client_key: crate::symbols::data::ClientKey::raw(format!(
                            "chip-fanout-{index}"
                        )),
                        source: crate::transactions::data::EntityReference::Existing(source),
                        target: crate::transactions::data::EntityReference::Existing(*target),
                        fields: crate::transactions::data::AspectFieldPatch::default(),
                    },
                )));
            }
            txn.push_batch(
                batch,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("test staging stays within configured resource budgets");
            txn.commit(execution, &runtime)
                .expect("chip fanout relation burst commit")
        });
        let commit = runtime
            .history()
            .latest_commit()
            .expect("chip fanout commit")
            .clone();

        let (outgoing_relations, adjacency_work) = measure_counted_work(execution, || {
            runtime
                .storage_access()
                .outgoing_relations_for_entity(execution, source, commit.version_id)
                .expect("funded adjacency read succeeds")
        });

        let counters = runtime.performance_access().counters();
        let (diagnostic_artifact_count, detailed_trace_entries) =
            fresh_diagnostics_metrics(&runtime, diagnostics_start);

        CountedPerfSample {
            charged_work: commit_work + adjacency_work,
            metrics: perf_metrics!({
                "commit_work": commit_work,
                "adjacency_work": adjacency_work,
                "changed_records": commit_outcome.changed_records.len(),
                "dense_patch_record_count": dense_patch_record_count(&runtime),
                "outgoing_relation_count": outgoing_relations.len(),
                "diagnostic_artifact_count": diagnostic_artifact_count,
                "detailed_trace_entries": detailed_trace_entries,
                "profile_boundary": profile_boundary_metrics(
                    &runtime,
                    RelationalRuntimeProfile::ChipSimulation,
                ),
                "adjacency_backend": format!("{:?}", runtime.config().storage.adjacency_policy.backend),
                "counters": counters,
            }),
        }
    });
    assert!(fanout_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &fanout_samples,
        "chip fanout mutation wave should preserve compressed adjacency truth and dense patch shape",
        |metrics| {
            metrics["changed_records"].as_u64() == Some(24)
                && metrics["dense_patch_record_count"].as_u64() == Some(24)
                && metrics["outgoing_relation_count"].as_u64() == Some(24)
                && metrics["adjacency_backend"].as_str()
                    == Some(&format!(
                        "{:?}",
                        AdjacencyBackend::CompressedFanoutAdjacency
                    ))
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && metrics["profile_boundary"]["execution_lane_code"].as_u64() == Some(1)
                && metrics["profile_boundary"]["diagnostics_boundary_code"].as_u64() == Some(1)
                && metrics["profile_boundary"]["matches_defaults"].as_u64() == Some(1)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "relation_slots_touched_by_commit") == 24
        },
    );
}
