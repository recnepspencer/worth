use worth_signal::facade::branch::{
    validate_signal_branch_name, SignalOwnerCancellationSource, SignalOwnerServicePorts,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

fn invalid_raw_id_route(
    mutation: &worth_signal::facade::branch::SignalBranchMutationPort<(), (), (), (), ()>,
    branch_id: &worth_signal::facade::history::RuntimeBranchId,
) {
    let _ = mutation.fork_exact(
        validate_signal_branch_name("invalid-raw-id").expect("the identity is valid"),
        branch_id,
        &SignalOwnerCancellationSource::new().token(),
    );
}

fn valid_basis_route(
    mutation: &worth_signal::facade::branch::SignalBranchMutationPort<(), (), (), (), ()>,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) {
    let _ = mutation.fork_exact(
        validate_signal_branch_name("valid-raw-id").expect("the identity is valid"),
        basis,
        &SignalOwnerCancellationSource::new().token(),
    );
}

fn main() {
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let branch_id = runtime.current_branch().id;
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the root basis is owner-issued");
    let services: Services = runtime.owner_component_services().expect("issuance");
    invalid_raw_id_route(&services.mutation_port(), &branch_id);
    valid_basis_route(&services.mutation_port(), &basis);
}
