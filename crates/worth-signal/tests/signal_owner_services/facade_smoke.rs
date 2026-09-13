use worth_signal::facade::branch::{
    validate_signal_branch_name, SignalBranchBasisPort, SignalBranchLifecyclePort,
    SignalBranchMutationPort, SignalOwnerCancellationSource, SignalOwnerLifecycleObservation,
    SignalOwnerServiceIssuanceDenial, SignalOwnerServicePorts, SignalOwnerUnavailable,
    ValidatedSignalBranchName,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

type Runtime = SignalRuntime<(), (), (), (), ()>;
type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

fn runtime() -> Runtime {
    SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

fn assert_send_sync_clone<T: Send + Sync + Clone>(_: &T) {}

#[test]
fn public_facade_issues_weak_ports_over_the_canonical_owner() {
    let mut runtime = runtime();
    let initial = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the owner root observes its bootstrap branch before cutover");

    let issued: Result<Services, SignalOwnerServiceIssuanceDenial> =
        runtime.owner_component_services();
    let services: Services = issued.expect("the public runtime issues its concrete service bundle");
    assert_send_sync_clone(&services);

    let basis: SignalBranchBasisPort<(), (), ()> = services.basis_port();
    let mutation: SignalBranchMutationPort<(), (), (), (), ()> = services.mutation_port();
    let lifecycle: SignalBranchLifecyclePort<(), (), ()> = services.lifecycle_port();
    assert_send_sync_clone(&basis);
    assert_send_sync_clone(&mutation);
    assert_send_sync_clone(&lifecycle);

    let reference = basis
        .issue_managed_branch_reference(&initial)
        .expect("the facade port issues a managed reference from real authority");
    let observed = basis
        .observe_current(&reference)
        .expect("the facade port reaches the canonical installed cell");
    assert_eq!(observed.observation(), initial.observation());

    let requested_identity: ValidatedSignalBranchName =
        validate_signal_branch_name("public-facade-child").expect("the owner name validates");
    let cancellation = SignalOwnerCancellationSource::new();
    let (created_branch, created_basis) = mutation
        .fork_exact(requested_identity, &initial, &cancellation.token())
        .expect("the public mutation port reaches the canonical fork engine")
        .into_parts();
    let created_reference = basis
        .issue_managed_branch_reference(&created_basis)
        .expect("the created basis carries owner authority");
    let created_observation = basis
        .observe_current(&created_reference)
        .expect("the created branch is installed in the canonical registry");
    assert_eq!(created_observation.branch_id(), created_branch.id);
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Open
    );

    drop(observed);
    drop(created_observation);
    drop(created_basis);
    drop(created_reference);
    drop(initial);
    drop(reference);
    drop(runtime);

    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    let unavailable: Result<_, SignalOwnerUnavailable> = basis.owner_service_cost_snapshot();
    assert!(unavailable.is_err());
}

#[cfg(feature = "test-operation-control")]
#[test]
fn public_facade_issues_drop_safe_control_for_real_owner_boundaries() {
    use worth_signal::facade::branch::{
        SignalOwnerOperationBoundary, SignalOwnerOperationControl, SignalOwnerOperationPause,
    };

    let mut runtime = runtime();
    runtime
        .owner_component_services()
        .expect("owner service issuance seals the canonical owner");
    let control: SignalOwnerOperationControl = runtime
        .owner_operation_control()
        .expect("the sealed owner issues feature-gated deterministic control");
    let pause: SignalOwnerOperationPause =
        control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    pause.release();
}
