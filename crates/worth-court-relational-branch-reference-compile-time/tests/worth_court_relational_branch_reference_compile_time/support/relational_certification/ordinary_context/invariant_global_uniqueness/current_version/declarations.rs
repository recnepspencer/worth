pub(crate) use {worth_relational::facade::history::BranchId, worth_relational::facade::runtime::RelationalRuntime, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity};

pub(crate) fn current_snapshot_version(runtime: &RelationalRuntime, branch: &BranchId) -> u64 {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    snapshot_for_supply_chain_identity(runtime, &identity)
        .version_id()
        .0
}