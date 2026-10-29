use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

fn admit_raw_branch(runtime: &RelationalRuntime, branch: &BranchId) {
    let _ = runtime.admit_branch_basis(branch);
}

fn main() {}
