//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryMutationRunBinding {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        running: crate::domain_computation::WorthQueryRunningDirectRun,
        terminal: crate::domain_computation::WorthQueryManagedRunTerminalKind,
        snapshot_release: WorthQueryApplicationSnapshotRelease,
        on_worker: bool,
    ) -> Result<WorthQueryMutationGraphWorkCompletion, WorthQueryMutationCleanupFailure> {
        Self::finish(self, running, terminal, snapshot_release, on_worker)
    }
}

impl WorthQueryProviderSessionBoundMutationRun {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        running: crate::domain_computation::WorthQueryRunningDirectRun,
        terminal: crate::domain_computation::WorthQueryManagedRunTerminalKind,
        snapshot_release: WorthQueryApplicationSnapshotRelease,
        on_worker: bool,
    ) -> Result<WorthQueryMutationGraphWorkCompletion, WorthQueryMutationCleanupFailure> {
        Self::finish(self, running, terminal, snapshot_release, on_worker)
    }
}
