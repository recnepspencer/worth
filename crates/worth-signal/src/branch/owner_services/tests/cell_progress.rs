use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::Arc;
use std::thread;

use crate::state::SignalBranchId;

use super::super::{
    SignalBranchCellAdmissionDenial, SignalBranchCellPoisonRecovery, SignalBranchExecutionCell,
    SignalBranchRegistry, SignalOwnerCancellationSource, SignalOwnerLifecycleState,
    SignalOwnerServiceCounters,
};
use super::progress_bound::{wait_until_progress, worker_park, PROGRESS_BOUND};
use super::with_movement_permit;

#[test]
fn parked_cell_keeps_unrelated_registry_lookup_and_work_independently_live() {
    let (counters, lifecycle, registry) = kernel(2);
    let installation = lifecycle.admit(61).expect("owner admits installation");
    install(&registry, &installation, SignalBranchId(1), 0_u64);
    install(&registry, &installation, SignalBranchId(2), 0_u64);
    drop(installation);
    let baseline = counters.snapshot();
    let (a_park, mut a_control) = worker_park();
    let (a_done_tx, a_done_rx) = mpsc::sync_channel(1);
    let a_lifecycle = Arc::clone(&lifecycle);
    let a_registry = Arc::clone(&registry);
    thread::spawn(move || {
        let result = run_cell_work(&a_lifecycle, &a_registry, SignalBranchId(1), |state| {
            a_park.park("branch A target cell");
            *state += 1;
            *state
        });
        let _ = a_done_tx.send(result);
    });
    a_control.wait_until_parked("branch A target cell");

    let before_b = counters.snapshot();
    let (b_done_tx, b_done_rx) = mpsc::sync_channel(1);
    let b_lifecycle = Arc::clone(&lifecycle);
    let b_registry = Arc::clone(&registry);
    thread::spawn(move || {
        let result = run_cell_work(&b_lifecycle, &b_registry, SignalBranchId(2), |state| {
            *state += 1;
            *state
        });
        let _ = b_done_tx.send(result);
    });
    let b_result = b_done_rx.recv_timeout(PROGRESS_BOUND);
    let after_b = counters.snapshot();
    a_control.release();
    let a_result = a_done_rx.recv_timeout(PROGRESS_BOUND);

    assert_eq!(
        b_result,
        Ok(Ok(1)),
        "branch B registry.lookup plus cell work must finish while A is parked"
    );
    assert_eq!(a_result, Ok(Ok(1)), "branch A must finish after release");
    assert_eq!(
        after_b.branch_registry_lookups(),
        before_b.branch_registry_lookups() + 1
    );
    assert_eq!(
        after_b.target_cell_contacts(),
        before_b.target_cell_contacts() + 1
    );
    assert_eq!(after_b.target_cell_waits(), before_b.target_cell_waits());
    assert_eq!(
        after_b.canonical_movements(),
        before_b.canonical_movements() + 1
    );
    let final_snapshot = counters.snapshot();
    assert_eq!(final_snapshot.target_cell_waits(), 0);
    assert_eq!(final_snapshot.branch_registry_entries_scanned(), 0);
    assert_eq!(
        final_snapshot.canonical_movements(),
        baseline.canonical_movements() + 2
    );
}

#[test]
fn same_cell_serializes_with_bounded_exact_contact_and_wait_evidence() {
    let (counters, lifecycle, registry) = kernel(1);
    let installation = lifecycle.admit(61).expect("owner admits installation");
    let cell = install(&registry, &installation, SignalBranchId(1), 0_u64);
    drop(installation);
    let baseline = counters.snapshot();
    let (first_park, mut first_control) = worker_park();
    let (first_done_tx, first_done_rx) = mpsc::sync_channel(1);
    let first_cell = Arc::clone(&cell);
    let first_lifecycle = Arc::clone(&lifecycle);
    thread::spawn(move || {
        let result = admitted_cell_work(&first_lifecycle, &first_cell, |state| {
            first_park.park("first same-cell operation");
            *state += 1;
        });
        let _ = first_done_tx.send(result);
    });
    first_control.wait_until_parked("first same-cell operation");

    let (second_done_tx, second_done_rx) = mpsc::sync_channel(1);
    let second_cell = Arc::clone(&cell);
    let second_lifecycle = Arc::clone(&lifecycle);
    thread::spawn(move || {
        let result = admitted_cell_work(&second_lifecycle, &second_cell, |state| *state += 1);
        let _ = second_done_tx.send(result);
    });
    let wait_recorded = wait_until_progress("second same-cell wait", || {
        counters.snapshot().target_cell_waits() == baseline.target_cell_waits() + 1
    });
    let second_was_blocked = second_done_rx.try_recv() == Err(TryRecvError::Empty);
    let blocked = counters.snapshot();
    first_control.release();
    let first_result = first_done_rx.recv_timeout(PROGRESS_BOUND);
    let second_result = second_done_rx.recv_timeout(PROGRESS_BOUND);

    assert!(
        wait_recorded,
        "same-cell serialization did not record its wait"
    );
    assert!(
        second_was_blocked,
        "second same-cell operation completed before release"
    );
    assert_eq!(first_result, Ok(Ok(())));
    assert_eq!(second_result, Ok(Ok(())));
    assert_eq!(
        blocked.target_cell_contacts(),
        baseline.target_cell_contacts() + 2
    );
    assert_eq!(
        blocked.target_cell_waits(),
        baseline.target_cell_waits() + 1
    );
    assert_eq!(
        blocked.canonical_movements(),
        baseline.canonical_movements()
    );
    let observation = lifecycle.admit(61).expect("follow-up read is admitted");
    assert_eq!(read(&cell, &observation), 2);
}

#[test]
fn one_admission_denies_nested_second_cell_and_releases_on_success_and_unwind() {
    let (_counters, lifecycle, registry) = kernel(2);
    let admission = lifecycle.admit(61).expect("owner admits cell work");
    let cell_a = install(&registry, &admission, SignalBranchId(1), 0_u64);
    let cell_b = install(&registry, &admission, SignalBranchId(2), 0_u64);
    let nested_work_executed = AtomicBool::new(false);

    cell_a
        .with_state(&admission, |_, _| {
            assert_eq!(
                cell_b
                    .with_state(&admission, |state, _| {
                        nested_work_executed.store(true, Ordering::Release);
                        *state += 1;
                    })
                    .unwrap_err(),
                SignalBranchCellAdmissionDenial::SecondCellWhileHeld
            );
        })
        .expect("first cell operation remains valid");
    assert!(!nested_work_executed.load(Ordering::Acquire));
    cell_b
        .with_state(&admission, |state, _| *state += 1)
        .expect("sequential second-cell work is legal");

    let unwind = catch_unwind(AssertUnwindSafe(|| {
        cell_a
            .with_state(&admission, |_, _| {
                assert_eq!(
                    cell_b.with_state(&admission, |_, _| ()).unwrap_err(),
                    SignalBranchCellAdmissionDenial::SecondCellWhileHeld
                );
                panic!("exercise operation cell-hold unwind");
            })
            .expect("outer cell admission is valid");
    }));
    assert!(unwind.is_err());
    cell_b
        .with_state(&admission, |state, _| *state += 1)
        .expect("unwind releases the operation's cell hold");
    assert_eq!(read(&cell_b, &admission), 2);
}

#[test]
fn cell_poison_policy_quarantines_partial_mutation_and_contains_failure() {
    let (counters, lifecycle, registry) = kernel(2);
    let installation = lifecycle.admit(61).expect("owner admits installation");
    let branch_a = install(&registry, &installation, SignalBranchId(1), 0_u64);
    let branch_b = install(&registry, &installation, SignalBranchId(2), 0_u64);
    drop(installation);
    let baseline = counters.snapshot();

    let panic_admission = lifecycle.admit(61).expect("panic path is admitted");
    let panic_result = catch_unwind(AssertUnwindSafe(|| {
        branch_a
            .with_state(&panic_admission, |state, work| {
                *state += 1;
                with_movement_permit(|permit| work.record_canonical_movement(permit));
                panic!("exercise branch-local poison recovery");
            })
            .expect("admission is valid before the injected panic");
    }));
    assert!(panic_result.is_err());
    drop(panic_admission);

    let healthy_admission = lifecycle.admit(61).expect("owner stays open");
    branch_b
        .with_state(&healthy_admission, |state, work| {
            *state += 1;
            with_movement_permit(|permit| work.record_canonical_movement(permit));
        })
        .expect("unrelated branch remains healthy");
    assert_eq!(
        branch_a
            .with_state(&healthy_admission, |state, _| *state += 1)
            .unwrap_err(),
        SignalBranchCellAdmissionDenial::PoisonedIncarnation
    );

    assert_eq!(
        branch_a.poison_recovery(),
        Some(SignalBranchCellPoisonRecovery::TerminallyQuarantinedPartialMutation)
    );
    assert_eq!(read(&branch_b, &healthy_admission), 1);
    assert_eq!(
        counters.snapshot().canonical_movements(),
        baseline.canonical_movements() + 2
    );
}

#[test]
fn cell_work_records_exact_semantic_deltas_without_reconstruction() {
    let (counters, lifecycle, registry) = kernel(1);
    let admission = lifecycle.admit(61).expect("owner admits work");
    let cell = install(&registry, &admission, SignalBranchId(1), 0_u64);
    let baseline = counters.snapshot();
    let (_, fork_work) = crate::data::graph::SignalGraph::new().fork_persistent();

    cell.with_state(&admission, |state, work| {
        *state = 9;
        with_movement_permit(|permit| work.record_canonical_movement(permit));
        work.record_retention_registry_contact();
        work.record_fork_source_capture(fork_work);
        work.record_diagnostic_event();
        work.record_dropped_diagnostic_event();
    })
    .expect("cell operation completes");

    let snapshot = counters.snapshot();
    assert_eq!(
        snapshot.target_cell_contacts(),
        baseline.target_cell_contacts() + 1
    );
    assert_eq!(snapshot.target_cell_waits(), baseline.target_cell_waits());
    assert_eq!(
        snapshot.canonical_movements(),
        baseline.canonical_movements() + 1
    );
    assert_eq!(snapshot.retention_registry_contacts(), 1);
    assert_eq!(snapshot.fork_source_captures(), 1);
    assert_eq!(snapshot.diagnostic_events_recorded(), 1);
    assert_eq!(snapshot.diagnostic_events_dropped(), 1);
    assert_eq!(snapshot.forked_mutable_graph_nodes_copied(), 0);
    assert_eq!(snapshot.branch_registry_entries_scanned(), 0);
}

#[test]
fn cancellation_after_preflight_cannot_erase_recorded_movement() {
    let (counters, lifecycle, registry) = kernel(1);
    let admission = lifecycle.admit(61).expect("owner admits work");
    let cell = install(&registry, &admission, SignalBranchId(1), 0_u64);
    let cancellation = SignalOwnerCancellationSource::new();
    let token = cancellation.token();
    let permit = token
        .preflight_movement()
        .expect("cancellation is open before linearization");
    cancellation.cancel();

    cell.with_state(&admission, |state, work| {
        *state = 1;
        work.record_canonical_movement(&permit);
    })
    .expect("late cancellation remains descriptive");

    assert!(token.preflight_movement().is_err());
    assert_eq!(read(&cell, &admission), 1);
    assert_eq!(counters.snapshot().canonical_movements(), 1);
}

fn kernel(
    maximum_live_branches: usize,
) -> (
    Arc<SignalOwnerServiceCounters>,
    Arc<SignalOwnerLifecycleState>,
    Arc<SignalBranchRegistry<u64>>,
) {
    let counters = Arc::new(SignalOwnerServiceCounters::default());
    let lifecycle = SignalOwnerLifecycleState::new(61, Arc::clone(&counters));
    let registry = Arc::new(SignalBranchRegistry::new(
        &lifecycle,
        maximum_live_branches,
        maximum_live_branches,
    ));
    (counters, lifecycle, registry)
}

fn install(
    registry: &SignalBranchRegistry<u64>,
    admission: &super::super::SignalOwnerOperationAdmission<'_>,
    branch_id: SignalBranchId,
    state: u64,
) -> Arc<SignalBranchExecutionCell<u64>> {
    registry
        .reserve(admission, branch_id)
        .expect("branch identity reserves")
        .install(state)
        .expect("branch state installs")
}

fn run_cell_work(
    lifecycle: &Arc<SignalOwnerLifecycleState>,
    registry: &SignalBranchRegistry<u64>,
    branch_id: SignalBranchId,
    operation: impl FnOnce(&mut u64) -> u64,
) -> Result<u64, String> {
    let admission = lifecycle.admit(61).map_err(|error| format!("{error:?}"))?;
    let cell = registry
        .lookup(&admission, branch_id)
        .map_err(|error| format!("{error:?}"))?;
    cell.with_state(&admission, |state, work| {
        let result = operation(state);
        with_movement_permit(|permit| work.record_canonical_movement(permit));
        result
    })
    .map_err(|error| format!("{error:?}"))
}

fn admitted_cell_work(
    lifecycle: &Arc<SignalOwnerLifecycleState>,
    cell: &SignalBranchExecutionCell<u64>,
    operation: impl FnOnce(&mut u64),
) -> Result<(), String> {
    let admission = lifecycle.admit(61).map_err(|error| format!("{error:?}"))?;
    cell.with_state(&admission, |state, work| {
        operation(state);
        with_movement_permit(|permit| work.record_canonical_movement(permit));
    })
    .map_err(|error| format!("{error:?}"))
}

fn read(
    cell: &SignalBranchExecutionCell<u64>,
    admission: &super::super::SignalOwnerOperationAdmission<'_>,
) -> u64 {
    cell.with_state(admission, |state, _| *state)
        .expect("cell observation completes")
}
