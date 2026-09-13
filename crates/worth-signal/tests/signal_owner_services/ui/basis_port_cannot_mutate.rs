use worth_signal::facade::branch::SignalOwnerServicePorts;
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

fn invalid_basis_mutation(
    services: &Services,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) {
    let _ = services.basis_port().advance_exact(
        basis,
        &mut (),
        &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        |_| Ok(()),
    );
}

fn valid_mutation(
    services: &Services,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) {
    let _ = services.mutation_port().advance_exact(
        basis,
        &mut (),
        &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        |_| Ok(()),
    );
}

fn main() {
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the root basis is owner-issued");
    let services = runtime.owner_component_services().expect("issuance");
    invalid_basis_mutation(&services, &basis);
    valid_mutation(&services, &basis);
}
