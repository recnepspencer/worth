pub(crate) use super::world::supply_chain::SupplyChainScale;
pub(crate) use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
pub(crate) use worth_relational::facade::branch::RelationalBranchIdentity;
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_relational::facade::inspection::RelationalBranchSharingObservation;
pub(crate) use worth_relational::facade::runtime::RelationalRuntime;

pub(crate) fn observe(
    runtime: &RelationalRuntime,
    branches: &[RelationalBranchIdentity],
) -> RelationalBranchSharingObservation {
    runtime
        .observe_branch_sharing(branches)
        .expect("the selected branches remain inspectable")
}

pub(crate) fn observe_seeded_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    scale: SupplyChainScale,
) -> RelationalBranchSharingObservation {
    let (world, expected) = certified_supply_chain_world(execution, scale);
    assert_oracle_matches(execution, &world, &expected);
    let main = world.runtime.main_branch_identity();
    observe(&world.runtime, &[main])
}

pub(crate) fn live_byte_lanes(
    observation: &RelationalBranchSharingObservation,
) -> [(&'static str, u64); 6] {
    [
        (
            "logical_branch_root_metadata_bytes",
            observation.logical_branch_root_metadata_bytes(),
        ),
        (
            "unique_physical_root_metadata_bytes",
            observation.unique_physical_root_metadata_bytes(),
        ),
        (
            "logical_branch_partition_payload_bytes",
            observation.logical_branch_partition_payload_bytes(),
        ),
        (
            "unique_physical_partition_payload_bytes",
            observation.unique_physical_partition_payload_bytes(),
        ),
        (
            "logical_branch_authoritative_bytes",
            observation.logical_branch_authoritative_bytes(),
        ),
        (
            "unique_physical_authoritative_bytes",
            observation.unique_physical_authoritative_bytes(),
        ),
    ]
}

pub(crate) fn fork_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    name: &str,
) {
    let (_, source_basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    runtime
        .fork_branch(execution, BranchId(name.to_owned()), source_basis)
        .expect("fork remains metadata-only");
}

pub(crate) fn branch_identity(runtime: &RelationalRuntime, name: &str) -> RelationalBranchIdentity {
    runtime
        .branch_identity(&BranchId(name.to_owned()))
        .expect("branch identity is owner-issued")
}
