//! The focused court proving a paused settlement holds no runtime-wide
//! authority.
//!
//! Every wait here is bounded and every exit opens the park. A branch parked
//! inside its settlement executor holds an admitted runtime operation, and the
//! owner close waits on exactly that, so a wait without a bound, or an exit
//! that left the park closed, would replace this court's named diagnostic with
//! a hung lib suite. The release is therefore held by a guard declared after
//! the runtime, which drop order runs before the runtime it has to unblock.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, RecvTimeoutError};
use std::sync::{Arc, Barrier};
use std::time::Duration;

use super::owner_service_phase3::{fork_from_main, perform_write_with_control};
use crate::tests::support::*;

/// Bound on the paused-settlement court's waits. A paused branch holds an
/// admitted runtime operation and the owner close waits on exactly that, so an
/// unbounded wait here would hang the lib suite instead of naming the defect.
const PAUSED_SETTLEMENT_COURT_TIMEOUT: Duration = Duration::from_secs(5);

/// The sole authority for opening this court's settlement park. The court holds
/// it in a value declared after the runtime, so every exit, returning or
/// unwinding, releases the park before the runtime close that waits on it.
struct SettlementParkRelease {
    release: Arc<Barrier>,
    opened: AtomicBool,
}

impl SettlementParkRelease {
    fn new(release: Arc<Barrier>) -> Self {
        Self {
            release,
            opened: AtomicBool::new(false),
        }
    }

    /// Open the park exactly once, and never block the court doing it. The
    /// barrier half runs on a proxy that carries no runtime, so a branch that
    /// never parked cannot strand the release, while the once-only guard keeps
    /// a repeat call from leaving a second proxy waiting on a partner that has
    /// already gone.
    fn open(&self) {
        if self.opened.swap(true, Ordering::AcqRel) {
            return;
        }
        let release = Arc::clone(&self.release);
        std::thread::spawn(move || {
            release.wait();
        });
    }
}

impl Drop for SettlementParkRelease {
    fn drop(&mut self) {
        self.open();
    }
}

/// Settlement holds no runtime-wide authority. A branch paused inside its own
/// settlement executor never contacts, and never blocks, an unrelated branch
/// running the full convenience commit path.
#[test]
fn phase3_paused_settlement_does_not_block_an_unrelated_branch_commit() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "phase3-settlement-independence-anchor");
    fork_from_main(&runtime, "paused-settlement");
    fork_from_main(&runtime, "progressing-settlement");

    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let park = SettlementParkRelease::new(Arc::clone(&release));
    let control = crate::mvcc::RelationalOperationControl::uninterrupted().with_boundary_pause(
        crate::mvcc::RelationalInterruptionBoundary::Settlement,
        Arc::clone(&reached),
        Arc::clone(&release),
    );
    let performed =
        perform_write_with_control(&runtime, "paused-settlement", "paused-write", control);
    let paused_commit_id = performed.canonical_commit().commit.commit_id;
    let paused_cell = runtime
        .history
        .branch_cell(&BranchId("paused-settlement".to_owned()))
        .expect("paused branch cell");
    let contacts_before = paused_cell.coordination().contact_count();
    let waits_before = paused_cell.coordination().wait_count();

    let settlement = runtime.settlement_port();
    let paused_port = settlement.clone();
    let (settled, settlement_completion) = sync_channel(1);
    let paused_thread = std::thread::spawn(move || {
        let result = paused_port.settle_performed_publication(performed);
        let _ = settled.send(());
        result
    });

    let (arrived, arrival) = sync_channel(1);
    let arrival_proxy = Arc::clone(&reached);
    std::thread::spawn(move || {
        arrival_proxy.wait();
        let _ = arrived.send(());
    });
    if arrival
        .recv_timeout(PAUSED_SETTLEMENT_COURT_TIMEOUT)
        .is_err()
    {
        panic!("branch A never reached its settlement executor pause");
    }
    assert!(
        !paused_thread.is_finished(),
        "branch A remains paused inside its settlement executor"
    );
    let mut progressing = test_owner_begin_transaction_for_branch(
        &runtime,
        BranchId("progressing-settlement".to_owned()),
    );
    progressing
        .push_batch(
            batch_create("progressing-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    // Branch B commits on a worker so this court can bound it: only a thread
    // outside the commit can hold the completion receiver and convict
    // serialization by name instead of blocking here forever. The sender moves
    // into the worker, so a panicking branch B disconnects the channel at once
    // and this court reports that panic instead of charging the delay to
    // serialization.
    let (finished, completion) = sync_channel(1);
    let committing = &runtime;
    let progressing_result = std::thread::scope(|scope| {
        let committer = scope.spawn(move || {
            let committed = committing
                .commit_branch_transaction(
                    progressing,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("branch B commits end to end while branch A settlement is paused");
            finished
                .send(())
                .expect("the focused court still owns the completion receiver");
            committed
        });
        match completion.recv_timeout(PAUSED_SETTLEMENT_COURT_TIMEOUT) {
            Ok(()) => {}
            // Scope teardown joins branch B before an unwind can reach the
            // guard's drop, and a serialized branch B cannot finish while
            // branch A is parked, so these two exits open the park themselves.
            Err(RecvTimeoutError::Timeout) => {
                park.open();
                panic!("branch B serialized behind branch A's paused settlement executor");
            }
            Err(RecvTimeoutError::Disconnected) => park.open(),
        }
        match committer.join() {
            Ok(committed) => committed,
            Err(worker_panic) => std::panic::resume_unwind(worker_panic),
        }
    });
    assert!(
        !paused_thread.is_finished(),
        "branch B did not release branch A"
    );
    assert_eq!(
        paused_cell.coordination().contact_count(),
        contacts_before,
        "branch B never contacts branch A coordination",
    );
    assert_eq!(
        paused_cell.coordination().wait_count(),
        waits_before,
        "branch B never waits on branch A coordination",
    );
    assert!(
        settlement.retains_pending_settlement(paused_commit_id),
        "branch A's parked record stays installed for the whole unrelated commit"
    );
    assert!(
        !settlement.retains_pending_settlement(progressing_result.commit.commit_id),
        "branch B's finished commit leaves no pending settlement record behind"
    );

    park.open();
    settlement_completion
        .recv_timeout(PAUSED_SETTLEMENT_COURT_TIMEOUT)
        .expect("branch A never finished settling after its pause opened");
    let paused_result = paused_thread
        .join()
        .expect("paused settlement worker joins")
        .expect("branch A settles after release");
    assert_eq!(paused_result.commit.commit_id, paused_commit_id);
    assert!(
        !settlement.retains_pending_settlement(paused_commit_id),
        "branch A's finished settlement leaves no pending record behind"
    );
    release_test_commit_snapshot(&runtime, &paused_result);
    release_test_commit_snapshot(&runtime, &progressing_result);
}
