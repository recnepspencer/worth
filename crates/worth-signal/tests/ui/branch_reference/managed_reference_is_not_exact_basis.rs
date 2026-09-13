use worth_signal::facade::{
    branch::{AdmittedSignalBranchBasis, ManagedSignalBranchReference},
    SignalRuntime,
};

fn valid_exact_advance(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    basis: &AdmittedSignalBranchBasis,
) {
    let _ = runtime.advance_signal_branch(&mut (), basis, |_| Ok(()));
}

fn managed_reference_cannot_replace_exact_basis(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    reference: &ManagedSignalBranchReference,
) {
    let _ = runtime.advance_signal_branch(&mut (), reference, |_| Ok(()));
}

fn main() {}
