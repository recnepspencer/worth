use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_commit_delta_matrix() {
    let suite = "commit_delta_matrix";

    let narrow_samples = capture_perf_samples(suite, "single_partition_create_burst", || {
        let runtime = runtime_with_test_schema();
        commit_measurement(&runtime, |runtime| {
            let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
            for index in 0..64 {
                txn.push_batch(
                    batch_create(&format!("perf-entity-{index}")),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("test staging stays within configured resource budgets");
            }
            txn.commit(
                runtime,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("single-partition create burst commit")
        })
    });
    assert!(narrow_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &narrow_samples,
        "single-partition commits should remain sparse and clone-free",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "snapshot_pin_full_rebuilds") == 0
                && counter_u64(metrics, "partitions_touched_by_commit") == 1
                && metric_u64(metrics, "packet_count") <= 4
        },
    );

    let cross_partition_samples =
        capture_perf_samples(suite, "cross_partition_relation_burst", || {
            let runtime = runtime_with_test_schema();
            let sources = (0..24)
                .map(|index| {
                    create_entity_in_partition(&runtime, &format!("src-{index}"), PartitionId(1))
                })
                .collect::<Vec<_>>();
            let targets = (0..24)
                .map(|index| {
                    create_entity_in_partition(&runtime, &format!("dst-{index}"), PartitionId(7))
                })
                .collect::<Vec<_>>();

            commit_measurement(&runtime, |runtime| {
                let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
                let mut batch = WorkerIntentBatch::new("cross-partition-relations");
                for (index, (source, target)) in sources.iter().zip(targets.iter()).enumerate() {
                    batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
                        crate::transactions::data::RelationSpec {
                            partition_id: PartitionId(9),
                            kind_id: KindId(2),
                            client_key: crate::symbols::data::ClientKey::raw(format!(
                                "cross-{index}"
                            )),
                            source: crate::transactions::data::EntityReference::Existing(*source),
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
                txn.commit(
                    runtime,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("cross-partition relation burst commit")
            })
        });
    assert!(cross_partition_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &cross_partition_samples,
        "cross-partition relation bursts should avoid global cloning and stay packet-bounded",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "snapshot_pin_full_rebuilds") == 0
                && metric_u64(metrics, "touched_partitions") <= 3
                && counter_u64(metrics, "bulk_mutation_cross_partition_relation_count") == 24
                && metric_u64(metrics, "packet_count") <= 8
        },
    );

    let persisted_single_create_samples =
        capture_perf_samples(suite, "persisted_single_entity_create", || {
            let runtime = persisted_runtime_with_test_schema();
            commit_measurement(&runtime, |runtime| {
                let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
                txn.push_batch(
                    batch_create("persisted-single"),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("test staging stays within configured resource budgets");
                txn.commit(
                    runtime,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("persisted single entity create")
            })
        });
    emit_metric_summaries(
        suite,
        "persisted_single_entity_create",
        &persisted_single_create_samples,
        &[
            (
                "working_state_preparation_micros",
                &["phase_timing", "working_state_preparation_micros"],
            ),
            (
                "authoritative_mutation_micros",
                &["phase_timing", "authoritative_mutation_micros"],
            ),
            (
                "artifact_assembly_micros",
                &["phase_timing", "artifact_assembly_micros"],
            ),
            (
                "durable_append_micros",
                &["phase_timing", "durable_append_micros"],
            ),
            (
                "publication_micros",
                &["phase_timing", "publication_micros"],
            ),
        ],
    );
    assert!(persisted_single_create_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &persisted_single_create_samples,
        "persisted single creates should remain clone-free and single-partition",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "snapshot_pin_full_rebuilds") == 0
                && counter_u64(metrics, "partitions_touched_by_commit") == 1
                && metric_u64(metrics, "packet_count") <= 4
                && metrics["phase_timing"]["durable_append_micros"]
                    .as_u64()
                    .unwrap_or(0)
                    > 0
        },
    );
}
