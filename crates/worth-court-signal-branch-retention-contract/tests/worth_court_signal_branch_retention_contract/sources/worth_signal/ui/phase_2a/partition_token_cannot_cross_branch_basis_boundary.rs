use worth_signal::facade::{branch::AdmittedSignalBranchBasis, PartitionToken};

fn require_admitted_branch_basis(_: AdmittedSignalBranchBasis) {}

fn main() {
    let token = PartitionToken::new("host-partition");
    require_admitted_branch_basis(token);
}
