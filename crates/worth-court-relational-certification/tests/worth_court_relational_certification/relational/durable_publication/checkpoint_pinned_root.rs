//! Original pinned readers and divergent branch truth across actual source recovery.
use super::branch_history_publications::fork;
use super::checkpoint_recovery_observation::snapshot_for_branch as snapshot_for_owner_branch;
use super::*;
use worth_relational::facade::history::BranchId;
#[test]
fn native_checkpoint_keeps_the_original_pinned_reader_after_revisions_and_sibling_writes() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    let entity =
        super::checkpoint_root_publications::create_entity(execution, &source, "work-seed");
    let _small = source
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    super::checkpoint_root_publications::create_in_partition(
        execution,
        &source,
        "work-secondary",
        PartitionId(41),
    );
    let _grown = source
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let pinned = snapshot_for_owner_branch(&source, &BranchId("main".into()));
    super::checkpoint_root_publications::update_entity(execution, &source, entity, "work-revised");
    let sibling = {
        fork(execution, &source, "work-sibling");
        BranchId("work-sibling".into())
    };
    let sibling_commit = super::checkpoint_root_publications::create_on_branch(
        execution,
        &source,
        "work-fork",
        sibling,
    );
    release_test_commit_snapshot(&source, &sibling_commit);
    let native = source
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &native)
        .unwrap();
    assert_eq!(
        source
            .read_truth()
            .read_snapshot(&pinned, execution)
            .expect("read request remains live")
            .unwrap()
            .entities()
            .len(),
        2
    );
}
