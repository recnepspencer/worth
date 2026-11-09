use worth_relational::facade::branch::AdmittedRelationalBranchBasis;

fn reconstruct(value: AdmittedRelationalBranchBasis) -> AdmittedRelationalBranchBasis {
    AdmittedRelationalBranchBasis { ..value }
}

fn main() {
    let _ = reconstruct;
}
