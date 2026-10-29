use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

fn admit_owner_identity(runtime: &RelationalRuntime, branch: &BranchId) {
    let identity = runtime
        .branch_identity(branch)
        .expect("the owner issues identity only for a registered branch");
    let _basis = runtime
        .admit_branch_basis(&identity)
        .expect("the owner admits its exact identity");
}

fn main() {
    let _ = admit_owner_identity;
}
