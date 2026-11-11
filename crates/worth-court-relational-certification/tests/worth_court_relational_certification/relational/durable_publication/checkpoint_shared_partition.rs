//! Original pinned readers and divergent branch truth across actual source recovery.
use super::branch_history_publications::fork;
use super::branch_history_publications::transaction;
use super::checkpoint_recovery_observation::snapshot_for_branch as snapshot_for_owner_branch;
use super::*;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::transactions::BulkEntityCreateIntent;
#[test]
fn divergent_sibling_and_main_restore_the_shared_partition_and_keep_the_original_pinned_reader() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "partition-alias-seed");
    let mut transaction = transaction(&runtime, &BranchId("main".into()));
    transaction
        .push_batch(
            WorkerIntentBatch::new("shared-second-partition").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId(41),
                    kind_id: KindId(1),
                    client_keys: vec!["shared-secondary".into()],
                    field_patches: vec![field_patch("name", "name", "shared-secondary")],
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let second = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &second);
    let main = BranchId("main".into());
    let pinned = snapshot_for_owner_branch(&runtime, &main);
    let sibling = {
        fork(execution, &runtime, "partition-alias-sibling");
        BranchId("partition-alias-sibling".into())
    };
    let on_sibling = super::checkpoint_root_publications::create_on_branch(
        execution,
        &runtime,
        "sibling-only",
        sibling.clone(),
    );
    release_test_commit_snapshot(&runtime, &on_sibling);
    let on_main = create_entity_outcome(execution, &runtime, "main-only");
    release_test_commit_snapshot(&runtime, &on_main);

    medium.file_checkpoint(capture(execution, &runtime));
    assert_eq!(
        runtime
            .read_truth()
            .read_snapshot(&pinned, execution)
            .expect("read request remains live")
            .unwrap()
            .entities()
            .len(),
        2
    );

    let (recovered, _) = recover(execution, &medium);
    for branch in [main, sibling] {
        let snapshot = snapshot_for_owner_branch(&recovered, &branch);
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot, execution)
                .expect("read request remains live")
                .unwrap()
                .entities()
                .len(),
            3
        );
    }
}
