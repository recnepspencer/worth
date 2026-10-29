pub use worth_relational::facade::runtime::RelationalRuntime;
pub use worth_relational::facade::transactions::CommitResult;

pub fn release_commit_snapshot(runtime: &mut RelationalRuntime, result: &CommitResult) {
    runtime
        .snapshots()
        .release_snapshot(&result.snapshot)
        .expect("fixture commit snapshot should close exactly once");
}
