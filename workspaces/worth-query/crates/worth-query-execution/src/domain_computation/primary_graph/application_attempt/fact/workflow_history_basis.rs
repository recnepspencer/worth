use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

/// Currentness is not permission to append. Replay-only observations deliberately
/// cannot authorize a new candidate, even at the original immutable basis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorkflowHistoryRequirement {
    Append { maximum_transitions: usize },
    Exact,
    ReplayOnly,
}

pub(super) fn remains_equal(
    runtime: &RelationalRuntime,
    current: &SnapshotHandle,
    observed: &SnapshotHandle,
    requirement: WorkflowHistoryRequirement,
    transition_count: usize,
) -> bool {
    // Handles are native-owner-issued. Their snapshot IDs name separate leases,
    // so compare the immutable truth axes, then validate the live candidate
    // handle with its owner. The application attempt retains the original basis.
    let capacity_available = match requirement {
        WorkflowHistoryRequirement::Append {
            maximum_transitions,
        } => transition_count < maximum_transitions,
        WorkflowHistoryRequirement::Exact => true,
        WorkflowHistoryRequirement::ReplayOnly => false,
    };
    capacity_available
        && current.runtime_instance_id() == observed.runtime_instance_id()
        && current.branch_id() == observed.branch_id()
        && current.version_id() == observed.version_id()
        && runtime.read_truth().project_snapshot(current).is_some()
}

#[cfg(test)]
mod tests;
