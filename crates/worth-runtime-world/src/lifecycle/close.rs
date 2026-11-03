use crate::history::HistoryReclamationDenial;
mod admission;
#[path = "close/drain.rs"]
mod drain;
pub use admission::RuntimeWorldPreCloseAdmission;
#[path = "close/report.rs"]
mod report;
mod result_snapshots;
pub use result_snapshots::{
    RuntimeWorldResultSnapshotCleanup, RuntimeWorldResultSnapshotCleanupDenial,
    RuntimeWorldResultSnapshotCleanupProgress, RuntimeWorldResultSnapshotCleanupReport,
};

pub use report::{
    RuntimeWorldCloseReport, RuntimeWorldRecoveredCustodyReport, RuntimeWorldRetainedRecordReport,
};

/// Internal close progression for a managed Runtime World owner. The state
/// is explicit so close cannot be represented by a boolean with no transition
/// semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeWorldCloseState {
    Open,
    Closing,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldCloseDenial {
    CloseNotCompleted,
    OwnerUnavailable,
    Busy,
    Stopped(worth_execution::MapKernelStop),
    Execution(worth_execution::WorkCeilingDenial),
    PublicationFrontierChanged,
    UnresolvedBusinessCustody,
    CleanupRequired,
    CloseProgressPoisoned,
    CloseProgressInvariant,
    UndeliveredPublication,
    AlreadyClosing,
    AlreadyClosed,
    /// A declared in-flight critical section could not be drained.
    InFlightCriticalSection,
    /// The Home retains an actual accepted journal job that can still settle.
    DurablePublicationUnsettled,
}

use super::owner::RuntimeWorldOwnerRoot;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Close under the caller's original request. Busy or stopped admission
    /// leaves Open; after sealing, the SAME retained task resumes in Closing.
    /// Unresolved live business custody refuses before sealing. Unclaimed
    /// recovered durable obligations remain journal-backed and are reported.
    /// Head retirement
    /// runs outside admission locks and retains each exact item on refusal.
    pub fn close(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RuntimeWorldCloseReport, RuntimeWorldCloseDenial> {
        drain::close_owner(&self.state, request)
    }

    pub fn lifecycle_observation(&self) -> super::RuntimeWorldOwnerLifecycleObservation {
        match self
            .state
            .close
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .state()
        {
            RuntimeWorldCloseState::Open => super::RuntimeWorldOwnerLifecycleObservation::Open,
            RuntimeWorldCloseState::Closing => {
                super::RuntimeWorldOwnerLifecycleObservation::Closing
            }
            RuntimeWorldCloseState::Closed => super::RuntimeWorldOwnerLifecycleObservation::Closed,
        }
    }

    fn require_optional_memory_history(&self) -> Result<(), HistoryReclamationDenial> {
        match &self.state.durable {
            Some(_) => Err(HistoryReclamationDenial::AuthoritativeDurableHistory),
            None => Ok(()),
        }
    }
}

impl<D, I, E, Ctx, T> super::ports::RuntimeWorldLifecycleService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn bootstrap_root(
        &self,
        intent: crate::branch::RuntimeWorldBootstrapIntent,
    ) -> crate::branch::RuntimeWorldBootstrapOutcome {
        RuntimeWorldOwnerRoot::bootstrap_root(self, intent)
    }

    fn close(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RuntimeWorldCloseReport, RuntimeWorldCloseDenial> {
        RuntimeWorldOwnerRoot::close(self, request)
    }
    fn retire_changed_history(
        &self,
    ) -> Result<Vec<crate::identity::CompositeCommitIdentity>, HistoryReclamationDenial> {
        self.require_optional_memory_history()?;
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| {
                HistoryReclamationDenial::OwnerUnavailable(
                    super::RuntimeWorldOwnerUnavailable::new(),
                )
            })?;
        let retired = self.state.history.retire_changed_history();
        Ok(self.reclaim_retired_entries(retired))
    }
    fn retire_unprotected_history(
        &self,
        keep: &crate::identity::CompositeCommitIdentity,
    ) -> Result<Vec<crate::identity::CompositeCommitIdentity>, HistoryReclamationDenial> {
        self.require_optional_memory_history()?;
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| {
                HistoryReclamationDenial::OwnerUnavailable(
                    super::RuntimeWorldOwnerUnavailable::new(),
                )
            })?;
        let retired = self
            .state
            .history
            .retire_unprotected_history(keep)
            .map_err(HistoryReclamationDenial::Catalog)?;
        Ok(self.reclaim_retired_entries(retired))
    }
    fn release_retired_branch_history(
        &self,
        from: &crate::identity::CompositeCommitIdentity,
    ) -> Result<crate::history::RetiredBranchHistoryRelease, HistoryReclamationDenial> {
        self.require_optional_memory_history()?;
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| {
                HistoryReclamationDenial::OwnerUnavailable(
                    super::RuntimeWorldOwnerUnavailable::new(),
                )
            })?;
        let (released, held) = self
            .state
            .history
            .release_retired_branch_history(from)
            .map_err(HistoryReclamationDenial::Catalog)?;
        Ok(crate::history::RetiredBranchHistoryRelease::new(
            self.reclaim_retired_entries(released),
            held,
        ))
    }
    fn retained_commits(
        &self,
        candidates: &[crate::identity::CompositeCommitIdentity],
    ) -> Vec<crate::identity::CompositeCommitIdentity> {
        self.state.history.retained_commits(candidates)
    }
    fn reclaim_retention(
        &self,
        keys: &[crate::inspection::RuntimeWorldRetentionKey],
        maximum: usize,
    ) -> Result<
        crate::retention::RetentionReclamationReport,
        crate::inspection::RuntimeWorldRetentionInspectionDenial,
    > {
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| {
                crate::inspection::RuntimeWorldRetentionInspectionDenial::OwnerUnavailable(
                    super::RuntimeWorldOwnerUnavailable::new(),
                )
            })?;
        self.state.retention.reclaim_keys(keys, maximum)
    }
}

#[derive(Debug)]
pub(crate) struct RuntimeWorldCloseContract {
    state: RuntimeWorldCloseState,
    progress: Option<std::sync::Arc<std::sync::Mutex<drain::CloseProgress>>>,
}

impl RuntimeWorldCloseContract {
    pub(crate) const fn open() -> Self {
        Self {
            state: RuntimeWorldCloseState::Open,
            progress: None,
        }
    }

    pub(crate) const fn state(&self) -> RuntimeWorldCloseState {
        self.state
    }

    fn begin_admitted(&mut self, task: std::sync::Arc<std::sync::Mutex<drain::CloseProgress>>) {
        // The private constructor retains the Open close guard until here.
        self.state = RuntimeWorldCloseState::Closing;
        self.progress = Some(task);
    }

    pub(crate) fn finish(&mut self) -> Result<(), RuntimeWorldCloseDenial> {
        match self.state {
            RuntimeWorldCloseState::Closing => {
                self.state = RuntimeWorldCloseState::Closed;
                Ok(())
            }
            RuntimeWorldCloseState::Open => Err(RuntimeWorldCloseDenial::AlreadyClosing),
            RuntimeWorldCloseState::Closed => Err(RuntimeWorldCloseDenial::AlreadyClosed),
        }
    }
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Drops removed history entries and reclaims the pins only they held.
    /// Returns the removed commits in the order given.
    fn reclaim_retired_entries(
        &self,
        retired: crate::history::RetiredHistoryEntries,
    ) -> Vec<crate::identity::CompositeCommitIdentity> {
        let keys: Vec<_> = retired
            .iter()
            .filter_map(|slot| slot.get())
            .flat_map(|entry| entry.retention_keys())
            .collect();
        let identities = retired
            .iter()
            .filter_map(|slot| slot.get())
            .map(|entry| entry.commit().identity().clone())
            .collect();
        // Dropping the entries releases their pins; a pin no other holder
        // shares is then reclaimed from the registry.
        drop(retired);
        self.state
            .retention
            .reclaim_keys(&keys, keys.len())
            .expect("retired history pins belong to this owner");
        identities
    }
}
