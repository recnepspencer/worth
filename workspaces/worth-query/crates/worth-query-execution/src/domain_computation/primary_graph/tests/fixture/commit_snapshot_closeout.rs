use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::CommitResult;

pub(in crate::domain_computation::primary_graph) fn release_test_commit_snapshot(
    runtime: &RelationalRuntime,
    committed: &CommitResult,
) {
    crate::relational_snapshot_release::release_query_snapshot(runtime, &committed.snapshot);
}
