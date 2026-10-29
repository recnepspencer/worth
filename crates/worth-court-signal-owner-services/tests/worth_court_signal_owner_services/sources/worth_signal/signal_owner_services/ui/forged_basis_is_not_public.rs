use worth_signal::facade::branch::{AdmittedSignalBranchBasis, SignalBranchBasisDescriptor};

fn invalid_forged_basis(descriptor: SignalBranchBasisDescriptor) -> AdmittedSignalBranchBasis {
    AdmittedSignalBranchBasis(std::sync::Arc::new(descriptor))
}

fn valid_carriage(basis: AdmittedSignalBranchBasis) -> AdmittedSignalBranchBasis {
    basis
}

fn main() {}
