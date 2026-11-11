//! Original pinned readers and divergent branch truth across actual source recovery.
use super::branch_history_publications::fork;
use super::checkpoint_recovery_observation::snapshot_for_branch as snapshot_for_owner_branch;
use super::*;
use worth_relational::facade::history::BranchId;
#[test]
fn shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    create_entity(execution, &source, "reuse-reader-seed");
    medium.file_checkpoint(capture(execution, &source));
    let (recovered, _) = recover(execution, &medium);

    let main = BranchId("main".into());
    let pinned = snapshot_for_owner_branch(&recovered, &main);
    create_entity(execution, &recovered, "reuse-reader-next");
    assert_eq!(
        recovered
            .read_truth()
            .read_snapshot(&pinned, execution)
            .expect("read request remains live")
            .unwrap()
            .entities()
            .len(),
        1
    );
    let latest = snapshot_for_owner_branch(&recovered, &main);
    assert_eq!(
        recovered
            .read_truth()
            .read_snapshot(&latest, execution)
            .expect("read request remains live")
            .unwrap()
            .entities()
            .len(),
        2
    );
}

#[test]
fn divergent_main_and_sibling_keep_their_original_truth_after_source_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    create_entity(execution, &source, "reuse-sibling-seed");
    let sibling = {
        fork(execution, &source, "reuse-sibling");
        BranchId("reuse-sibling".into())
    };
    let sibling_commit = super::checkpoint_root_publications::create_on_branch(
        execution,
        &source,
        "reuse-sibling-only",
        sibling.clone(),
    );
    release_test_commit_snapshot(&source, &sibling_commit);
    let main_commit = create_entity_outcome(execution, &source, "reuse-main-only");
    release_test_commit_snapshot(&source, &main_commit);
    let checkpoint = capture(execution, &source);
    medium.file_checkpoint(checkpoint);
    let (recovered, _) = recover(execution, &medium);
    for (branch, expected) in [(BranchId("main".into()), 2), (sibling, 2)] {
        let snapshot = snapshot_for_owner_branch(&recovered, &branch);
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot, execution)
                .expect("read request remains live")
                .unwrap()
                .entities()
                .len(),
            expected
        );
    }
}
