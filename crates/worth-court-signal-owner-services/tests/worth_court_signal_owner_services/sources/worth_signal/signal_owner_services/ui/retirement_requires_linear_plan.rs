use worth_signal::facade::branch::{
    SignalBranchRetirementReason, SignalOwnerCancellationSource, SignalOwnerServicePorts,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

fn invalid_retirement_input(
    services: &Services,
    basis: worth_signal::facade::branch::AdmittedSignalBranchBasis,
) {
    let _ = services
        .lifecycle_port()
        .retire_exact(basis, &SignalOwnerCancellationSource::new().token());
}

fn valid_retirement_input(
    services: &Services,
    basis: worth_signal::facade::branch::AdmittedSignalBranchBasis,
) {
    let plan = services
        .lifecycle_port()
        .plan_retirement_exact(basis, SignalBranchRetirementReason::Superseded);
    if let worth_proof::TransitionOutcome::Success(plan) = plan {
        let _ = services
            .lifecycle_port()
            .retire_exact(plan, &SignalOwnerCancellationSource::new().token());
    }
}

fn main() {
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the root basis is owner-issued");
    let services: Services = runtime.owner_component_services().expect("issuance");
    invalid_retirement_input(&services, basis.clone());
    valid_retirement_input(&services, basis);
}
