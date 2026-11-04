//! Original subject and actual journal recovery live outside disposable calls.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use worth_foundational::facade::WorldDurableJournal;
use worth_runtime_world::facade::{
    WorldDurableArtifact, WorldDurableMediumDenial as Denial, WorldPublicationObservation,
};
use worth_store::physical_runtime::*;

#[derive(Default)]
pub(crate) struct WorldPublications {
    branches: Mutex<Option<Arc<WorldPublicationTask>>>,
    custody: Mutex<Option<Arc<WorldPublicationTask>>>,
    occurrences: Mutex<Option<Arc<WorldPublicationTask>>>,
}
impl WorldPublications {
    pub(crate) fn posture(&self) -> worth_runtime_world::facade::WorldPublicationTaskPosture {
        use worth_runtime_world::facade::WorldPublicationTaskPosture as Posture;
        for slot in [&self.branches, &self.custody, &self.occurrences] {
            let Ok(slot) = slot.try_lock() else {
                return Posture::Busy;
            };
            if let Some(task) = slot.as_ref() {
                let Ok(state) = task.state.try_lock() else {
                    return Posture::Busy;
                };
                if !matches!(
                    state.endpoint,
                    Some(Endpoint::Committed(_) | Endpoint::Terminal(_) | Endpoint::NoEffect)
                ) && (state.source.is_some() || state.recovery.is_some())
                {
                    return Posture::Unsettled;
                }
            }
        }
        Posture::Terminal
    }
    fn tasks(&self) -> Vec<Arc<WorldPublicationTask>> {
        [&self.branches, &self.custody, &self.occurrences]
            .into_iter()
            .filter_map(|slot| slot.lock().unwrap_or_else(|e| e.into_inner()).clone())
            .collect()
    }
    pub(super) fn settle_for_close(&self, serving: &ServingPhysicalRuntime) {
        for task in self.tasks() {
            let Some(_driver) = task.try_driver() else {
                continue;
            };
            task.resume();
            if task.observation().is_some() && !task.pending() {
                continue;
            }
            task.stop();
            let name = OrderedJournalName::new(task.subject.journal().name())
                .expect("closed World journal");
            if let Ok(journal) = serving.ordered_journal(name) {
                super::publication::advance(&task, &journal);
                let _ = task.observation();
            }
        }
    }
    pub(super) fn task(
        &self,
        subject: &WorldDurableArtifact,
    ) -> Result<Arc<WorldPublicationTask>, Denial> {
        let slot = match subject.journal() {
            WorldDurableJournal::Branches => &self.branches,
            WorldDurableJournal::Custody => &self.custody,
            WorldDurableJournal::Occurrences => &self.occurrences,
        };
        let mut slot = slot.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(task) = slot.as_ref() {
            if task.subject.same_subject(subject) {
                return Ok(task.clone());
            }
            if task.pending() {
                return Err(Denial::PublicationBusy);
            }
        }
        let task = Arc::new(WorldPublicationTask {
            subject: subject.clone(),
            state: Mutex::new(TaskState::default()),
            driver: AtomicBool::new(false),
        });
        let retired = slot.replace(task.clone());
        drop(slot);
        drop(retired);
        Ok(task)
    }
}
pub(super) struct WorldPublicationTask {
    pub(super) subject: WorldDurableArtifact,
    state: Mutex<TaskState>,
    driver: AtomicBool,
}
pub(super) struct WorldPublicationDriver<'a>(&'a WorldPublicationTask);
impl Drop for WorldPublicationDriver<'_> {
    fn drop(&mut self) {
        self.0.driver.store(false, Ordering::Release);
    }
}
#[derive(Default)]
struct TaskState {
    recovery: Option<OrderedJournalPublicationRecovery>,
    endpoint: Option<Endpoint>,
    source: Option<OrderedJournalPublicationSource>,
    invalid: bool,
}
pub(super) enum Endpoint {
    Ready(OrderedJournalPublicationOperation),
    Stopped(OrderedJournalStoppedPublication),
    Committed(OrderedJournalCommitted),
    Terminal(Arc<OrderedJournalPublicationTerminal>),
    NoEffect,
    Unresolved,
}
impl WorldPublicationTask {
    pub(super) fn try_driver(&self) -> Option<WorldPublicationDriver<'_>> {
        self.driver
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| WorldPublicationDriver(self))
    }
    pub(super) fn invalidate(&self) {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).invalid = true;
    }
    pub(super) fn pending(&self) -> bool {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        !matches!(
            s.endpoint,
            Some(Endpoint::Committed(_) | Endpoint::Terminal(_) | Endpoint::NoEffect)
        ) && (s.source.is_some() || s.recovery.is_some())
    }
    pub(super) fn started(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .source
            .is_some()
    }
    pub(super) fn install(
        &self,
        operation: OrderedJournalPublicationOperation,
        recovery: OrderedJournalPublicationRecovery,
    ) {
        let source = operation.observe().source();
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        s.source = Some(source);
        s.recovery = Some(recovery);
        s.endpoint = Some(Endpoint::Ready(operation));
    }
    pub(super) fn committed(&self, committed: OrderedJournalCommitted) {
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        s.invalid |= s.source.is_some_and(|source| source != committed.source());
        s.source = Some(committed.source());
        s.endpoint = Some(Endpoint::Committed(committed));
    }
    pub(super) fn take_endpoint(&self) -> Option<Endpoint> {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .endpoint
            .take()
    }
    pub(super) fn restore(&self, endpoint: Endpoint) {
        let retired = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .endpoint
            .replace(endpoint);
        drop(retired);
    }
    /// Consume recovery outside R. Reinstall the SAME returned retainer before
    /// the endpoint is lent to a disposable invoking context.
    pub(super) fn resume(&self) {
        let recovery = {
            let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if s.endpoint.is_some() && !matches!(s.endpoint, Some(Endpoint::Terminal(_))) {
                return;
            }
            s.recovery.take()
        };
        let Some(recovery) = recovery else {
            return;
        };
        match recovery.resume() {
            OrderedJournalPublicationResume::Committed(committed) => self.committed(committed),
            OrderedJournalPublicationResume::Ready {
                operation,
                recovery,
            } => self.install(operation, recovery),
            OrderedJournalPublicationResume::Stopped {
                operation,
                recovery,
            } => {
                let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
                s.recovery = Some(recovery);
                s.endpoint = Some(Endpoint::Stopped(operation));
            }
            OrderedJournalPublicationResume::Busy(recovery) => {
                self.state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .recovery = Some(recovery);
            }
            OrderedJournalPublicationResume::Terminal(observation) => self.settled(observation),
        }
    }
    pub(super) fn stop(&self) {
        let (recovery, endpoint) = {
            let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
            (s.recovery.take(), s.endpoint.take())
        };
        let recovery = recovery.map(OrderedJournalPublicationRecovery::stop);
        let endpoint = match endpoint {
            Some(Endpoint::Ready(operation)) => Some(Endpoint::Stopped(operation.stop())),
            other => other,
        };
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        s.recovery = recovery;
        s.endpoint = endpoint;
    }
    pub(super) fn settled(&self, observation: OrderedJournalPublicationObservation) {
        let expected = self.state.lock().unwrap_or_else(|e| e.into_inner()).source;
        let endpoint = if expected != Some(observation.source()) {
            Endpoint::Unresolved
        } else if let Some(terminal) = observation.terminal() {
            Endpoint::Terminal(terminal.clone())
        } else if observation.stopped() && observation.mutation_identity().is_none() {
            Endpoint::NoEffect
        } else {
            Endpoint::Unresolved
        };
        self.restore(endpoint);
    }
    pub(super) fn observation(&self) -> Option<WorldPublicationObservation> {
        // Temporarily move custody out so owner observation/settlement and
        // destructors never execute while the adapter state lock is held.
        let mut endpoint = self.take_endpoint()?;
        let observed = match &endpoint {
            Endpoint::Ready(operation) => operation.observe().terminal().cloned(),
            Endpoint::Stopped(operation) => operation.observe().terminal().cloned(),
            _ => None,
        };
        if let Some(terminal) = observed {
            let source = self.state.lock().unwrap_or_else(|e| e.into_inner()).source;
            if source != Some(terminal.source()) {
                self.invalidate();
            }
            endpoint = Endpoint::Terminal(terminal);
        }
        self.restore(endpoint);
        // This method is invoked outside disposable R. Consume the SAME
        // retained recovery to obtain the physical owner's sealed completion,
        // rather than promote its terminal description into a World witness.
        self.resume();
        let endpoint = self.take_endpoint()?;
        let invalid = self.state.lock().unwrap_or_else(|e| e.into_inner()).invalid;
        let result = if invalid {
            Some(WorldPublicationObservation::Indeterminate {
                detail: Denial::PublicationUnresolved,
            })
        } else {
            match &endpoint {
                Endpoint::Committed(committed) => {
                    Some(super::publication_fate::committed(committed))
                }
                Endpoint::Terminal(terminal) => Some(super::publication_fate::terminal(terminal)),
                Endpoint::NoEffect => Some(WorldPublicationObservation::ProvenNoEffect {
                    detail: Denial::Unavailable,
                }),
                Endpoint::Unresolved => Some(WorldPublicationObservation::Indeterminate {
                    detail: Denial::PublicationUnresolved,
                }),
                Endpoint::Ready(_) | Endpoint::Stopped(_) => None,
            }
        };
        self.restore(endpoint);
        result
    }
}
