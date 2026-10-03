use crate::domain_computation::primary_graph::WorthQueryApplicationSnapshotRelease;
use crate::domain_computation::{
    WorthQueryManagedRunTerminalKind, WorthQueryMutationGraphWorkCompletion,
    WorthQueryMutationRunBinding, WorthQueryProviderSessionBoundMutationRun,
    WorthQueryRunningDirectRun,
};

/// Private cleanup owner for the managed mutation run.
///
/// Provider-bound is the only posture that can complete a commit receipt. An
/// unbound owner remains necessary to close a run whose provider admission was
/// denied before a session existed.
pub(super) enum WorthQueryApplicationMutationCleanupOwner {
    Unbound(WorthQueryMutationRunBinding),
    ProviderBound(WorthQueryProviderSessionBoundMutationRun),
}

impl WorthQueryApplicationMutationCleanupOwner {
    pub(super) fn finish(
        self,
        running: WorthQueryRunningDirectRun,
        terminal: WorthQueryManagedRunTerminalKind,
        snapshot_release: WorthQueryApplicationSnapshotRelease,
    ) -> Result<WorthQueryMutationGraphWorkCompletion, ()> {
        match self {
            Self::Unbound(run) => run.finish(running, terminal, snapshot_release),
            Self::ProviderBound(run) => run.finish(running, terminal, snapshot_release),
        }
    }
}
