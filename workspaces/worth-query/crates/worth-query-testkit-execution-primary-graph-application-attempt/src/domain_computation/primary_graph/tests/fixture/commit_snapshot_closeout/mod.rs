use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::CommitResult;

pub fn release_test_commit_snapshot(runtime: &mut RelationalRuntime, committed: &CommitResult) {
    worth_query_execution::relational_snapshot_release::release_query_snapshot(
        runtime,
        &committed.snapshot,
    );
}
