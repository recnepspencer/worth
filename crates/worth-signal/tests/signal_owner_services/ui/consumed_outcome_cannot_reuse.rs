use worth_signal::facade::branch::{SignalOwnerCancellationSource, SignalOwnerServicePorts};
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

fn invalid_consumed_outcome(
    services: &Services,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) {
    let outcome = services
        .mutation_port()
        .advance_exact(
            basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the outcome exists before the disputed consume");
    let _next_basis = outcome.into_basis();
    let _transaction = outcome.transaction();
}

fn valid_borrow_before_consume(
    services: &Services,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) {
    let outcome = services
        .mutation_port()
        .advance_exact(
            basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the outcome exists");
    let _transaction = outcome.transaction();
    let _next_basis = outcome.into_basis();
}

fn main() {
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the root basis is owner-issued");
    let services: Services = runtime.owner_component_services().expect("issuance");
    invalid_consumed_outcome(&services, &basis);
    valid_borrow_before_consume(&services, &basis);
}
