pub(crate) use std::collections::BTreeSet;
pub(crate) use std::time::Duration;
pub(crate) use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;
pub(crate) use {std::sync::atomic::AtomicBool, std::sync::atomic::Ordering};
pub(crate) use {std::sync::mpsc::sync_channel, std::sync::mpsc::RecvTimeoutError};
pub(crate) use {std::sync::Arc, std::sync::Barrier};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch_with_result, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::fork_supply_chain_branch_from_main, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld};
pub(crate) use {worth_relational::facade::branch::RelationalBranchReferenceState};
pub(crate) use {worth_relational::facade::history::BranchId, worth_relational::facade::history::RelationalCommitReceipt};
pub(crate) use {worth_relational::facade::mvcc::BranchBoundRelationalTransaction, worth_relational::facade::mvcc::RelationalInterruptionBoundary, worth_relational::facade::mvcc::RelationalOperationControl, worth_relational::facade::mvcc::RelationalTransactionIntent};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::CommitResult, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) const OWNER_PHASE_COURT_TIMEOUT: Duration = std::time::Duration::from_secs(5);

pub(crate) struct OwnerPhaseCourtBranches {
    pub(crate) storm: BranchId,
    pub(crate) maintenance: BranchId,
}

pub(crate) struct PausedCourtObservation {
    pub(crate) storm_reference: RelationalBranchReferenceState,
    pub(crate) storm_coordination: BranchCoordination,
    pub(crate) maintenance_reference: RelationalBranchReferenceState,
    pub(crate) maintenance_head: RelationalCommitReceipt,
}

/// Coordination accounting the public sharing observation reports for a branch.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BranchCoordination {
    pub(crate) contacts: u64,
    pub(crate) waits: u64,
}

impl OwnerPhaseCourtBranches {
    pub(crate) fn fork(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        world: &ProductionSeededSupplyChainWorld,
    ) -> Self {
        let storm = BranchId("storm".to_owned());
        let maintenance = BranchId("maintenance".to_owned());
        fork_supply_chain_branch_from_main(execution, &world.runtime, storm.clone());
        fork_supply_chain_branch_from_main(execution, &world.runtime, maintenance.clone());
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
    pub(crate) fn contested_violation(
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

pub(crate) struct OwnerPhasePause {
    pub(crate) reached: Arc<Barrier>,
    pub(crate) release: Arc<Barrier>,
    pub(crate) opened: AtomicBool,
}

impl OwnerPhasePause {
    pub(crate) fn new() -> Self {
        Self {
            reached: std::sync::Arc::new(std::sync::Barrier::new(2)),
            release: std::sync::Arc::new(std::sync::Barrier::new(2)),
            opened: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub(crate) fn control(
        &self,
        boundary: RelationalInterruptionBoundary,
    ) -> RelationalOperationControl {
        worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted()
            .with_boundary_pause_for_test(
                boundary,
                std::sync::Arc::clone(&self.reached),
                std::sync::Arc::clone(&self.release),
            )
    }

    pub(crate) fn await_arrival(&self, phase: &str) {
        let (arrived, arrival) = sync_channel(1);
        let reached = std::sync::Arc::clone(&self.reached);
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
    pub(crate) fn signal_release(&self, confirm: bool) -> bool {
        if self.opened.swap(true, std::sync::atomic::Ordering::AcqRel) {
            return true;
        }
        let (released, completion) = sync_channel(1);
        let release = std::sync::Arc::clone(&self.release);
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

/// Compare one court commit against the independent Supply Chain oracle and
/// then close the exact snapshot it named, so a finished court leaves no open
/// published-snapshot handle behind.
pub(crate) fn assert_court_commit_matches_oracle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
    committed: &CommitResult,
    branch: BranchLabel,
    delta: DeltaId,
) {
    let observed = observe_supply_chain_snapshot(
        request_execution,
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
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
    branch: &BranchId,
    delta: DeltaId,
) -> WorkerIntentBatch {
    lower_supply_chain_production_delta(
        request_execution,
        &world.runtime,
        &world.program,
        &world.handles,
        branch,
        &std::collections::BTreeSet::new(),
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
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            control,
        )
        .expect("the controlled court transaction binds to its exact admitted basis");
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
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
