use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc};
use std::thread;

use crate::branch::SignalBranchBasisObservationDenial;
use crate::state::SignalBranchId;

use super::super::{SignalOwnerLifecycleState, SignalOwnerServiceCounters};
use super::progress_bound::PROGRESS_BOUND;
use super::runtime_root::runtime_with_two_branches;

#[test]
fn basis_observation_preserves_registry_identity_and_retirement_posture() {
    let (mut runtime, branch_a, branch_b, _) = runtime_with_two_branches();
    let (basis, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = basis.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("basis observation admits");

    let observed_a = owner
        .observe_branch_exact(&admission, branch_a.id)
        .expect("healthy branch A observes");
    let observed_b = owner
        .observe_branch_exact(&admission, branch_b.id)
        .expect("healthy branch B observes");
    assert_ne!(observed_a.branch_id(), observed_b.branch_id());

    let unknown = SignalBranchId(99_999);
    assert!(matches!(
        owner.observe_branch_exact(&admission, unknown),
        Err(SignalBranchBasisObservationDenial::UnknownBranch { branch_id })
            if branch_id == unknown
    ));

    let retirement = owner
        .begin_retirement(&admission, branch_a.id)
        .expect("retirement marks canonical registry membership");
    assert!(matches!(
        owner.observe_branch_exact(&admission, branch_a.id),
        Err(SignalBranchBasisObservationDenial::RetirementInProgress { branch_id })
            if branch_id == branch_a.id
    ));
    drop(retirement);
    owner
        .observe_branch_exact(&admission, branch_a.id)
        .expect("abandoned retirement restores healthy observation");
}

#[test]
fn basis_observation_preserves_foreign_and_expired_owner_posture() {
    let (mut runtime, branch_a, _, _) = runtime_with_two_branches();
    let (basis, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = basis.upgrade_owner().expect("owner remains live");
    let runtime_instance_id = owner.runtime_instance_id();
    let foreign_lifecycle = SignalOwnerLifecycleState::new(
        runtime_instance_id + 1,
        Arc::new(SignalOwnerServiceCounters::default()),
    );
    let foreign_admission = foreign_lifecycle
        .admit(runtime_instance_id + 1)
        .expect("foreign owner admits its own operation");
    let next_incarnation = SignalOwnerLifecycleState::new(
        runtime_instance_id,
        Arc::new(SignalOwnerServiceCounters::default()),
    );
    let expired_admission = next_incarnation
        .admit(runtime_instance_id)
        .expect("next lifecycle incarnation admits its own operation");

    assert!(matches!(
        owner.observe_branch_exact(&foreign_admission, branch_a.id),
        Err(SignalBranchBasisObservationDenial::OwnerUnavailable(_))
    ));
    assert!(matches!(
        owner.observe_branch_exact(&expired_admission, branch_a.id),
        Err(SignalBranchBasisObservationDenial::OwnerUnavailable(_))
    ));
}

#[test]
fn basis_observation_distinguishes_cell_misuse_retirement_and_quarantine() {
    let (mut runtime, branch_a, branch_b, _) = runtime_with_two_branches();
    let (basis, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = basis.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("cell observation admits");
    let cell_a = owner
        .lookup_cell(&admission, branch_a.id)
        .expect("branch A cell is live");
    let cell_b = owner
        .lookup_cell(&admission, branch_b.id)
        .expect("branch B cell is live");

    let nested = cell_a
        .with_state(&admission, |_, _| {
            owner.observe_branch_exact(&admission, branch_b.id)
        })
        .expect("outer cell hold is valid");
    assert!(matches!(
        nested,
        Err(SignalBranchBasisObservationDenial::OwnerCellMisuse { branch_id })
            if branch_id == branch_b.id
    ));
    owner
        .observe_branch_exact(&admission, branch_b.id)
        .expect("misuse denial does not damage healthy twin");

    let (observe_tx, observe_rx) = mpsc::sync_channel(1);
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let observing_owner = Arc::clone(&owner);
    let observing_cell = Arc::clone(&cell_a);
    let observation_thread = thread::spawn(move || {
        observe_rx.recv().expect("retirement observation starts");
        let observing_admission = observing_owner
            .admit()
            .expect("independent retirement observation admits");
        result_tx
            .send(observing_cell.observe_exact(&observing_admission))
            .expect("retirement observation returns");
    });
    let retirement = owner
        .begin_retirement(&admission, branch_a.id)
        .expect("branch A retirement begins");
    retirement
        .execute(|_, _| {
            observe_tx
                .send(())
                .expect("release the independent observer while retirement owns the cell");
            let observed = result_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("independent observation must not wait on the retiring cell");
            assert!(matches!(
                observed,
                Err(SignalBranchBasisObservationDenial::RetirementInProgress { branch_id })
                    if branch_id == branch_a.id
            ));
            Ok::<_, ()>(())
        })
        .expect("retirement reaches its cell")
        .expect("retirement completes");
    observation_thread
        .join()
        .expect("independent retirement observer remains healthy");
    assert!(matches!(
        cell_a.observe_exact(&admission),
        Err(SignalBranchBasisObservationDenial::RetiredBranch { branch_id })
            if branch_id == branch_a.id
    ));
    assert!(matches!(
        owner.observe_branch_exact(&admission, branch_a.id),
        Err(SignalBranchBasisObservationDenial::UnknownBranch { branch_id })
            if branch_id == branch_a.id
    ));

    let poison = catch_unwind(AssertUnwindSafe(|| {
        let _ = cell_b.with_state(&admission, |_, _| {
            panic!("inject branch-cell poison after admission")
        });
    }));
    assert!(poison.is_err());
    assert!(matches!(
        owner.observe_branch_exact(&admission, branch_b.id),
        Err(SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id })
            if branch_id == branch_b.id
    ));
}
