use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

/// Settle one exact snapshot opened by Query execution.
///
/// Callers invoke this only on terminal paths while holding the same runtime
/// authority that opened the handle. A denial is therefore an internal
/// lifecycle violation, not a pre-effect execution denial.
pub(crate) fn release_query_snapshot(runtime: &mut RelationalRuntime, snapshot: &SnapshotHandle) {
    runtime
        .snapshots()
        .release_snapshot(snapshot)
        .expect("Query releases each owned snapshot exactly once through its issuing runtime");
}
