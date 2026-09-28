mod receipt;
mod selection;

use crate::facade::branch::AdmittedRelationalBranchBasis;
use crate::facade::history::BranchId;
use crate::facade::runtime::RelationalRuntime;

fn observe(runtime: &RelationalRuntime, branch: &str) -> AdmittedRelationalBranchBasis {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("branch identity");
    runtime
        .observe_branch(&identity)
        .expect("admitted branch basis")
        .1
}
