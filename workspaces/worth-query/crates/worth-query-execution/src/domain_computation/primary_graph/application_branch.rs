use worth_relational::facade::history::BranchId;
use worth_runtime_bridge::facade::TruthBranchIdentity;

/// Sole ordinary-branch name for the single-branch application limit.
/// All `BranchId("main")` / `"main"` branch construction must route through this
/// module (PB4).
pub(in crate::domain_computation) const PRIMARY_APPLICATION_BRANCH: &str = "main";

pub(in crate::domain_computation) fn primary_relational_branch_id() -> BranchId {
    BranchId(PRIMARY_APPLICATION_BRANCH.to_owned())
}

pub(in crate::domain_computation) fn primary_truth_branch_identity() -> TruthBranchIdentity {
    TruthBranchIdentity::from_relational_branch_id(PRIMARY_APPLICATION_BRANCH)
}
