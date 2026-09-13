use std::rc::Rc;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

use crate::branch::{validate_signal_branch_name, AdmittedSignalBranchBasis};
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;
use crate::state::SignalBranchHandle;

use super::super::{
    SignalBranchBasisPort, SignalBranchCellAdmissionDenial, SignalBranchLifecyclePort,
    SignalBranchMutationPort, SignalBranchRegistryDenial, SignalOwnerCancellationSource,
};
use super::progress_bound::{worker_park, PROGRESS_BOUND};
use super::with_movement_permit;

type TestRuntime = SignalRuntime<(), (), (), (), ()>;

pub(in crate::branch::owner_services) fn runtime_with_two_branches() -> (
    TestRuntime,
    SignalBranchHandle,
    SignalBranchHandle,
    AdmittedSignalBranchBasis,
) {
    runtime_with_two_branches_from_graph(SignalGraph::new())
}

pub(in crate::branch::owner_services) fn runtime_with_two_branches_from_graph(
    graph: SignalGraph,
) -> (
    TestRuntime,
    SignalBranchHandle,
    SignalBranchHandle,
    AdmittedSignalBranchBasis,
) {
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let branch_a = runtime.current_branch();
    let source = runtime
        .observe_signal_branch_basis(branch_a.clone())
        .expect("the real runtime admits its initial branch");
    let branch_b = runtime
        .fork_signal_branch("phase-3-independent-b", &source)
        .expect("the real runtime forks branch B before sealing")
        .created_branch()
        .clone();
    let branch_b_basis = runtime
        .observe_signal_branch_basis(branch_b.clone())
        .expect("the real runtime admits branch B before sealing");
    (runtime, branch_a, branch_b, branch_b_basis)
}

#[test]
fn real_runtime_cells_make_independent_progress_while_one_target_is_parked() {
    let (mut runtime, branch_a, branch_b, branch_b_basis) = runtime_with_two_branches();
    let (basis_port, _, _) = runtime
        .owner_port_slots()
        .expect("the callback-free runtime seals into real owner cells");
    let owner = basis_port
        .upgrade_owner()
        .expect("the root owns sealed state");
    let setup = owner.admit().expect("the sealed owner admits setup lookup");
    let cell_a = owner
        .lookup_cell(&setup, branch_a.id)
        .expect("branch A owns its canonical state cell");
    let cell_b = owner
        .lookup_cell(&setup, branch_b.id)
        .expect("branch B owns its canonical state cell");
    drop(setup);

    let (a_park, mut a_control) = worker_park();
    let (a_done_tx, a_done_rx) = mpsc::sync_channel(1);
    let a_owner = owner.clone();
    let a_cell = cell_a.clone();
    let branch_a_id = branch_a.id;
    thread::spawn(move || {
        let admission = a_owner.admit().expect("branch A operation is admitted");
        let result = a_cell.with_state(&admission, |state, work| {
            assert_eq!(state.branch_id(), branch_a_id);
            assert_eq!(state.state().graph().current_branch().id, branch_a_id);
            a_park.park("real branch A canonical-state cell");
            state.state_mut().clear_branch_mutation_nodes();
            with_movement_permit(|permit| work.record_canonical_movement(permit));
            state.state().graph().current_branch().id
        });
        let _ = a_done_tx.send(result);
    });
    a_control.wait_until_parked("real branch A canonical-state cell");
    let a_before_b = cell_a.cost_snapshot();
    let owner_before_b = owner.cost_snapshot();

    let (b_done_tx, b_done_rx) = mpsc::sync_channel(1);
    let b_owner = owner.clone();
    let branch_b_id = branch_b.id;
    thread::spawn(move || {
        let admission = b_owner.admit().expect("branch B operation is admitted");
        let looked_up = b_owner
            .lookup_cell(&admission, branch_b_id)
            .expect("B registry lookup completes while A is parked");
        let cancellation = SignalOwnerCancellationSource::new();
        let mut runtime_ctx = ();
        let result = looked_up
            .advance_exact::<(), (), _>(
                &admission,
                &branch_b_basis,
                &mut runtime_ctx,
                &cancellation.token(),
                |_| Ok(()),
            )
            .map(|outcome| {
                let (observation, transaction) = outcome.into_parts();
                assert_eq!(observation.generation().get(), 1);
                assert_eq!(transaction.touched_nodes, 0);
                branch_b_id
            });
        let _ = b_done_tx.send(result);
    });
    let b_result = b_done_rx.recv_timeout(PROGRESS_BOUND);
    let a_after_b = cell_a.cost_snapshot();
    let b_after = cell_b.cost_snapshot();
    let owner_after_b = owner.cost_snapshot();
    let a_still_parked = a_done_rx.try_recv() == Err(TryRecvError::Empty);
    a_control.release();
    let a_result = a_done_rx.recv_timeout(PROGRESS_BOUND);

    assert!(
        matches!(b_result, Ok(Ok(observed)) if observed == branch_b_id),
        "B finishes before A release"
    );
    assert!(a_still_parked, "B cannot release A");
    assert_eq!(a_result, Ok(Ok(branch_a_id)));
    assert_eq!(a_after_b, a_before_b, "B causes zero A-cell work");
    assert_eq!(
        (b_after.contacts(), b_after.waits(), b_after.movements()),
        (1, 0, 1)
    );
    assert_eq!(
        owner_after_b.branch_registry_lookups(),
        owner_before_b.branch_registry_lookups() + 1
    );
    assert_eq!(
        owner_after_b.target_cell_contacts(),
        owner_before_b.target_cell_contacts() + 1
    );
    assert_eq!(
        owner_after_b.target_cell_waits(),
        owner_before_b.target_cell_waits()
    );
    assert_eq!(
        owner_after_b.canonical_movements(),
        owner_before_b.canonical_movements() + 1
    );
    assert_eq!(owner_after_b.branch_registry_entries_scanned(), 0);
}

#[test]
fn sealing_moves_complete_membership_and_retirement_serializes_real_state() {
    let (mut runtime, branch_a, branch_b, _branch_b_basis) = runtime_with_two_branches();
    let observed = runtime
        .observe_signal_branch_basis(branch_a.clone())
        .expect("the pre-seal owner observation is exact");
    let expected_definition_basis = observed
        .observation()
        .target()
        .as_basis()
        .expect("a branch observation carries a basis target")
        .definition_basis();
    let (basis_port, _, _) = runtime.owner_port_slots().expect("sealing succeeds");
    let owner = basis_port.upgrade_owner().expect("sealed owner upgrades");
    assert_eq!(owner.definition_basis(), expected_definition_basis);
    assert_eq!(owner.admitted_retention_count(branch_a.id), 1);
    let fork_admission = owner.admit().expect("fork reservation is admitted");
    let original_children = owner
        .metadata
        .branch_children(&fork_admission, branch_a.id)
        .expect("existing lineage is owner-admitted");
    let fork_reservation = owner
        .reserve_fork_destination(
            &fork_admission,
            &observed,
            validate_signal_branch_name("phase-3-reserved-destination")
                .expect("the requested identity is validated"),
        )
        .expect("owner reserves destination before source capture");
    assert_eq!(
        fork_reservation.branch().parent_branch_id,
        Some(branch_a.id)
    );
    assert_ne!(fork_reservation.branch().id, branch_a.id);
    assert_ne!(fork_reservation.branch().id, branch_b.id);
    let reserved_branch_id = fork_reservation.branch().id;
    let mut expected_children = original_children.clone();
    expected_children.push(reserved_branch_id);
    expected_children.sort_unstable();
    assert_eq!(
        owner
            .metadata
            .branch_children(&fork_admission, branch_a.id)
            .expect("fork lineage is owner-admitted"),
        expected_children
    );
    assert_eq!(owner.live_count(), 2);
    assert_eq!(owner.reservation_count(), 1);
    drop(fork_reservation);
    assert_eq!(
        owner
            .metadata
            .branch_children(&fork_admission, branch_a.id)
            .expect("dropped fork lineage is observable"),
        original_children
    );
    assert_eq!(owner.reservation_count(), 0);
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let reservation = owner
            .reserve_fork_destination(
                &fork_admission,
                &observed,
                validate_signal_branch_name("phase-3-unwind-destination")
                    .expect("unwind identity is validated"),
            )
            .expect("unwind reservation is admitted");
        let mut expected = original_children.clone();
        expected.push(reservation.branch().id);
        expected.sort_unstable();
        assert_eq!(
            owner
                .metadata
                .branch_children(&fork_admission, branch_a.id)
                .expect("unwind lineage is visible"),
            expected
        );
        panic!("exercise fork reservation unwind cleanup");
    }));
    assert!(unwind.is_err());
    assert_eq!(owner.reservation_count(), 0);
    assert_eq!(
        owner
            .metadata
            .branch_children(&fork_admission, branch_a.id)
            .expect("unwind lineage cleanup is observable"),
        original_children
    );
    drop(fork_admission);
    drop(observed);
    assert_eq!(owner.admitted_retention_count(branch_a.id), 0);
    assert_eq!(owner.live_count(), 2);
    let admission = owner.admit().expect("sealed membership is admitted");
    let first = owner
        .lookup_cell(&admission, branch_b.id)
        .expect("B is live");
    let second = owner
        .lookup_cell(&admission, branch_b.id)
        .expect("B is stable");
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    first
        .with_state(&admission, |state, _| {
            assert_eq!(state.head_generation(), 0);
            assert_eq!(state.restore_snapshot_id(), None);
        })
        .expect("B head truth lives in its canonical cell");
    let branch_a_cell = owner
        .lookup_cell(&admission, branch_a.id)
        .expect("A is live");
    let branch_b_before = first.cost_snapshot();
    branch_a_cell
        .with_state(&admission, |_, _| {
            assert!(matches!(
                first.with_state(&admission, |_, _| ()),
                Err(SignalBranchCellAdmissionDenial::SecondCellWhileHeld)
            ));
            assert!(matches!(
                owner.metadata.branch_children(&admission, branch_a.id),
                Err(super::super::owner_metadata::SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse)
            ));
        })
        .expect("one admitted A-cell operation is valid");
    assert_eq!(first.cost_snapshot(), branch_b_before);
    assert_eq!(
        owner
            .metadata
            .branch_children(&admission, branch_a.id)
            .expect("metadata access succeeds after the cell hold releases"),
        original_children,
        "the denied in-cell observation cannot mutate transferred lineage metadata"
    );
    let retirement = owner
        .begin_retirement(&admission, branch_b.id)
        .expect("B retirement reserves its one cell");
    let retired = retirement
        .execute(|state, work| {
            assert_eq!(state.handle(), &branch_b);
            with_movement_permit(|permit| work.record_canonical_movement(permit));
            Ok::<_, ()>(state.state().graph().current_branch().id)
        })
        .expect("retirement cell admission succeeds")
        .expect("retirement operation performs");
    assert_eq!(retired, branch_b.id);
    assert_eq!(owner.live_count(), 1);
    assert!(owner.lookup_cell(&admission, branch_a.id).is_ok());
    assert!(matches!(
        owner.lookup_cell(&admission, branch_b.id),
        Err(SignalBranchRegistryDenial::UnknownBranch(id)) if id == branch_b.id
    ));
}

#[test]
fn two_runtime_roots_preserve_owner_affinity() {
    let (mut runtime_a, branch_a, _, _) = runtime_with_two_branches();
    let (mut runtime_b, branch_b, _, _) = runtime_with_two_branches();
    let (port_a, _, _) = runtime_a.owner_port_slots().expect("A seals");
    let (port_b, _, _) = runtime_b.owner_port_slots().expect("B seals");
    let owner_a = port_a.upgrade_owner().expect("A upgrades");
    let owner_b = port_b.upgrade_owner().expect("B upgrades");
    let admission_a = owner_a.admit().expect("A admits");
    assert!(owner_a.lookup_cell(&admission_a, branch_a.id).is_ok());
    assert!(matches!(
        owner_b.lookup_cell(&admission_a, branch_b.id),
        Err(SignalBranchRegistryDenial::ForeignOwner)
    ));
}

#[test]
fn weak_port_slots_do_not_keep_the_runtime_owner_alive() {
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let (basis, mutation, lifecycle) = runtime.owner_port_slots().expect("runtime seals");
    let runtime_id = basis
        .upgrade_owner()
        .expect("rooted port upgrades")
        .runtime_instance_id();
    assert_eq!(basis.diagnostic_owner_runtime_instance_id(), runtime_id);
    assert_eq!(mutation.diagnostic_owner_runtime_instance_id(), runtime_id);
    assert_eq!(lifecycle.diagnostic_owner_runtime_instance_id(), runtime_id);
    assert_eq!(
        lifecycle
            .upgrade_owner()
            .expect("lifecycle owner upgrades")
            .lifecycle_observation(),
        super::super::SignalOwnerLifecycleObservation::Open
    );
    assert_eq!(
        mutation
            .upgrade_owner()
            .expect("same owner")
            .runtime_instance_id(),
        runtime_id
    );
    drop(runtime);
    assert!(basis.upgrade_owner().is_err());
    assert!(mutation.upgrade_owner().is_err());
    assert!(lifecycle.upgrade_owner().is_err());
}

macro_rules! assert_not_impl {
    ($ty:ty: $bound:path) => {
        const _: fn() = || {
            trait AmbiguousIfImpl<A> {
                fn probe() {}
            }
            impl<T: ?Sized> AmbiguousIfImpl<()> for T {}
            struct IfImpl;
            impl<T: ?Sized + $bound> AmbiguousIfImpl<IfImpl> for T {}
            let _ = <$ty as AmbiguousIfImpl<_>>::probe;
        };
    };
}

#[test]
fn port_auto_traits_do_not_substitute_for_runtime_issuance_eligibility() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SignalBranchBasisPort<(), (), ()>>();
    assert_send_sync::<SignalBranchMutationPort<(), (), Rc<()>, Rc<()>, ()>>();
    assert_send_sync::<SignalBranchLifecyclePort<(), (), ()>>();
    assert_not_impl!(SignalBranchBasisPort<*const (), (), ()>: Send);
    assert_not_impl!(SignalBranchMutationPort<(), *const (), (), (), ()>: Sync);
    assert_not_impl!(SignalBranchLifecyclePort<(), (), *const ()>: Send);

    let source = SignalOwnerCancellationSource::new();
    let token = source.token();
    assert!(token.preflight_movement().is_ok());
    source.cancel();
    assert!(token.preflight_movement().is_err());
}
