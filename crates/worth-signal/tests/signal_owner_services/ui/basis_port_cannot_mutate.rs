use worth_signal::facade::branch::SignalOwnerServicePorts;
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

fn invalid_basis_mutation(
    services: &Services,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) {
    let _ = services.basis_port().advance_exact(
        execution,
        basis,
        &mut (),
        &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        |_| Ok(()),
    );
}

fn valid_mutation(
    services: &Services,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) {
    let _ = services.mutation_port().advance_exact(
        execution,
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
    // This standalone compile contract uses the declared operational memory policy.
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    invalid_basis_mutation(&services, &basis, execution);
    valid_mutation(&services, &basis, execution);
}
