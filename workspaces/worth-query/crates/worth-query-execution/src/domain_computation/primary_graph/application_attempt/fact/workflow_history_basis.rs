use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

pub(super) fn remains_equal(
    runtime: &RelationalRuntime,
    current: &SnapshotHandle,
    observed: &SnapshotHandle,
    maximum_transitions: usize,
    transition_count: usize,
) -> bool {
    // Handles are native-owner-issued. Their snapshot IDs name separate leases,
    // so compare the immutable truth axes, then validate the live candidate
    // handle with its owner. The application attempt retains the original basis.
    transition_count < maximum_transitions
        && current.runtime_instance_id() == observed.runtime_instance_id()
        && current.branch_id() == observed.branch_id()
        && current.version_id() == observed.version_id()
        && runtime.read_truth().project_snapshot(current).is_some()
}

#[cfg(test)]
mod tests;
