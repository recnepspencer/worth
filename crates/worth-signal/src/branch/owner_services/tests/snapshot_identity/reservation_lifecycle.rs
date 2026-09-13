use crate::branch::{
    admit_runtime_signal_branch_observation, SignalBranchMergeDenial,
    SignalBranchSnapshotCaptureDenial,
};
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;
use crate::state::SignalSnapshotId;

use super::super::super::SignalOwnerCancellationSource;

#[test]
fn cancellation_stale_denial_and_unwind_return_capacity_without_reusing_identity() {
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::builder(SignalGraph::new())
        .with_kernel_defaults()
        .maximum_stored_branch_snapshots(1)
        .build();
    let branch = runtime.current_branch();
    let starting_basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("the real runtime admits its starting branch");
    let (_, mutation, _) = runtime.owner_port_slots().expect("the runtime seals");
    let owner = mutation.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("snapshot work admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("the target cell is installed");

    let cancellation = SignalOwnerCancellationSource::new();
    cancellation.cancel();
    let cancelled_reservation = owner
        .metadata
        .reserve_snapshot(&admission, &cell)
        .expect("capacity reserves before cancellation");
    let cancelled_id = cancelled_reservation.snapshot_id();
    assert!(matches!(
        cell.capture_snapshot_exact(
            &starting_basis,
            cancelled_reservation,
            &cancellation.token(),
        ),
        Err(SignalBranchSnapshotCaptureDenial::CancelledNoMovement)
    ));

    let mut unwind_id = None;
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let reservation = owner
            .metadata
            .reserve_snapshot(&admission, &cell)
            .expect("cancelled capture returned its capacity");
        unwind_id = Some(reservation.snapshot_id());
        panic!("snapshot identity reservation unwind");
    }));
    assert!(unwind.is_err());

    let active_cancellation = SignalOwnerCancellationSource::new();
    let advanced = cell
        .advance_exact::<(), (), _>(
            &admission,
            &starting_basis,
            &mut (),
            &active_cancellation.token(),
            |_| Ok(()),
        )
        .expect("the cell moves so the starting basis becomes stale");
    let (advanced_observation, _) = advanced.into_parts();
    let advanced_basis = admit_runtime_signal_branch_observation(
        advanced_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("the advanced basis retains its branch"),
    );
    let stale_reservation = owner
        .metadata
        .reserve_snapshot(&admission, &cell)
        .expect("unwind returned its capacity");
    let stale_id = stale_reservation.snapshot_id();
    assert!(matches!(
        cell.capture_snapshot_exact(
            &starting_basis,
            stale_reservation,
            &active_cancellation.token(),
        ),
        Err(SignalBranchSnapshotCaptureDenial::BasisMismatch { .. })
    ));

    let healthy_reservation = owner
        .metadata
        .reserve_snapshot(&admission, &cell)
        .expect("stale denial returned its capacity");
    let healthy_id = healthy_reservation.snapshot_id();
    let capture = cell
        .capture_snapshot_exact(
            &advanced_basis,
            healthy_reservation,
            &active_cancellation.token(),
        )
        .expect("the healthy twin installs through the real cell");

    assert_eq!(
        (
            cancelled_id,
            unwind_id.expect("the unwind observed its reservation"),
            stale_id,
            healthy_id,
            capture.snapshot().meta.snapshot_id,
        ),
        (
            SignalSnapshotId(0),
            SignalSnapshotId(1),
            SignalSnapshotId(2),
            SignalSnapshotId(3),
            SignalSnapshotId(3),
        ),
        "every terminal reservation returns capacity while burned identities stay burned"
    );
}

#[test]
fn identity_exhaustion_is_precise_repeatable_and_pre_effect() {
    let mut graph = SignalGraph::new();
    graph
        .diagnostics_state_mut()
        .synchronize_branch_snapshot_allocator(u64::MAX, 1);
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::builder(graph)
        .with_kernel_defaults()
        .maximum_stored_branch_snapshots(1)
        .build();
    let branch = runtime.current_branch();
    let basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("the exhausted runtime admits its starting branch");
    let (_, mutation, _) = runtime.owner_port_slots().expect("the runtime seals");
    let owner = mutation.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("snapshot work admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("the target cell is installed");
    let before = cell.cost_snapshot();

    for _ in 0..2 {
        assert!(matches!(
            owner.metadata.reserve_snapshot(&admission, &cell),
            Err(
                SignalBranchSnapshotCaptureDenial::SnapshotIdentityExhausted {
                    next_snapshot_id: SignalSnapshotId(u64::MAX),
                }
            )
        ));
    }
    assert_eq!(
        cell.cost_snapshot(),
        before,
        "identity exhaustion denies before target contact or movement"
    );
    let advanced = cell
        .advance_exact::<(), (), _>(
            &admission,
            &basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("repeated exhaustion leaves the capacity-one owner healthy");
    let (observation, _) = advanced.into_parts();
    assert_eq!(observation.generation().get(), 1);
}

#[test]
fn unsealed_capture_reports_the_same_precise_identity_exhaustion_before_movement() {
    let mut graph = SignalGraph::new();
    graph
        .diagnostics_state_mut()
        .synchronize_branch_snapshot_allocator(u64::MAX, 1);
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    let branch = runtime.current_branch();
    let basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("the unsealed runtime admits its starting branch");
    let before = basis.observation().clone();

    let denial = runtime
        .capture_signal_branch_snapshot(&basis)
        .expect_err("identity exhaustion must deny unsealed capture");
    assert!(
        matches!(
            denial,
            SignalBranchSnapshotCaptureDenial::SnapshotIdentityExhausted {
                next_snapshot_id: SignalSnapshotId(u64::MAX),
            }
        ),
        "unsealed exhaustion must remain precise, got {denial:?}"
    );
    let after = runtime
        .observe_signal_branch_basis(branch)
        .expect("identity exhaustion leaves the branch observable");
    assert!(
        after.observation().compare(&before).is_ok(),
        "unsealed identity exhaustion cannot move the exact branch reference"
    );
}

#[test]
fn unsealed_merge_reports_identity_exhaustion_before_target_movement() {
    let mut graph = SignalGraph::new();
    graph
        .diagnostics_state_mut()
        .synchronize_branch_snapshot_allocator(u64::MAX, 1);
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    let initial = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the exhausted merge world admits its root");
    let (source, source_basis) = runtime
        .fork_signal_branch("exhausted-merge-source", &initial)
        .expect("the source sibling forks")
        .into_parts();
    let (target, target_basis) = runtime
        .fork_signal_branch("exhausted-merge-target", &initial)
        .expect("the target sibling forks")
        .into_parts();
    let before = target_basis.observation().clone();

    assert!(matches!(
        runtime.merge_branch(&source_basis, &target_basis),
        Err(SignalBranchMergeDenial::SnapshotIdentityExhausted {
            next_snapshot_id: SignalSnapshotId(u64::MAX),
        })
    ));
    let source_after = runtime
        .observe_signal_branch_basis(source)
        .expect("the denied merge leaves the source observable");
    assert!(source_after
        .observation()
        .compare(source_basis.observation())
        .is_ok());
    let target_after = runtime
        .observe_signal_branch_basis(target)
        .expect("the denied merge leaves the target observable");
    assert!(target_after.observation().compare(&before).is_ok());
}
