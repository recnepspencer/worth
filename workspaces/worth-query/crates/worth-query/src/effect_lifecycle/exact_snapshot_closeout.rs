pub(super) fn release_exact_execution_snapshot(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) {
    runtime
        .snapshots()
        .release_snapshot(snapshot)
        .expect("effect execution releases each exact snapshot once through its issuing runtime");
}
