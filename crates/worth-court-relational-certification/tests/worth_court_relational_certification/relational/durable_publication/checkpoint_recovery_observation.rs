//! Immutable owner observations used by public checkpoint journeys.
use worth_relational::facade::{
    durability::{DurabilityError, RelationalRecoveryStop},
    history::BranchId,
    runtime::RelationalRuntime,
    snapshots::SnapshotHandle,
};

pub(super) fn snapshot_for_branch(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> SnapshotHandle {
    let identity = runtime.branch_identity(branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap()
}

pub(super) fn failure(stop: RelationalRecoveryStop) -> DurabilityError {
    match stop {
        RelationalRecoveryStop::Durability(error) => error,
        RelationalRecoveryStop::AdmissionImage(cause) => {
            panic!("unexpected admission refusal: {cause:?}")
        }
        RelationalRecoveryStop::Request(cause) => panic!("unexpected request stop: {cause:?}"),
        RelationalRecoveryStop::Allocation(cause) => {
            panic!("unexpected allocation stop: {cause:?}")
        }
        RelationalRecoveryStop::Content(cause) => panic!("unexpected content refusal: {cause:?}"),
    }
}
