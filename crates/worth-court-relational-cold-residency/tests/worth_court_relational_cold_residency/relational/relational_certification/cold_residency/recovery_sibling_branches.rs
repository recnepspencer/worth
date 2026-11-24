//! Divergent siblings remain independent after restoring the main storage mirror.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::RecoveryVerificationMode, history::BranchId, mvcc::RelationalTransactionIntent,
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    durable_publication::{batch_create, create_entity, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn recovery_reads_both_divergent_branches_after_mirror_reuse() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let main = BranchId("main".into());
    let sibling = BranchId("reuse-sibling".into());
    let source = memory_runtime();
    create_entity(execution, &source, "reuse-sibling-seed");
    let (_, fork_source) = source.observe_fork_source(&main).unwrap();
    source
        .fork_branch(execution, sibling.clone(), fork_source)
        .unwrap();
    let basis = source
        .admit_branch_basis(&source.branch_identity(&sibling).unwrap())
        .unwrap();
    let mut transaction = source
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch_create("reuse-sibling-only"),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &source).unwrap();
    release_test_commit_snapshot(&source, &committed);
    create_entity(execution, &source, "reuse-main-only");
    let checkpoint = source.durability_authority().checkpoint(execution).unwrap();
    let mut plan = source
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.checkpoint = Some(checkpoint);

    let mut recovered = memory_runtime();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for (branch, expected) in [(main, 2), (sibling, 2)] {
        let identity = recovered.branch_identity(&branch).unwrap();
        let (_, basis) = recovered.observe_branch(&identity).unwrap();
        let snapshot = recovered
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
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
