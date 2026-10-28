use crate::facade::history::BranchId;

use super::super::fixture::FintechWorld;

pub(crate) fn open_analysis_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut FintechWorld,
) -> BranchId {
    world.create_analysis_branch(execution)
}

pub(crate) fn open_audit_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut FintechWorld,
) -> BranchId {
    let branch = BranchId("audit".to_string());
    world
        .runtime
        .history_authority()
        .fork_branch_from(execution, branch.clone(), &BranchId("main".to_string()))
        .unwrap();
    branch
}
