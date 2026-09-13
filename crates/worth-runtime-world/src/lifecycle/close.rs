use crate::history::{
    CompositeHistoryReclamationRequest, HistoryReclamationDenial, HistoryReclamationOutcome,
};
#[path = "close/drain.rs"]
mod drain;
#[path = "close/report.rs"]
mod report;

pub use report::{RuntimeWorldCloseReport, RuntimeWorldRetainedRecordReport};

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
    AlreadyClosing,
    AlreadyClosed,
    /// A declared in-flight critical section could not be drained.
    InFlightCriticalSection,
}

use super::owner::RuntimeWorldOwnerRoot;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Close the owner and report what the drain settled, released, and
    /// deliberately exposed. A retained obligation is never discarded.
    ///
    /// Bootstrap and operation admission remain locked through the ledger
    /// check and Open -> Closing transition. Closing excludes new work, so
    /// the drain releases component custody outside lifecycle locks.
    pub fn close(&self) -> Result<RuntimeWorldCloseReport, RuntimeWorldCloseDenial> {
        drain::close_owner(&self.state)
    }

    /// How many callers are queued inside close admission right now. Close
    /// blocks on the operation ledger before it decides, so a world that will
    /// not close is distinguished from a world nobody is closing by this count
    /// rather than by waiting.
    #[cfg(test)]
    pub fn close_admission_waiters(&self) -> usize {
        self.state
            .close_admission_waiters
            .load(std::sync::atomic::Ordering::SeqCst)
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

    fn close(&self) -> Result<RuntimeWorldCloseReport, RuntimeWorldCloseDenial> {
        RuntimeWorldOwnerRoot::close(self)
    }
    fn reclaim_history(
        &self,
        request: CompositeHistoryReclamationRequest,
    ) -> Result<HistoryReclamationOutcome, HistoryReclamationDenial> {
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| {
                HistoryReclamationDenial::OwnerUnavailable(
                    super::RuntimeWorldOwnerUnavailable::new(),
                )
            })?;
        self.state.history.reclaim_batch(request)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeWorldCloseContract {
    state: RuntimeWorldCloseState,
}

impl RuntimeWorldCloseContract {
    pub(crate) const fn open() -> Self {
        Self {
            state: RuntimeWorldCloseState::Open,
        }
    }

    pub(crate) const fn state(self) -> RuntimeWorldCloseState {
        self.state
    }

    pub(crate) fn begin(&mut self) -> Result<(), RuntimeWorldCloseDenial> {
        match self.state {
            RuntimeWorldCloseState::Open => {
                self.state = RuntimeWorldCloseState::Closing;
                Ok(())
            }
            RuntimeWorldCloseState::Closing => Err(RuntimeWorldCloseDenial::AlreadyClosing),
            RuntimeWorldCloseState::Closed => Err(RuntimeWorldCloseDenial::AlreadyClosed),
        }
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
