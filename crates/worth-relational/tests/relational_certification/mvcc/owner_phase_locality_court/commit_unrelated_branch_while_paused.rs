//! commit unrelated branch while paused.
use super::*;

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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
            let committed = commit_branch_batch_with_result(execution, runtime, branch, batch);
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
