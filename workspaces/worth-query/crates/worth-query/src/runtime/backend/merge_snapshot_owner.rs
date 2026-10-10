/// Backend-owned closeout for the exact snapshot published by a relational
/// merge.
///
/// Backends that execute merges override the default invariant failure and
/// release through the same Relational runtime that issued the snapshot.
#[doc(hidden)]
pub trait WorthQueryMergeSnapshotOwner {
    fn release_query_merge_snapshot(
        &mut self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        panic!("a backend that performs a relational merge must settle its published snapshot")
    }
}
