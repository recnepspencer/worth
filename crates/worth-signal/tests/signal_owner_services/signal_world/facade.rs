use worth_signal::facade::branch::{
    SignalBranchBasisReadmissionDenial, SignalBranchRetentionReleaseOutcome,
    SignalOwnerCancellationSource, SignalOwnerLifecycleObservation,
};

use super::observation::neutral_basis;
use super::world::CargoRoutingWorld;

fn assert_send_sync_clone<T: Send + Sync + Clone>(_: &T) {}

#[test]
fn concrete_facade_ports_compile_and_cover_the_owner_method_shapes() {
    let world = CargoRoutingWorld::new();
    let bundle = world.services.clone();
    assert_send_sync_clone(&bundle);

    let basis = bundle.basis_port();
    let mutation = bundle.mutation_port();
    let lifecycle = bundle.lifecycle_port();
    assert_send_sync_clone(&basis);
    assert_send_sync_clone(&mutation);
    assert_send_sync_clone(&lifecycle);
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Open
    );

    let reference = basis
        .issue_managed_branch_reference(&world.main_basis)
        .expect("only the concrete owner facade issues managed references");
    let observed = basis
        .observe_current(&reference)
        .expect("the concrete basis facade observes the canonical cell");
    assert_eq!(neutral_basis(&observed), neutral_basis(&world.main_basis));
    basis
        .compare_current_exact(&observed)
        .expect("the concrete basis facade compares complete exact truth");

    let lease = basis
        .retain_exact(&observed)
        .expect("the concrete basis facade retains one exact target");
    match basis.release_exact(lease) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => {
            assert_eq!(receipt.branch_id(), observed.branch_id());
        }
        other => panic!("the live owner must account for an explicit release: {other:?}"),
    }

    let fork = mutation
        .fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("facade-child")
                .expect("the validated identity is issued by the facade vocabulary"),
            &world.main_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the concrete mutation facade issues a real fork");
    let (_, child_basis) = fork.into_parts();
    let captured = mutation
        .capture_exact(&child_basis, &SignalOwnerCancellationSource::new().token())
        .expect("the concrete mutation facade captures a real snapshot");
    let (snapshot, captured_basis) = captured.into_parts();
    let restored = mutation
        .restore_exact(
            &captured_basis,
            &snapshot,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the concrete mutation facade restores the exact snapshot");
    assert_eq!(
        neutral_basis(&restored).generation,
        neutral_basis(&captured_basis).generation + 1
    );

    let foreign = CargoRoutingWorld::new();
    assert!(matches!(
        basis.readmit_exact(&reference, foreign.main_basis.descriptor()),
        Err(SignalBranchBasisReadmissionDenial::OwnerMismatch { .. })
    ));
}
