use super::*;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_commit_delta_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let narrow_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema();
        commit_measurement(&runtime, |runtime| {
            measure_counted_work(execution, || {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(runtime);
                for index in 0..64 {
                    txn.push_batch(
                        batch_create(&format!("perf-entity-{index}")),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .expect("test staging stays within configured resource budgets");
                }
                txn.commit(execution, runtime)
                    .expect("single-partition create burst commit")
            })
        })
    });
    assert!(narrow_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &narrow_samples,
        "single-partition commits should remain sparse and clone-free",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "snapshot_pin_full_rebuilds") == 0
                && counter_u64(metrics, "partitions_touched_by_commit") == 1
                && metric_u64(metrics, "packet_count") <= 4
        },
    );

    let cross_partition_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema();
        let sources = (0..24)
            .map(|index| {
                create_entity_in_partition(
                    execution,
                    &runtime,
                    &format!("src-{index}"),
                    PartitionId(1),
                )
            })
            .collect::<Vec<_>>();
        let targets = (0..24)
            .map(|index| {
                create_entity_in_partition(
                    execution,
                    &runtime,
                    &format!("dst-{index}"),
                    PartitionId(7),
                )
            })
            .collect::<Vec<_>>();

        commit_measurement(&runtime, |runtime| {
            measure_counted_work(execution, || {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(runtime);
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
                txn.commit(execution, runtime)
                    .expect("cross-partition relation burst commit")
            })
        })
    });
    assert!(cross_partition_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
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

    let persisted_single_create_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_for_medium(&medium);
        let mut sample = commit_measurement(&runtime, |runtime| {
            measure_counted_work(execution, || {
                let mut txn =
                    crate::tests::support::test_owner_begin_transaction_for_main(runtime);
                txn.push_batch(
                    batch_create("persisted-single"),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("test staging stays within configured resource budgets");
                txn.commit(execution, runtime)
                    .expect("persisted single entity create")
            })
        });
        assert_eq!(
            medium.committed_count(),
            1,
            "persisted single create must append one canonical commit to the owned durable medium"
        );
        let latest = medium
            .latest_committed_artifact()
            .expect("owned durable medium must hold the appended commit artifact");
        assert!(!latest.as_bytes().is_empty());
        sample
            .metrics
            .insert_value("medium_committed_count", medium.committed_count());
        sample
    });
    assert!(persisted_single_create_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &persisted_single_create_samples,
        "persisted single creates should remain clone-free, single-partition, and durably appended to an owned medium",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "snapshot_pin_full_rebuilds") == 0
                && counter_u64(metrics, "partitions_touched_by_commit") == 1
                && metric_u64(metrics, "packet_count") <= 4
                && metric_u64(metrics, "durable_log_len") == 1
                && metric_u64(metrics, "medium_committed_count") == 1
                && metric_u64(metrics, "charged_work") > 0
        },
    );
}
