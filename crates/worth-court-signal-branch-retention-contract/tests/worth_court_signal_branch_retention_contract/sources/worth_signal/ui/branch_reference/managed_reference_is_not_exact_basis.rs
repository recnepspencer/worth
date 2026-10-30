use worth_signal::facade::{
    branch::{AdmittedSignalBranchBasis, ManagedSignalBranchReference},
    SignalRuntime,
};

fn valid_exact_advance(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    basis: &AdmittedSignalBranchBasis,
) {
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(runtime.runtime_policy().serial_memory_bytes),
        worth_execution::CancellationToken::new(),
        None,
    );
    let _ = runtime.advance_signal_branch(
        worth_execution::ExecutionRequest::serial(&serial_request),
        &mut (),
        basis,
        |_| Ok(()),
    );
}

fn managed_reference_cannot_replace_exact_basis(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    reference: &ManagedSignalBranchReference,
) {
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(runtime.runtime_policy().serial_memory_bytes),
        worth_execution::CancellationToken::new(),
        None,
    );
    let _ = runtime.advance_signal_branch(
        worth_execution::ExecutionRequest::serial(&serial_request),
        &mut (),
        reference,
        |_| Ok(()),
    );
}

fn main() {}
