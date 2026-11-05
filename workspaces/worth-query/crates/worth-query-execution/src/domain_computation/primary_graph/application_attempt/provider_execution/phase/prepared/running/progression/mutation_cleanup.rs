use crate::domain_computation::vocabulary::primary_graph::snapshot_lease::WorthQueryApplicationSnapshotRelease;
use {
    crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind,
    crate::domain_computation::WorthQueryMutationGraphWorkCompletion,
    crate::domain_computation::WorthQueryMutationRunBinding,
    crate::domain_computation::WorthQueryProviderSessionBoundMutationRun,
    crate::domain_computation::WorthQueryRunningDirectRun,
};

/// Private cleanup owner for the managed mutation run.
///
/// Provider-bound is the only posture that can complete a commit receipt. An
/// unbound owner remains necessary to close a run whose provider admission was
/// denied before a session existed.
pub enum WorthQueryApplicationMutationCleanupOwner {
    Unbound(WorthQueryMutationRunBinding),
    ProviderBound(WorthQueryProviderSessionBoundMutationRun),
}

impl WorthQueryApplicationMutationCleanupOwner {
    pub(super) fn finish(
        self,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        running: WorthQueryRunningDirectRun,
        terminal: WorthQueryManagedRunTerminalKind,
        snapshot_release: WorthQueryApplicationSnapshotRelease,
    ) -> Result<
        WorthQueryMutationGraphWorkCompletion,
        crate::domain_computation::WorthQueryMutationCleanupFailure,
    > {
        let on_worker = provider.managed_cleanup_control.take();
        match self {
            Self::Unbound(run) => run.finish(running, terminal, snapshot_release, on_worker),
            Self::ProviderBound(run) => run.finish(running, terminal, snapshot_release, on_worker),
        }
    }
}
