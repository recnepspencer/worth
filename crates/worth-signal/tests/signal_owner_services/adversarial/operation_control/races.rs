use super::super::world::{AdversarialWorld, PROGRESS_BOUND};
use std::sync::{mpsc, Arc, Barrier};
use std::thread;
use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchAdvanceDenial,
    SignalBranchRestoreDenial, SignalBranchRetirementDenial, SignalBranchRetirementReason,
    SignalBranchSnapshotCaptureDenial, SignalOwnerCancellationSource,
};

#[path = "races/retirement_fences.rs"]
mod retirement_fences;

fn assert_one_success(
    left: Result<(), &'static str>,
    right: Result<(), &'static str>,
    label: &str,
) {
    assert_eq!(
        [left.is_ok(), right.is_ok()]
            .into_iter()
            .filter(|succeeded| *succeeded)
            .count(),
        1,
        "{label} must have exactly one canonical winner: left={left:?} right={right:?}"
    );
}
fn advance_denial(denial: SignalBranchAdvanceDenial) -> &'static str {
    match denial {
        SignalBranchAdvanceDenial::BasisMismatch { .. } => "BasisMismatch",
        SignalBranchAdvanceDenial::RetirementInProgress { .. } => "RetirementInProgress",
        SignalBranchAdvanceDenial::RetiredBranch { .. } => "RetiredBranch",
        other => panic!("unexpected advance denial: {other:?}"),
    }
}
fn restore_denial(denial: SignalBranchRestoreDenial) -> &'static str {
    match denial {
        SignalBranchRestoreDenial::BasisMismatch { .. } => "BasisMismatch",
        SignalBranchRestoreDenial::RetirementInProgress { .. } => "RetirementInProgress",
        SignalBranchRestoreDenial::RetiredBranch { .. } => "RetiredBranch",
        other => panic!("unexpected restore denial: {other:?}"),
    }
}
fn capture_denial(denial: SignalBranchSnapshotCaptureDenial) -> &'static str {
    match denial {
        SignalBranchSnapshotCaptureDenial::BasisMismatch { .. } => "BasisMismatch",
        SignalBranchSnapshotCaptureDenial::RetirementInProgress { .. } => "RetirementInProgress",
        SignalBranchSnapshotCaptureDenial::RetiredBranch { .. } => "RetiredBranch",
        other => panic!("unexpected snapshot denial: {other:?}"),
    }
}
fn retirement_denial(denial: SignalBranchRetirementDenial) -> &'static str {
    match denial {
        SignalBranchRetirementDenial::StaleBranchHead { .. } => "StaleBranchHead",
        SignalBranchRetirementDenial::RetirementInProgress { .. } => "RetirementInProgress",
        SignalBranchRetirementDenial::RetiredBranch { .. } => "RetiredBranch",
        SignalBranchRetirementDenial::SharedAdmittedBasis { .. } => "SharedAdmittedBasis",
        other => panic!("unexpected retirement denial: {other:?}"),
    }
}
pub(super) fn advance(
    mutation: &super::super::world::MutationPort,
    basis: &AdmittedSignalBranchBasis,
) -> Result<(), &'static str> {
    mutation
        .advance_exact(
            basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .map(|_| ())
        .map_err(advance_denial)
}
pub(super) fn capture(
    mutation: &super::super::world::MutationPort,
    basis: &AdmittedSignalBranchBasis,
) -> Result<(AdmittedSignalBranchSnapshot, AdmittedSignalBranchBasis), &'static str> {
    mutation
        .capture_exact(basis, &SignalOwnerCancellationSource::new().token())
        .map(|outcome| outcome.into_parts())
        .map_err(capture_denial)
}
pub(super) fn restore(
    mutation: &super::super::world::MutationPort,
    basis: &AdmittedSignalBranchBasis,
    snapshot: &AdmittedSignalBranchSnapshot,
) -> Result<(), &'static str> {
    mutation
        .restore_exact(
            basis,
            snapshot,
            &SignalOwnerCancellationSource::new().token(),
        )
        .map(|_| ())
        .map_err(restore_denial)
}
pub(super) fn retirement_plan(
    lifecycle: &super::super::world::LifecyclePort,
    basis: AdmittedSignalBranchBasis,
) -> Result<worth_signal::facade::branch::PlannedSignalBranchRetirement, &'static str> {
    match lifecycle.plan_retirement_exact(basis, SignalBranchRetirementReason::Superseded) {
        TransitionOutcome::Success(plan) => Ok(plan),
        TransitionOutcome::Denied(denial) => Err(retirement_denial(denial)),
    }
}

pub(super) fn retire(
    lifecycle: &super::super::world::LifecyclePort,
    plan: worth_signal::facade::branch::PlannedSignalBranchRetirement,
) -> Result<(), &'static str> {
    match lifecycle.retire_exact(plan, &SignalOwnerCancellationSource::new().token()) {
        TransitionOutcome::Success(_) => Ok(()),
        TransitionOutcome::Denied(denial) => Err(retirement_denial(denial)),
    }
}
fn run_pair<L, R>(runtime: &Option<super::super::world::Runtime>, left: L, right: R, label: &str)
where
    L: FnOnce() -> Result<(), &'static str> + Send,
    R: FnOnce() -> Result<(), &'static str> + Send,
{
    let owner_root = runtime
        .as_ref()
        .expect("the race contenders share a live owner root");
    let gate = Arc::new(Barrier::new(3));
    let (left_tx, left_rx) = mpsc::sync_channel(1);
    let (right_tx, right_rx) = mpsc::sync_channel(1);
    thread::scope(|scope| {
        let left_gate = Arc::clone(&gate);
        scope.spawn(move || {
            left_gate.wait();
            let _ = left_tx.send(left());
        });
        let right_gate = Arc::clone(&gate);
        scope.spawn(move || {
            right_gate.wait();
            let _ = right_tx.send(right());
        });
        gate.wait();
        let left_result = left_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("left race participant completes");
        let right_result = right_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("right race participant completes");
        assert_one_success(left_result, right_result, label);
    });
    owner_root
        .owner_operation_control()
        .expect("the owner root remains live through both race contenders");
}

pub(super) fn prove_advance_is_effectful() {
    let world = AdversarialWorld::new();
    advance(&world.mutation, &world.child_basis).expect("an uncontended advance succeeds");
}

pub(super) fn prove_capture_and_restore_are_effectful() {
    let world = AdversarialWorld::new();
    let (snapshot, basis) =
        capture(&world.mutation, &world.child_basis).expect("an uncontended capture succeeds");
    restore(&world.mutation, &basis, &snapshot).expect("an uncontended restore succeeds");
}

pub(super) fn prove_retirement_is_effectful() {
    let AdversarialWorld {
        runtime,
        lifecycle,
        child_basis,
        ..
    } = AdversarialWorld::new();
    let plan =
        retirement_plan(&lifecycle, child_basis).expect("an uncontended retirement plan succeeds");
    retire(&lifecycle, plan).expect("an uncontended retirement succeeds");
    drop(runtime);
}

#[test]
fn same_branch_advance_advance_has_one_winner_and_both_orderings() {
    prove_advance_is_effectful();
    for pause_left in [true, false] {
        let world = AdversarialWorld::new();
        let control = world
            .runtime
            .as_ref()
            .expect("the race retains its owner root")
            .owner_operation_control()
            .expect("operation control is issued after sealing");
        let pause = control.arm_pause_once(
            worth_signal::facade::branch::SignalOwnerOperationBoundary::BranchRegistryLookup,
        );
        let left_mutation = world.mutation.clone();
        let right_mutation = world.mutation.clone();
        let left_basis = world.child_basis.clone();
        let right_basis = world.child_basis.clone();
        let (left_tx, left_rx) = mpsc::sync_channel(1);
        let (right_tx, right_rx) = mpsc::sync_channel(1);

        thread::scope(|scope| {
            let left = move || advance(&left_mutation, &left_basis);
            let right = move || advance(&right_mutation, &right_basis);
            if pause_left {
                scope.spawn(move || {
                    let _ = left_tx.send(left());
                });
                assert!(pause.wait_until_reached(PROGRESS_BOUND));
                scope.spawn(move || {
                    let _ = right_tx.send(right());
                });
            } else {
                scope.spawn(move || {
                    let _ = right_tx.send(right());
                });
                assert!(pause.wait_until_reached(PROGRESS_BOUND));
                scope.spawn(move || {
                    let _ = left_tx.send(left());
                });
            }
            let first = if pause_left {
                right_rx
                    .recv_timeout(PROGRESS_BOUND)
                    .expect("the unparked right operation wins")
            } else {
                left_rx
                    .recv_timeout(PROGRESS_BOUND)
                    .expect("the unparked left operation wins")
            };
            pause.release();
            let second = if pause_left {
                left_rx
                    .recv_timeout(PROGRESS_BOUND)
                    .expect("the parked left operation resolves")
            } else {
                right_rx
                    .recv_timeout(PROGRESS_BOUND)
                    .expect("the parked right operation resolves")
            };
            assert_one_success(first, second, "advance/advance");
        });
    }
}

#[test]
fn same_branch_advance_restore_has_one_winner() {
    prove_advance_is_effectful();
    prove_capture_and_restore_are_effectful();
    let world = AdversarialWorld::new();
    let (snapshot, current) = capture(&world.mutation, &world.child_basis)
        .expect("the race has a real admitted snapshot");
    let mutation_a = world.mutation.clone();
    let mutation_b = world.mutation.clone();
    let basis_a = current.clone();
    let basis_b = current.clone();
    let snapshot_b = snapshot.clone();
    run_pair(
        &world.runtime,
        move || advance(&mutation_a, &basis_a),
        move || restore(&mutation_b, &basis_b, &snapshot_b),
        "advance/restore",
    );
}

#[test]
fn same_branch_restore_restore_has_one_winner() {
    prove_capture_and_restore_are_effectful();
    let world = AdversarialWorld::new();
    let (snapshot, current) = capture(&world.mutation, &world.child_basis)
        .expect("the race has a real admitted snapshot");
    let mutation_a = world.mutation.clone();
    let mutation_b = world.mutation.clone();
    let basis_a = current.clone();
    let basis_b = current.clone();
    let snapshot_a = snapshot.clone();
    let snapshot_b = snapshot.clone();
    run_pair(
        &world.runtime,
        move || restore(&mutation_a, &basis_a, &snapshot_a),
        move || restore(&mutation_b, &basis_b, &snapshot_b),
        "restore/restore",
    );
}

#[test]
fn same_branch_snapshot_advance_has_one_winner() {
    prove_capture_and_restore_are_effectful();
    prove_advance_is_effectful();
    let world = AdversarialWorld::new();
    let mutation_a = world.mutation.clone();
    let mutation_b = world.mutation.clone();
    let basis_a = world.child_basis.clone();
    let basis_b = world.child_basis.clone();
    run_pair(
        &world.runtime,
        move || capture(&mutation_a, &basis_a).map(|_| ()),
        move || advance(&mutation_b, &basis_b),
        "snapshot/advance",
    );
}
