use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::CommitResult;

pub fn release_test_commit_snapshot(runtime: &RelationalRuntime, committed: &CommitResult) {
    worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
        runtime,
        &committed.snapshot,
    );
}
