//! Public owner-service workflow with two independently advanced branches.
//!
//! Run it with:
//!
//! ```text
//! cargo run -p worth-signal --example independent_branch_services
//! ```

use std::sync::mpsc;
use std::thread;

use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, SignalBranchBasisPort, SignalBranchMutationPort,
    SignalBranchRetentionReleaseOutcome, SignalOwnerCancellationSource,
    SignalOwnerLifecycleObservation, SignalOwnerServicePorts, SignalOwnerUnavailable,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Runtime = SignalRuntime<(), (), (), (), ()>;
type Services = SignalOwnerServicePorts<(), (), (), (), ()>;
type BasisPort = SignalBranchBasisPort<(), (), ()>;
type MutationPort = SignalBranchMutationPort<(), (), (), (), ()>;

fn main() {
    thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(owner_workflow)
        .expect("the owner workflow thread starts")
        .join()
        .expect("the owner workflow thread completes");
}

fn owner_workflow() {
    let mut runtime = Runtime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let initial = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the runtime issues the initial exact basis");
    let services: Services = runtime
        .owner_component_services()
        .expect("the canonical owner issues the weak service bundle");
    let basis_port: BasisPort = services.basis_port();
    let mutation_port: MutationPort = services.mutation_port();
    let lifecycle = services.lifecycle_port();

    let child_basis = mutation_port
        .fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("independent-child")
                .expect("the child name is valid"),
            &initial,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the owner admits a second branch")
        .into_parts()
        .1;
    let initial_reference = basis_port
        .issue_managed_branch_reference(&initial)
        .expect("the first branch receives a managed reference");
    let child_reference = basis_port
        .issue_managed_branch_reference(&child_basis)
        .expect("the second branch receives a managed reference");

    let (left, right) = advance_both_without_runtime_borrow(
        mutation_port.clone(),
        initial.clone(),
        child_basis.clone(),
    );
    assert!(basis_port.observe_current(&initial_reference).is_ok());
    assert!(basis_port.observe_current(&child_reference).is_ok());

    let lease = basis_port
        .retain_exact(&left)
        .expect("an exact advanced basis can carry a retention obligation");
    match basis_port.release_exact(lease) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => {
            assert_eq!(receipt.branch_id(), left.branch_id());
        }
        other => panic!("the live owner must account for the exact release: {other:?}"),
    }

    drop(left);
    drop(right);
    drop(child_basis);
    drop(initial);
    drop(initial_reference);
    drop(child_reference);
    drop(runtime);

    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    let unavailable: Result<_, SignalOwnerUnavailable> = basis_port.owner_service_cost_snapshot();
    assert!(
        unavailable.is_err(),
        "weak ports report typed owner loss after the strong root closes"
    );
    println!("independent_branch_services: public owner workflow held.");
}

fn advance_both_without_runtime_borrow(
    mutation: MutationPort,
    left_basis: AdmittedSignalBranchBasis,
    right_basis: AdmittedSignalBranchBasis,
) -> (AdmittedSignalBranchBasis, AdmittedSignalBranchBasis) {
    let (left_tx, left_rx) = mpsc::sync_channel(1);
    let (right_tx, right_rx) = mpsc::sync_channel(1);
    thread::scope(|scope| {
        let left_mutation = mutation.clone();
        scope.spawn(move || {
            let result = left_mutation
                .advance_exact(
                    &left_basis,
                    &mut (),
                    &SignalOwnerCancellationSource::new().token(),
                    |_| Ok(()),
                )
                .map(|outcome| outcome.into_basis())
                .map_err(|denial| format!("{denial:?}"));
            let _ = left_tx.send(result);
        });

        scope.spawn(move || {
            let result = mutation
                .advance_exact(
                    &right_basis,
                    &mut (),
                    &SignalOwnerCancellationSource::new().token(),
                    |_| Ok(()),
                )
                .map(|outcome| outcome.into_basis())
                .map_err(|denial| format!("{denial:?}"));
            let _ = right_tx.send(result);
        });

        let left = left_rx
            .recv_timeout(std::time::Duration::from_secs(3))
            .expect("the first branch worker completes")
            .unwrap_or_else(|denial| panic!("the first branch advance was denied: {denial}"));
        let right = right_rx
            .recv_timeout(std::time::Duration::from_secs(3))
            .expect("the second branch worker completes")
            .unwrap_or_else(|denial| panic!("the second branch advance was denied: {denial}"));
        (left, right)
    })
}
