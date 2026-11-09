//! Issue exact branch read handles and observe the selected fixture version.
use worth_relational::facade::{
    history::BranchId, identity::VersionId, runtime::RelationalRuntime, snapshots::SnapshotHandle,
};
pub(super) fn snapshot_for_owner_branch(
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
pub(super) fn main_selected_version(runtime: &RelationalRuntime) -> VersionId {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    basis.observation().version_id()
}
