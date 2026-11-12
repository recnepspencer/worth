//! Original subject and actual journal recovery live outside disposable calls.
use super::subject::SignalPublicationSubject;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use worth_foundational::facade::SignalDurableJournal;
use worth_signal::facade::durability::SignalPublicationCompletion;
use worth_signal::facade::durability::{
    SignalDurableMediumDenial as Denial, SignalPublicationObservation,
};
use worth_store::physical_runtime::*;

#[derive(Default)]
pub(crate) struct SignalPublications {
    definitions: Mutex<Option<Arc<SignalPublicationTask>>>,
    advances: Mutex<Option<Arc<SignalPublicationTask>>>,
}
impl SignalPublications {
    pub(crate) fn posture(&self) -> worth_signal::facade::durability::SignalPublicationTaskPosture {
        use worth_signal::facade::durability::SignalPublicationTaskPosture as Posture;
        for slot in [&self.definitions, &self.advances] {
            let Ok(slot) = slot.try_lock() else {
                return Posture::Busy;
            };
            if let Some(task) = slot.as_ref() {
                if task.driver.load(Ordering::Acquire) {
                    return Posture::Busy;
                }
                let Ok(state) = task.state.try_lock() else {
                    return Posture::Busy;
                };
                if state.invalid || matches!(state.endpoint, Some(Endpoint::Unresolved)) {
                    return Posture::Unsettled;
                }
                if !matches!(
                    state.endpoint,
                    Some(Endpoint::Committed(_) | Endpoint::NoEffect)
                ) {
                    return Posture::Unsettled;
                }
            }
        }
        Posture::Terminal
    }
    pub(super) fn task(
        &self,
        subject: &SignalPublicationSubject,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<SignalPublicationTask>, Denial> {
        let slot = match subject.journal() {
            SignalDurableJournal::Definitions => &self.definitions,
            SignalDurableJournal::Advances => &self.advances,
        };
        let mut slot = slot.try_lock().map_err(|_| Denial::PublicationBusy)?;
        if let Some(task) = slot.as_ref() {
            if task.subject.same_subject(subject) {
                return Ok(task.clone());
            }
            if task.unresolved() {
                return Err(Denial::PublicationUnresolved);
            }
            if task.pending() {
                return Err(Denial::PublicationBusy);
            }
        }
        let funding = execution
            .reserve_retained_memory(
                (std::mem::size_of::<SignalPublicationTask>() + 2 * std::mem::size_of::<usize>())
                    as u64,
            )
            .map_err(Denial::Request)?;
        let task = Arc::new(SignalPublicationTask {
            _funding: funding,
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
pub(super) struct SignalPublicationTask {
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
    pub(super) subject: SignalPublicationSubject,
    state: Mutex<TaskState>,
    driver: AtomicBool,
}
pub(super) struct SignalPublicationDriver<'a>(&'a SignalPublicationTask);
impl Drop for SignalPublicationDriver<'_> {
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
impl SignalPublicationDriver<'_> {
    /// Called only with the original exclusive driver and an admitted exact
    /// retry proof. Retire physical custody only after this task itself proved
    /// no effect; terminal observation on the ordinary advance path is stable.
    pub(super) fn retry_no_effect(&self) {
        let retired = {
            let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.invalid {
                return;
            }
            match state.endpoint.as_ref() {
                Some(Endpoint::NoEffect) => Some(std::mem::take(&mut *state)),
                Some(
                    Endpoint::Ready(_)
                    | Endpoint::Stopped(_)
                    | Endpoint::Committed(_)
                    | Endpoint::Terminal(_)
                    | Endpoint::Unresolved,
                )
                | None => None,
            }
        };
        drop(retired);
    }
}
impl SignalPublicationTask {
    fn unresolved(&self) -> bool {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.invalid || matches!(state.endpoint, Some(Endpoint::Unresolved))
    }
    pub(super) fn try_driver(&self) -> Option<SignalPublicationDriver<'_>> {
        self.driver
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| SignalPublicationDriver(self))
    }
    pub(super) fn invalidate(&self) {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).invalid = true;
    }
    pub(super) fn pending(&self) -> bool {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        self.driver.load(Ordering::Acquire)
            || !matches!(
                state.endpoint,
                Some(Endpoint::Committed(_) | Endpoint::Unresolved | Endpoint::NoEffect)
            )
    }
    pub(super) fn remember_refusal(
        &self,
        observation: SignalPublicationObservation,
    ) -> SignalPublicationObservation {
        use worth_foundational::facade::DurablePublicationFate;
        let endpoint = match &observation {
            SignalPublicationObservation::Pending => return observation,
            SignalPublicationObservation::Finished(completion) => match completion.fate() {
                DurablePublicationFate::Committed => return observation,
                DurablePublicationFate::ProvenNoEffect { .. } => Endpoint::NoEffect,
                DurablePublicationFate::Indeterminate { .. } => Endpoint::Unresolved,
            },
        };
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            state.source.is_none(),
            "start refusal cannot erase submitted custody"
        );
        state.endpoint = Some(endpoint);
        observation
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
            same @ (Some(
                Endpoint::Stopped(_)
                | Endpoint::Committed(_)
                | Endpoint::Terminal(_)
                | Endpoint::NoEffect
                | Endpoint::Unresolved,
            )
            | None) => same,
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
    pub(super) fn observation(&self) -> Option<SignalPublicationObservation> {
        // Temporarily move custody out so owner observation/settlement and
        // destructors never execute while the adapter state lock is held.
        let mut endpoint = self.take_endpoint()?;
        let observed = match &endpoint {
            Endpoint::Ready(operation) => operation.observe().terminal().cloned(),
            Endpoint::Stopped(operation) => operation.observe().terminal().cloned(),
            Endpoint::Committed(_)
            | Endpoint::Terminal(_)
            | Endpoint::NoEffect
            | Endpoint::Unresolved => None,
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
        // rather than promote its terminal description into a Signal witness.
        self.resume();
        let endpoint = self.take_endpoint()?;
        let invalid = self.state.lock().unwrap_or_else(|e| e.into_inner()).invalid;
        let result = if invalid && matches!(endpoint, Endpoint::Ready(_) | Endpoint::Stopped(_)) {
            None
        } else if invalid {
            Some(SignalPublicationObservation::Finished(
                SignalPublicationCompletion::indeterminate(Denial::PublicationUnresolved),
            ))
        } else {
            match &endpoint {
                Endpoint::Committed(committed) => {
                    Some(super::publication_fate::committed(committed))
                }
                Endpoint::Terminal(terminal) => Some(super::publication_fate::terminal(terminal)),
                Endpoint::NoEffect => Some(SignalPublicationObservation::Finished(
                    SignalPublicationCompletion::proven_no_effect(Denial::Unavailable),
                )),
                Endpoint::Unresolved => Some(SignalPublicationObservation::Finished(
                    SignalPublicationCompletion::indeterminate(Denial::PublicationUnresolved),
                )),
                Endpoint::Ready(_) | Endpoint::Stopped(_) => None,
            }
        };
        let endpoint = match &result {
            Some(SignalPublicationObservation::Finished(completion)) => match completion.fate() {
                worth_foundational::facade::DurablePublicationFate::Committed => endpoint,
                worth_foundational::facade::DurablePublicationFate::ProvenNoEffect { .. } => {
                    Endpoint::NoEffect
                }
                worth_foundational::facade::DurablePublicationFate::Indeterminate { .. } => {
                    Endpoint::Unresolved
                }
            },
            Some(SignalPublicationObservation::Pending) | None => endpoint,
        };
        self.restore(endpoint);
        result
    }
}
