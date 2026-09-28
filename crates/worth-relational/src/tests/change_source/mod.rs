mod observation_reads;
mod receipt;
mod runtime_handle;
mod selection;

use crate::facade::branch::AdmittedRelationalBranchBasis;
use crate::facade::history::{BranchId, CommitId};
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

/// The work of one full walk from `head` down a linear history of `N`
/// commits: `N` node visits, `N` catalog probes, and `N - 1` parent edges.
fn linear_ancestry_work(runtime: &RelationalRuntime, head: CommitId) -> usize {
    3 * runtime
        .history()
        .ancestor_closure_by_commit_id_order(head)
        .len()
        - 1
}
