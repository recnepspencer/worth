use worth_signal::facade::branch::{SignalOwnerCancellationSource, SignalOwnerServicePorts};
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

fn invalid_consumed_outcome(
    services: &Services,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) {
    let outcome = services
        .mutation_port()
        .advance_exact(
            execution,
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
    execution: worth_execution::ExecutionRequest<'_, '_>,
) {
    let outcome = services
        .mutation_port()
        .advance_exact(
            execution,
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
    // This standalone compile contract uses the declared operational memory policy.
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    invalid_consumed_outcome(&services, &basis, execution);
    valid_borrow_before_consume(&services, &basis, execution);
}
