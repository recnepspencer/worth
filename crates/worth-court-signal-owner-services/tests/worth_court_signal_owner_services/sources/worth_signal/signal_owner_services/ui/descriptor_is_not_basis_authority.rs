use worth_signal::facade::branch::{
    validate_signal_branch_name, SignalBranchBasisDescriptor, SignalOwnerCancellationSource,
    SignalOwnerServicePorts,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

fn invalid_descriptor_route(
    mutation: &worth_signal::facade::branch::SignalBranchMutationPort<(), (), (), (), ()>,
    descriptor: &SignalBranchBasisDescriptor,
) {
    let _ = mutation.fork_exact(
        validate_signal_branch_name("invalid-descriptor").expect("the identity is valid"),
        descriptor,
        &SignalOwnerCancellationSource::new().token(),
    );
}

fn valid_basis_route(
    mutation: &worth_signal::facade::branch::SignalBranchMutationPort<(), (), (), (), ()>,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) {
    let _ = mutation.fork_exact(
        validate_signal_branch_name("valid-basis").expect("the identity is valid"),
        basis,
        &SignalOwnerCancellationSource::new().token(),
    );
}

fn main() {
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the root basis is owner-issued");
    let services: Services = runtime.owner_component_services().expect("issuance");
    invalid_descriptor_route(&services.mutation_port(), basis.descriptor());
    valid_basis_route(&services.mutation_port(), &basis);
}
