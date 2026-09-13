use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    validate_signal_branch_name, SignalBranchRetirementReason, SignalOwnerCancellationSource,
    SignalOwnerLifecycleObservation, SignalOwnerServicePorts,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Runtime = SignalRuntime<(), (), (), (), ()>;
type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

fn assert_send_sync_clone<T: Send + Sync + Clone>(_: &T) {}

fn main() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(compile_public_port_matrix)
        .expect("the compiler-pass fixture thread starts")
        .join()
        .expect("the compiler-pass fixture completes");
}

fn compile_public_port_matrix() {
    let mut runtime = Runtime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the root basis is owner-issued");
    let services: Services = runtime
        .owner_component_services()
        .expect("the concrete public bundle is issued");
    assert_send_sync_clone(&services);

    let basis_port = services.basis_port();
    let mutation_port = services.mutation_port();
    let lifecycle_port = services.lifecycle_port();
    assert_send_sync_clone(&basis_port);
    assert_send_sync_clone(&mutation_port);
    assert_send_sync_clone(&lifecycle_port);

    let reference = basis_port
        .issue_managed_branch_reference(&basis)
        .expect("managed references come from exact bases");
    let observed = basis_port
        .observe_current(&reference)
        .expect("the reference observes its canonical cell");
    let _ = basis_port.readmit_exact(&reference, observed.descriptor());
    let _ = basis_port.compare_current_exact(&observed);
    let lease = basis_port
        .retain_exact(&observed)
        .expect("retention is public");
    let _ = basis_port.release_exact(lease);
    assert_eq!(
        lifecycle_port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Open
    );
    let _ = basis_port.owner_service_cost_snapshot();

    let child = mutation_port
        .fork_exact(
            validate_signal_branch_name("compile-child").expect("the identity is valid"),
            &basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("fork is public")
        .into_parts();
    let captured = mutation_port
        .capture_exact(&child.1, &SignalOwnerCancellationSource::new().token())
        .expect("capture is public")
        .into_parts();
    let _restored = mutation_port
        .restore_exact(
            &captured.1,
            &captured.0,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("restore is public");

    let retirement =
        lifecycle_port.plan_retirement_exact(child.1, SignalBranchRetirementReason::Superseded);
    if let TransitionOutcome::Success(plan) = retirement {
        let _ = lifecycle_port.retire_exact(plan, &SignalOwnerCancellationSource::new().token());
    }
}
