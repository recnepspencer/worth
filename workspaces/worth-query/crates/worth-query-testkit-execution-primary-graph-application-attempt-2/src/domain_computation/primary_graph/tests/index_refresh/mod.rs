use worth_relational::facade::history::BranchId;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;
pub fn commit_empty(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &str,
) -> RelationalCommitReceipt {
    let branch_id = BranchId(branch.to_owned());
    let identity = runtime
        .branch_identity(&branch_id)
        .expect("commit branch identity exists");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("commit branch basis is current");
    let committed = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("transaction binds to exact branch")
        .commit(execution, runtime)
        .expect("empty fixture transaction commits");
    let receipt = committed.commit.clone();
    worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::commit_snapshot_closeout::release_test_commit_snapshot(runtime, &committed);
    receipt
}
