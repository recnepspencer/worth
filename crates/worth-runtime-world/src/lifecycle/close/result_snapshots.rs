//! Incremental original-result retirement; final owner/task Drop is unbudgeted.
use crate::history::{CompositeHistoryCatalog, RuntimeWorldPublicationFrontier};
use crate::identity::CompositeCommitIdentity;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::branch::{
    RelationalBranchBasisPort, RelationalCommitResultSnapshotCleanup as NativeTask,
    RelationalCommitResultSnapshotCleanupDenial as NativeDenial,
    RelationalCommitResultSnapshotCleanupPending as NativePending,
    RelationalCommitResultSnapshotCleanupProgress as NativeProgress,
    RelationalCommitResultSnapshotRelease as Released,
};
use worth_relational::facade::{runtime::RelationalRuntime, transactions::CommitResult};

#[derive(Debug)]
pub struct RuntimeWorldResultSnapshotCleanup {
    native: RelationalBranchBasisPort,
    history: CompositeHistoryCatalog,
    frontier: RuntimeWorldPublicationFrontier,
    after: Option<CompositeCommitIdentity>,
    current: Option<CurrentResult>,
    counts: RuntimeWorldResultSnapshotCleanupReport,
    terminal: bool,
}

#[derive(Debug)]
struct CurrentResult {
    identity: CompositeCommitIdentity,
    original: Arc<CommitResult>,
    observer: Option<NativeTask>,
    last: bool,
}

#[derive(Debug)]
pub enum RuntimeWorldResultSnapshotCleanupDenial {
    Busy,
    Stopped(WorkCeilingDenial),
    CloseNotCompleted,
    OwnerUnavailable,
    InconsistentSealedCatalog,
    Native(NativeDenial),
}

#[derive(Debug)]
pub enum RuntimeWorldResultSnapshotCleanupProgress {
    Advanced,
    Complete(RuntimeWorldResultSnapshotCleanupReport),
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeWorldResultSnapshotCleanupReport {
    released: usize,
    already_released: usize,
}
impl RuntimeWorldResultSnapshotCleanupReport {
    pub fn released(&self) -> usize {
        self.released
    }
    pub fn already_released(&self) -> usize {
        self.already_released
    }
}

impl RuntimeWorldResultSnapshotCleanup {
    pub(in crate::lifecycle) fn from_closed_owner(
        native: RelationalBranchBasisPort,
        history: CompositeHistoryCatalog,
        frontier: RuntimeWorldPublicationFrontier,
    ) -> Self {
        let terminal = frontier.cleanup_is_empty();
        Self {
            native,
            history,
            frontier,
            after: None,
            current: None,
            counts: RuntimeWorldResultSnapshotCleanupReport {
                released: 0,
                already_released: 0,
            },
            terminal,
        }
    }

    /// One row visit or authenticated Native step. Home retains this task and
    /// both actual owners across every return. No owner calls occur under a
    /// World lock. A lost observer rejoins only the SAME original result cell.
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        owner: &RelationalRuntime,
    ) -> Result<RuntimeWorldResultSnapshotCleanupProgress, RuntimeWorldResultSnapshotCleanupDenial>
    {
        use RuntimeWorldResultSnapshotCleanupDenial as Denial;
        use RuntimeWorldResultSnapshotCleanupProgress as Progress;
        if self.terminal {
            return Ok(Progress::Complete(self.counts));
        }
        if let Some(current) = self.current.as_mut() {
            // Native observes actual Complete before consulting a late stop.
            // original stays here even while the consuming observer is absent.
            let task = current.observer.take().unwrap_or_else(|| {
                self.native
                    .begin_commit_result_snapshot_cleanup(current.original.clone())
            });
            match task.advance(request, owner) {
                NativeProgress::Pending { task, cause } => {
                    current.observer = Some(task);
                    return match cause {
                        NativePending::Advanced => Ok(Progress::Advanced),
                        NativePending::Busy => Err(Denial::Busy),
                        NativePending::Stopped(cause) => Err(Denial::Stopped(cause)),
                    };
                }
                NativeProgress::Refused { task, cause } => {
                    current.observer = Some(task);
                    return Err(Denial::Native(cause));
                }
                NativeProgress::Complete { result, release } => {
                    current.original = result;
                    // Counts describe observed Complete variants, not which
                    // lost observer originally removed a registration.
                    match release {
                        Released::Released(_) => self.counts.released += 1,
                        Released::AlreadyReleased { .. } => self.counts.already_released += 1,
                    }
                    self.after = Some(current.identity.clone());
                    self.terminal = current.last;
                    // Same catalog still owns this result; this cannot be its
                    // final Drop. Native owner borrow also spans this return.
                    drop(self.current.take());
                }
            }
        } else {
            request.consult().map_err(Denial::Stopped)?;
            let row = self
                .history
                .try_result_snapshot_cleanup_row(&self.frontier, self.after.as_ref())?;
            if let Some(original) = row.result {
                self.current = Some(CurrentResult {
                    identity: row.identity,
                    original,
                    observer: None,
                    last: row.last,
                });
            } else {
                self.after = Some(row.identity);
                self.terminal = row.last;
            }
        }
        // Last captured identity completes terminal state immediately, with no
        // extra request check that could hide an already observed completion.
        if self.terminal {
            Ok(Progress::Complete(self.counts))
        } else {
            Ok(Progress::Advanced)
        }
    }
}

#[cfg(test)]
mod tests;
