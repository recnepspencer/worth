#[allow(unused_imports)]
use crate::signal_owner_services::ordinary_context::signal_world::facade::*;
pub(crate) use {
    worth_signal::facade::branch::SignalBranchBasisReadmissionDenial,
    worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
    worth_signal::facade::branch::SignalOwnerLifecycleObservation,
};

pub(crate) use crate::signal_owner_services::ordinary_context::signal_world::observation::neutral_basis;
pub(crate) use crate::signal_owner_services::ordinary_context::signal_world::world::CargoRoutingWorld;

#[test]
fn concrete_facade_ports_compile_and_cover_the_owner_method_shapes() {
    let world =
        crate::signal_owner_services::ordinary_context::signal_world::world::CargoRoutingWorld::new(
        );
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
        worth_signal::facade::branch::SignalOwnerLifecycleObservation::Open
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
        worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Released(receipt) => {
            assert_eq!(receipt.branch_id(), observed.branch_id());
        }
        other => panic!("the live owner must account for an explicit release: {other:?}"),
    }

    let fork = mutation
        .fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("facade-child")
                .expect("the validated identity is issued by the facade vocabulary"),
            &world.main_basis,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        )
        .expect("the concrete mutation facade issues a real fork");
    let (_, child_basis) = fork.into_parts();
    let captured = mutation
        .capture_exact(
            &child_basis,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        )
        .expect("the concrete mutation facade captures a real snapshot");
    let (snapshot, captured_basis) = captured.into_parts();
    let restored = mutation
        .restore_exact(
            &captured_basis,
            &snapshot,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        )
        .expect("the concrete mutation facade restores the exact snapshot");
    assert_eq!(
        neutral_basis(&restored).generation,
        neutral_basis(&captured_basis).generation + 1
    );

    let foreign =
        crate::signal_owner_services::ordinary_context::signal_world::world::CargoRoutingWorld::new(
        );
    assert!(matches!(
        basis.readmit_exact(&reference, foreign.main_basis.descriptor()),
        Err(worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::OwnerMismatch { .. })
    ));
}
