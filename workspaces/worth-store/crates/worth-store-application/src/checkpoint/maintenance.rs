//! One Home owns its automatic allowance, independent of the calling composition.
mod panic_boundary;
mod worker;
use super::{
    ApplicationHomeCheckpoint as Summary, ApplicationHomeCheckpointDenial as Denial,
    ApplicationHomeCheckpointLimits as Limits, ApplicationHomeCheckpointOperation,
};
use std::sync::{Arc, Mutex};
use worker::MaintenanceWorker;

#[derive(Debug)]
pub enum ApplicationHomeCheckpointMaintenanceDenial {
    Unavailable,
    Busy,
    WorkerCreation(std::io::ErrorKind),
    WorkerPanicked,
    DeadlineOverflow,
}
#[derive(Debug, Default)]
pub(super) struct CheckpointMaintenance {
    worker: Mutex<Option<Arc<MaintenanceWorker>>>,
}
impl CheckpointMaintenance {
    pub(super) fn install(&self, limits: Limits) -> Result<(), Denial> {
        let mut worker = self
            .worker
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if worker.is_some() {
            return Err(Denial::Maintenance(
                ApplicationHomeCheckpointMaintenanceDenial::Busy,
            ));
        }
        Self::initialize(&mut worker, limits)?;
        Ok(())
    }
    fn initialize(
        worker: &mut Option<Arc<MaintenanceWorker>>,
        limits: Limits,
    ) -> Result<Arc<MaintenanceWorker>, Denial> {
        if let Some(original) = worker {
            return Ok(original.clone());
        }
        // The worker's own root admits its Arc/control bill before allocation.
        let original = Arc::new(MaintenanceWorker::install(limits)?);
        *worker = Some(original.clone());
        Ok(original)
    }
    fn worker(&self, limits: Limits) -> Result<Arc<MaintenanceWorker>, Denial> {
        let mut worker = self
            .worker
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        Self::initialize(&mut worker, limits)
    }
    pub(super) fn is_active(&self) -> bool {
        self.worker
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .is_some_and(|worker| worker.is_active())
    }
    pub(super) fn capture(
        &self,
        operation: &ApplicationHomeCheckpointOperation,
        limits: Limits,
    ) -> Result<Summary, Denial> {
        // A bare Store Home uses finite defaults. An installed Host allowance
        // always wins; lazy close never replaces that same owner's worker.
        self.worker(limits)?.capture_for_close(operation)
    }
    pub(super) fn advance(
        &self,
        operation: &ApplicationHomeCheckpointOperation,
        limits: Limits,
    ) -> Result<Option<Summary>, Denial> {
        self.worker(limits)?.advance(operation)
    }
}
