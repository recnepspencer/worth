use worth_relational::facade::runtime::RelationalRuntime;

pub(super) fn release_commit_snapshot(
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) {
    runtime
        .snapshots()
        .release_snapshot(snapshot)
        .expect("memory workspace releases its exact committed snapshot once");
}
