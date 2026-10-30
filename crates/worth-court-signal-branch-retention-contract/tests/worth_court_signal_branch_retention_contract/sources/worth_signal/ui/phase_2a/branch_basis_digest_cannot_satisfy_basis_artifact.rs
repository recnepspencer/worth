use worth_signal::facade::branch::AdmittedSignalBranchBasis;

fn require_admitted_branch_basis(_: AdmittedSignalBranchBasis) {}

fn main() {
    let digest: &str = "signal-branch-basis-digest";
    require_admitted_branch_basis(digest);
}
