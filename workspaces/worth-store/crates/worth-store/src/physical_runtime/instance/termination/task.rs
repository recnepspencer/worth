//! One accepted physical close, independent of how long its caller waits.
use std::sync::{mpsc, Arc, OnceLock};
use std::thread::JoinHandle;

use super::{PhysicalStoreCloseObservation, PhysicalStoreCloseOutcome};
use crate::physical_runtime::{RuntimeIdentity, ServingPhysicalRuntime};

mod custody;
mod outcome;
mod worker;

use custody::PhysicalCloseTaskCustody;
pub use outcome::{
    PhysicalStoreCloseFailure, PhysicalStoreCloseFailureCause, PhysicalStoreCloseStartRefused,
};

/// Readiness of the owned worker, never evidence that the Store closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalStoreCloseTaskReadiness {
    Running,
    WorkerFinished,
}

pub enum PhysicalStoreCloseTaskPoll {
    Pending(PhysicalStoreCloseTask),
    Completed(PhysicalStoreCloseOutcome),
    Failed(PhysicalStoreCloseFailure),
}

/// Caller custody of one Store-owned termination task.
///
/// Dropping this observer neither cancels nor joins the accepted close. The
/// worker retains the actual Store through its existing close algorithm. Process
/// termination or a worker panic supplies no successful-close evidence.
#[must_use]
pub struct PhysicalStoreCloseTask {
    custody: PhysicalCloseTaskCustody,
    receiver: mpsc::Receiver<PhysicalStoreCloseOutcome>,
    worker: JoinHandle<()>,
    received: Option<PhysicalStoreCloseOutcome>,
    observation: Arc<OnceLock<PhysicalStoreCloseObservation>>,
}

impl PhysicalStoreCloseTask {
    /// Accept the real serving owner, or return it unstarted on spawn refusal.
    /// Allocation and OS thread creation are cooperative substrate safe points;
    /// this operation provides no hard deadline bound.
    pub fn start(serving: ServingPhysicalRuntime) -> Result<Self, PhysicalStoreCloseStartRefused> {
        Self::start_observing(serving, worker::PlanObserver::None)
    }

    /// Forward the existing control to the actual worker-owned plan.
    #[cfg(feature = "certification-test-authority")]
    pub fn start_with_plan_observer_for_test(
        serving: ServingPhysicalRuntime,
        observe: impl FnOnce(&super::PhysicalStoreClosePlan) + Send + 'static,
    ) -> Result<Self, PhysicalStoreCloseStartRefused> {
        Self::start_observing(serving, worker::PlanObserver::Observe(Box::new(observe)))
    }

    fn start_observing(
        serving: ServingPhysicalRuntime,
        observer: worker::PlanObserver,
    ) -> Result<Self, PhysicalStoreCloseStartRefused> {
        let identity = serving.runtime_identity();
        let observation = Arc::new(OnceLock::new());
        let (sender, receiver) = mpsc::channel();
        let worker =
            worker::spawn_close_worker(serving, sender, Arc::clone(&observation), observer)?;
        Ok(Self {
            custody: PhysicalCloseTaskCustody::accepted(identity),
            receiver,
            worker,
            received: None,
            observation,
        })
    }

    pub fn runtime_identity(&self) -> RuntimeIdentity {
        self.custody.identity()
    }

    pub fn readiness(&self) -> PhysicalStoreCloseTaskReadiness {
        if self.worker.is_finished() {
            PhysicalStoreCloseTaskReadiness::WorkerFinished
        } else {
            PhysicalStoreCloseTaskReadiness::Running
        }
    }

    /// None means the real plan has not published its observer yet. Scrub
    /// drainage may already be underway; None never means no effects occurred.
    pub fn observation(&self) -> Option<&PhysicalStoreCloseObservation> {
        self.observation.get()
    }

    /// Consume caller custody and return the same task while it remains pending.
    /// Check the original request only after Pending, so observed terminal facts
    /// win over a late cancellation/deadline. A finished-worker join is expected
    /// to be quick, but is not a strict nonblocking or hard deadline guarantee.
    pub fn poll(mut self) -> PhysicalStoreCloseTaskPoll {
        if self.received.is_none() {
            self.received = self.receiver.try_recv().ok();
        }
        if !self.worker.is_finished() {
            return PhysicalStoreCloseTaskPoll::Pending(self);
        }
        let Self {
            custody,
            receiver,
            worker,
            mut received,
            observation,
        } = self;
        let joined = worker.join();
        // The worker may have sent between the first receive and is_finished.
        if received.is_none() {
            received = receiver.try_recv().ok();
        }
        match outcome::classify_delivery(joined, received) {
            Ok(actual) => {
                custody.outcome_delivered();
                PhysicalStoreCloseTaskPoll::Completed(actual)
            }
            Err((cause, actual)) => PhysicalStoreCloseTaskPoll::Failed(
                PhysicalStoreCloseFailure::new(custody, cause, actual, observation.get().cloned()),
            ),
        }
    }
}
