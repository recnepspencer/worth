use super::*;

pub(crate) fn current_snapshot_version(runtime: &RelationalRuntime, branch: &BranchId) -> u64 {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    snapshot_for_supply_chain_identity(runtime, &identity)
        .version_id()
        .0
}
