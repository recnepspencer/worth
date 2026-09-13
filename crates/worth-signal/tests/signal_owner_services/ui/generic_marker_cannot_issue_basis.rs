use worth_proof::{AuthorityMarker, AuthorityWitness};
use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, ManagedSignalBranchReference, SignalBranchBasisPort,
};

fn invalid_generic_authority<Auth: AuthorityMarker>(
    port: &SignalBranchBasisPort<(), (), ()>,
    authority: AuthorityWitness<Auth>,
) {
    let _reference: ManagedSignalBranchReference = port
        .issue_managed_branch_reference(&authority)
        .expect("this line is intentionally not type-correct");
}

fn valid_concrete_basis(
    port: &SignalBranchBasisPort<(), (), ()>,
    basis: &AdmittedSignalBranchBasis,
) {
    let _reference = port
        .issue_managed_branch_reference(basis)
        .expect("the owner-issued basis is the only issuance input");
}

fn main() {}
