//! The bounded two-branch Supply Chain court every owner-phase locality proof
//! runs in.
//!
//! It forks the unrelated branches from one root, parks one of them at a real
//! production interruption boundary, and runs the other branch's full ordinary
//! commit under a bound. Every wait here is bounded on purpose: a phase that
//! never parks, or an unrelated commit that serializes behind the park, must
//! convict by name inside the court's budget instead of hanging the lane.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, RecvTimeoutError};
use std::sync::{Arc, Barrier};
use std::time::Duration;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

use super::invariant_oracle_expectations::expected_supply_chain_branch;
use super::world::supply_chain::{
    commit_branch_batch_with_result, compare, fork_supply_chain_branch_from_main,
    head_for_supply_chain_branch, lower_supply_chain_production_delta,
    observe_supply_chain_snapshot, BranchLabel, DeltaId, ProductionSeededSupplyChainWorld,
};
use worth_relational::facade::branch::RelationalBranchReferenceState;
use worth_relational::facade::history::{BranchId, RelationalCommitReceipt};
use worth_relational::facade::mvcc::{
    BranchBoundRelationalTransaction, RelationalInterruptionBoundary, RelationalOperationControl,
    RelationalTransactionIntent,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{CommitResult, WorkerIntentBatch};

/// Bound on every court wait; it only decides how fast serialization convicts.
///
/// A whole court commits in well under a second in this memory-resident world,
/// so this is generous headroom rather than a guess, and it keeps a convicted
/// lane fast enough to read as a test failure instead of a stuck job.
pub(crate) const OWNER_PHASE_COURT_TIMEOUT: Duration = Duration::from_secs(5);

/// The unrelated Supply Chain branches every locality court forks from one root.
pub(crate) struct OwnerPhaseCourtBranches {
    pub(crate) storm: BranchId,
    pub(crate) maintenance: BranchId,
}

/// Owner-state evidence captured while the storm branch is paused.
pub(crate) struct PausedCourtObservation {
    pub(crate) storm_reference: RelationalBranchReferenceState,
    pub(crate) storm_coordination: BranchCoordination,
    maintenance_reference: RelationalBranchReferenceState,
    maintenance_head: RelationalCommitReceipt,
}

/// Coordination accounting the public sharing observation reports for a branch.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BranchCoordination {
    pub(crate) contacts: u64,
    pub(crate) waits: u64,
}

impl OwnerPhaseCourtBranches {
    pub(crate) fn fork(world: &ProductionSeededSupplyChainWorld) -> Self {
        let storm = BranchId("storm".to_owned());
        let maintenance = BranchId("maintenance".to_owned());
        fork_supply_chain_branch_from_main(&world.runtime, storm.clone());
        fork_supply_chain_branch_from_main(&world.runtime, maintenance.clone());
        Self { storm, maintenance }
    }

    pub(crate) fn capture(&self, runtime: &RelationalRuntime) -> PausedCourtObservation {
        PausedCourtObservation {
            storm_reference: branch_reference_state(runtime, &self.storm),
            storm_coordination: coordination_counters(runtime, &self.storm),
            maintenance_reference: branch_reference_state(runtime, &self.maintenance),
            maintenance_head: head_for_supply_chain_branch(runtime, &self.maintenance),
        }
    }
}

impl PausedCourtObservation {
    pub(crate) fn assert_branch_locality(
        &self,
        court: &OwnerPhaseCourtBranches,
        runtime: &RelationalRuntime,
        committed: &CommitResult,
    ) {
        assert_eq!(
            branch_reference_state(runtime, &court.storm),
            self.storm_reference,
            "an unrelated ordinary commit must not move the paused storm reference"
        );
        assert_eq!(
            coordination_counters(runtime, &court.storm),
            self.storm_coordination,
            "an unrelated ordinary commit must add exactly zero storm contacts and waits"
        );
        assert_ne!(
            branch_reference_state(runtime, &court.maintenance),
            self.maintenance_reference,
            "the unrelated maintenance reference must move while storm stays paused"
        );
        assert_eq!(committed.commit.branch_id, court.maintenance);
        assert_eq!(
            committed.commit.parents,
            vec![self.maintenance_head.commit_id],
            "the unrelated maintenance head advances exactly one canonical commit"
        );
        assert_eq!(
            head_for_supply_chain_branch(runtime, &court.maintenance),
            committed.commit,
            "the observed maintenance head is exactly the returned canonical commit"
        );
    }

    /// Report a paused branch that moved or coordinated while the unrelated
    /// commit was still in flight.
    ///
    /// The park is held across that whole window, so storm's own reference and
    /// counters are stable inside it; reading them there refuses evidence a
    /// gate taken and released inside the unrelated commit would leave equal at
    /// both endpoints. The violation is returned rather than raised so the
    /// caller can open the park before unwinding through scope teardown.
    fn contested_violation(
        &self,
        court: &OwnerPhaseCourtBranches,
        runtime: &RelationalRuntime,
    ) -> Option<String> {
        let coordination = coordination_counters(runtime, &court.storm);
        if branch_reference_state(runtime, &court.storm) != self.storm_reference {
            Some("the in-flight unrelated commit moved the paused storm reference".to_owned())
        } else if coordination != self.storm_coordination {
            Some(format!(
                "the in-flight unrelated commit charged the paused storm branch {coordination:?} against {:?}",
                self.storm_coordination
            ))
        } else {
            None
        }
    }
}

/// One owner-phase pause held by a worker branch. Both halves are observed
/// through a proxy thread and a bounded channel, so neither a phase that never
/// parks nor a park nobody is waiting on can block the court on a barrier.
///
/// Opening is not politeness. A parked production thread holds an admitted
/// runtime operation, and the owner close this world drops through waits while
/// any operation is in flight. A court that panicked with its park still closed
/// would hang in drop and print no diagnostic at all, so the park opens itself
/// on the way out and every exit from a park is bounded.
pub(crate) struct OwnerPhasePause {
    reached: Arc<Barrier>,
    release: Arc<Barrier>,
    opened: AtomicBool,
}

impl OwnerPhasePause {
    pub(crate) fn new() -> Self {
        Self {
            reached: Arc::new(Barrier::new(2)),
            release: Arc::new(Barrier::new(2)),
            opened: AtomicBool::new(false),
        }
    }

    pub(crate) fn control(
        &self,
        boundary: RelationalInterruptionBoundary,
    ) -> RelationalOperationControl {
        RelationalOperationControl::uninterrupted().with_boundary_pause(
            boundary,
            Arc::clone(&self.reached),
            Arc::clone(&self.release),
        )
    }

    pub(crate) fn await_arrival(&self, phase: &str) {
        let (arrived, arrival) = sync_channel(1);
        let reached = Arc::clone(&self.reached);
        let proxy = std::thread::spawn(move || {
            reached.wait();
            let _ = arrived.send(());
        });
        if arrival.recv_timeout(OWNER_PHASE_COURT_TIMEOUT).is_err() {
            panic!("the storm branch never reached its {phase} pause");
        }
        proxy.join().expect("the pause-arrival proxy joins");
    }

    /// Release the park at most once. The barrier half is always waited on a
    /// proxy thread, so a branch that parks late is still freed; `confirm` only
    /// decides whether the court waits here for that to happen.
    fn signal_release(&self, confirm: bool) -> bool {
        if self.opened.swap(true, Ordering::AcqRel) {
            return true;
        }
        let (released, completion) = sync_channel(1);
        let release = Arc::clone(&self.release);
        std::thread::spawn(move || {
            release.wait();
            let _ = released.send(());
        });
        !confirm || completion.recv_timeout(OWNER_PHASE_COURT_TIMEOUT).is_ok()
    }

    pub(crate) fn open(&self) {
        assert!(
            self.signal_release(true),
            "no branch was waiting at the owner-phase pause the court tried to open"
        );
    }
}

impl Drop for OwnerPhasePause {
    /// Release a park the court left closed. Without this a failing assertion
    /// between arrival and release would strand the parked branch, and the
    /// owner close in this world's drop would wait on it forever, replacing the
    /// court's own named diagnostic with a hang.
    ///
    /// This hands the release to a proxy and returns rather than waiting for
    /// it. That owner close is already the synchronization point: it cannot
    /// finish until the freed branch returns, so waiting again here would only
    /// charge a second budget to a court that has already failed.
    fn drop(&mut self) {
        let _ = self.signal_release(false);
    }
}

/// Open the park and then take the worker's result under a bound.
///
/// Joining a worker whose park is still closed is exactly what turns a failing
/// court into a hang, so opening first belongs to this call rather than to the
/// caller's discipline. A join proxy signals completion, including a panic,
/// so the court can bound its receive without polling the worker.
pub(crate) fn join_paused_worker<T: Send + 'static>(
    worker: std::thread::JoinHandle<T>,
    pause: &OwnerPhasePause,
    phase: &str,
) -> T {
    pause.open();
    let (finished, completion) = sync_channel(1);
    std::thread::spawn(move || {
        let _ = finished.send(worker.join());
    });
    match completion
        .recv_timeout(OWNER_PHASE_COURT_TIMEOUT)
        .unwrap_or_else(|_| {
            panic!("the storm branch never finished its {phase} after the court opened its pause")
        }) {
        Ok(value) => value,
        Err(worker_panic) => std::panic::resume_unwind(worker_panic),
    }
}

/// Run one full ordinary Supply Chain commit on the unrelated branch while the
/// storm branch holds an owner-phase park. A worker runs it so the court can
/// bound it: only a thread outside the commit can hold the completion receiver
/// and convict serialization by name instead of hanging. The worker is scoped
/// because it borrows the court-owned world in place: `std::thread::scope` is
/// what lets that borrow cross the spawn without manufacturing `'static`
/// ownership of the runtime, and it ends that borrow with the court's own
/// frame. Both halves reach the runtime through a shared reference, and the
/// court reads the paused branch while the worker still holds its own, so
/// reintroducing an exclusive borrow here stops compiling instead of quietly
/// becoming a locality claim nothing observes. Scope teardown joins, so every
/// failing exit opens the park before it returns.
pub(crate) fn commit_unrelated_branch_while_paused(
    world: &ProductionSeededSupplyChainWorld,
    court: &OwnerPhaseCourtBranches,
    before: &PausedCourtObservation,
    batch: WorkerIntentBatch,
    pause: &OwnerPhasePause,
    phase: &str,
) -> CommitResult {
    let (finished, completion) = sync_channel(1);
    let branch = court.maintenance.clone();
    std::thread::scope(|scope| {
        let runtime = &world.runtime;
        let committer = scope.spawn(move || {
            let committed = commit_branch_batch_with_result(runtime, branch, batch);
            finished
                .send(())
                .expect("the locality court still owns the completion receiver");
            committed
        });
        if let Some(violation) = before.contested_violation(court, &world.runtime) {
            pause.open();
            panic!("{violation}");
        }
        match completion.recv_timeout(OWNER_PHASE_COURT_TIMEOUT) {
            Ok(()) => {}
            Err(RecvTimeoutError::Timeout) => {
                pause.open();
                panic!(
                    "the unrelated maintenance commit serialized behind the paused storm {phase}"
                );
            }
            Err(RecvTimeoutError::Disconnected) => pause.open(),
        }
        match committer.join() {
            Ok(committed) => committed,
            Err(worker_panic) => std::panic::resume_unwind(worker_panic),
        }
    })
}

/// Compare one court commit against the independent Supply Chain oracle and
/// then close the exact snapshot it named, so a finished court leaves no open
/// published-snapshot handle behind.
pub(crate) fn assert_court_commit_matches_oracle(
    world: &ProductionSeededSupplyChainWorld,
    committed: &CommitResult,
    branch: BranchLabel,
    delta: DeltaId,
) {
    let observed = observe_supply_chain_snapshot(
        &world.program,
        &world.handles.for_snapshot(committed.snapshot.clone()),
        &world.runtime,
        &committed.snapshot,
    )
    .expect("the performed court root is observable through its exact snapshot");
    let expected = expected_supply_chain_branch(&world.program, branch, Some(delta));
    compare(&expected, &observed)
        .expect("the court commit matches the independent Supply Chain oracle");
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .expect("the exact court commit snapshot closes once comparison completes");
}

pub(crate) fn lower_court_delta(
    world: &ProductionSeededSupplyChainWorld,
    branch: &BranchId,
    delta: DeltaId,
) -> WorkerIntentBatch {
    lower_supply_chain_production_delta(
        &world.runtime,
        &world.program,
        &world.handles,
        branch,
        &BTreeSet::new(),
        delta,
    )
    .expect("the production lowerer accepts the court branch's actual observed state")
}

pub(crate) fn begin_court_transaction(
    runtime: &RelationalRuntime,
    branch: &BranchId,
    control: RelationalOperationControl,
    batch: WorkerIntentBatch,
) -> BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(branch)
        .expect("the court branch identity is owner-issued");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("the court branch basis is owner-admitted");
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .expect("the controlled court transaction binds to its exact admitted basis");
    transaction
        .push_batch(batch, AllocationPolicy::SystemAllocation)
        .expect("the lowered Supply Chain delta fits the configured transaction budget");
    transaction
}

pub(crate) fn branch_reference_state(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> RelationalBranchReferenceState {
    runtime
        .branch_reference_state(branch)
        .expect("every court branch keeps a live owner reference cell")
}

pub(crate) fn coordination_counters(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> BranchCoordination {
    let identity = runtime
        .branch_identity(branch)
        .expect("the court branch identity is owner-issued");
    let observation = runtime
        .observe_branch_sharing(std::slice::from_ref(&identity))
        .expect("the paused court branch remains inspectable through public sharing");
    BranchCoordination {
        contacts: observation.coordination_contacts(),
        waits: observation.coordination_waits(),
    }
}
