use worth_signal::facade::{branch::AdmittedSignalBranchBasis, OutputIdentity};

fn require_admitted_branch_basis(_: AdmittedSignalBranchBasis) {}

fn main() {
    let token = OutputIdentity::new("host-output");
    require_admitted_branch_basis(token);
}
