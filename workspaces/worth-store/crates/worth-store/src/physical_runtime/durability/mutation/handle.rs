use std::cell::Cell;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};

use super::{
    PhysicalMutationCancellationOutcome, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdentity, PhysicalMutationOutcome,
    PhysicalMutationPoll, PhysicalMutationProgress, PhysicalMutationProgressPhase,
    PhysicalMutationRequestFingerprint, PhysicalMutationTerminalFact,
};

pub struct PhysicalMutationHandle {
    attempt: Arc<PhysicalMutationAttempt>,
    observed_terminal: Cell<bool>,
}

pub(in crate::physical_runtime) struct PhysicalMutationAttempt {
    identity: PhysicalMutationIdentity,
    idempotency: PhysicalMutationIdempotencyKeyIdentity,
    fingerprint: PhysicalMutationRequestFingerprint,
    deadline: PhysicalMutationDeadline,
    owner: Weak<crate::physical_runtime::durability::lifecycle::PhysicalMutationRuntimeOwner>,
    effect_cutover: Mutex<()>,
    state: Mutex<PhysicalMutationAttemptState>,
    changed: Condvar,
}

struct PhysicalMutationAttemptState {
    progress: PhysicalMutationProgress,
    settlement_committed: bool,
    stale: bool,
    terminal: Option<PhysicalMutationTerminalFact>,
    terminal_visible: bool,
    observers: u32,
    terminal_observed: bool,
    completed_unobserved_recorded: bool,
}

impl PhysicalMutationHandle {
    pub(in crate::physical_runtime) fn new(attempt: &Arc<PhysicalMutationAttempt>) -> Self {
        attempt.add_observer();
        Self {
            attempt: Arc::clone(attempt),
            observed_terminal: Cell::new(false),
        }
    }

    pub fn identity(&self) -> PhysicalMutationIdentity {
        self.attempt.identity
    }

    pub fn idempotency_identity(&self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.attempt.idempotency
    }

    pub fn request_fingerprint(&self) -> PhysicalMutationRequestFingerprint {
        self.attempt.fingerprint
    }

    pub fn deadline(&self) -> PhysicalMutationDeadline {
        self.attempt.deadline
    }

    pub fn progress(&self) -> PhysicalMutationProgress {
        self.attempt.progress()
    }

    pub fn poll(&self) -> PhysicalMutationPoll {
        let poll = self.attempt.poll();
        if matches!(poll, PhysicalMutationPoll::Terminal(_)) {
            self.observed_terminal.set(true);
        }
        poll
    }

    pub fn request_cancellation(&self) -> PhysicalMutationCancellationOutcome {
        let outcome = self.attempt.request_cancellation();
        if matches!(
            outcome,
            PhysicalMutationCancellationOutcome::AlreadyTerminal(_)
        ) {
            self.observed_terminal.set(true);
        }
        outcome
    }

    pub fn wait(self) -> PhysicalMutationOutcome {
        let terminal = self.attempt.wait();
        self.observed_terminal.set(true);
        terminal
    }
}

impl Drop for PhysicalMutationHandle {
    fn drop(&mut self) {
        self.attempt.release_observer(self.observed_terminal.get());
    }
}

impl PhysicalMutationAttempt {
    pub(in crate::physical_runtime) fn new(
        prepared: &crate::physical_runtime::PreparedPhysicalMutation,
        owner: Weak<crate::physical_runtime::durability::lifecycle::PhysicalMutationRuntimeOwner>,
    ) -> Arc<Self> {
        Arc::new(Self {
            identity: prepared.mutation_identity(),
            idempotency: prepared.idempotency_identity(),
            fingerprint: prepared.request_fingerprint(),
            deadline: prepared.deadline(),
            owner,
            effect_cutover: Mutex::new(()),
            state: Mutex::new(PhysicalMutationAttemptState {
                progress: PhysicalMutationProgress::admitted(prepared.mutation_identity()),
                settlement_committed: false,
                stale: false,
                terminal: None,
                terminal_visible: false,
                observers: 0,
                terminal_observed: false,
                completed_unobserved_recorded: false,
            }),
            changed: Condvar::new(),
        })
    }

    pub(in crate::physical_runtime) const fn identity(&self) -> PhysicalMutationIdentity {
        self.identity
    }

    pub(in crate::physical_runtime) const fn idempotency_identity(
        &self,
    ) -> PhysicalMutationIdempotencyKeyIdentity {
        self.idempotency
    }

    pub(in crate::physical_runtime) const fn fingerprint(
        &self,
    ) -> PhysicalMutationRequestFingerprint {
        self.fingerprint
    }

    pub(in crate::physical_runtime) const fn deadline(&self) -> PhysicalMutationDeadline {
        self.deadline
    }

    pub(in crate::physical_runtime) fn enter(&self, phase: PhysicalMutationProgressPhase) {
        self.state().progress.enter(phase);
    }

    pub(in crate::physical_runtime) fn effect_cutover(&self) -> MutexGuard<'_, ()> {
        self.effect_cutover
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::physical_runtime) fn cancellation_requested(&self) -> bool {
        self.state().progress.cancellation_requested()
    }

    pub(in crate::physical_runtime) fn commit_settlement(&self) {
        self.state().settlement_committed = true;
    }

    pub(in crate::physical_runtime) fn mark_runtime_closing(&self) {
        let _cutover = self.effect_cutover();
        let mut state = self.state();
        state.progress.mark_runtime_closing();
        if !state.settlement_committed && state.terminal.is_none() {
            state.progress.request_cancellation();
        }
    }

    pub(in crate::physical_runtime) fn mark_stale(&self) {
        self.state().stale = true;
    }

    pub(in crate::physical_runtime) fn install_terminal(
        &self,
        terminal: PhysicalMutationTerminalFact,
    ) {
        let mut state = self.state();
        state.terminal = Some(terminal);
    }

    pub(in crate::physical_runtime) fn publish_terminal(&self) {
        let mut state = self.state();
        state
            .progress
            .enter(PhysicalMutationProgressPhase::Terminal);
        state.terminal_visible = true;
        self.changed.notify_all();
    }

    pub(in crate::physical_runtime) fn record_completed_unobserved_on_completion(
        &self,
    ) -> Option<crate::physical_runtime::CompletedUnobservedPhysicalMutation> {
        let mut state = self.state();
        if state.observers == 0 && !state.terminal_observed && !state.completed_unobserved_recorded
        {
            state.completed_unobserved_recorded = true;
            completed_unobserved_event(&state, self.identity)
        } else {
            None
        }
    }

    fn add_observer(&self) {
        let mut state = self.state();
        state.observers = state.observers.saturating_add(1);
    }

    fn release_observer(&self, observed_terminal: bool) {
        let completed_unobserved = {
            let mut state = self.state();
            state.terminal_observed |= observed_terminal;
            state.observers = state.observers.saturating_sub(1);
            let completed = state.observers == 0
                && !state.terminal_observed
                && !state.completed_unobserved_recorded
                && matches!(
                    state.terminal,
                    Some(PhysicalMutationTerminalFact::Completed(_))
                )
                && {
                    state.completed_unobserved_recorded = true;
                    true
                };
            completed
                .then(|| completed_unobserved_event(&state, self.identity))
                .flatten()
        };
        if let Some(event) = completed_unobserved
            && let Some(owner) = self.owner.upgrade()
        {
            owner.record_completed_unobserved(event);
        }
    }

    fn progress(&self) -> PhysicalMutationProgress {
        self.state().progress
    }

    fn poll(&self) -> PhysicalMutationPoll {
        let state = self.state();
        if state.terminal_visible {
            PhysicalMutationPoll::Terminal(
                state
                    .terminal
                    .as_ref()
                    .expect("visible terminal state retains its exact fact")
                    .observation(),
            )
        } else {
            PhysicalMutationPoll::Pending(state.progress)
        }
    }

    fn request_cancellation(&self) -> PhysicalMutationCancellationOutcome {
        let _cutover = self.effect_cutover();
        let mut state = self.state();
        let (outcome, class) = if state.stale {
            (
                PhysicalMutationCancellationOutcome::StaleHandle {
                    identity: self.identity,
                },
                crate::physical_runtime::PhysicalMutationCancellationClass::Stale,
            )
        } else if state.progress.runtime_closing() {
            (
                PhysicalMutationCancellationOutcome::RuntimeClosing {
                    identity: self.identity,
                },
                crate::physical_runtime::PhysicalMutationCancellationClass::RuntimeClosing,
            )
        } else if state.terminal_visible {
            let terminal = state
                .terminal
                .as_ref()
                .expect("visible terminal state retains its exact fact");
            (
                PhysicalMutationCancellationOutcome::AlreadyTerminal(terminal.observation()),
                crate::physical_runtime::PhysicalMutationCancellationClass::Terminal,
            )
        } else if state.settlement_committed {
            (
                PhysicalMutationCancellationOutcome::SettlementAlreadyEffectful {
                    identity: self.identity,
                    phase: state.progress.phase(),
                },
                crate::physical_runtime::PhysicalMutationCancellationClass::Effectful,
            )
        } else {
            state.progress.request_cancellation();
            (
                PhysicalMutationCancellationOutcome::AcceptedBeforeEffect {
                    identity: self.identity,
                },
                crate::physical_runtime::PhysicalMutationCancellationClass::Accepted,
            )
        };
        drop(state);
        if let Some(owner) = self.owner.upgrade() {
            owner.record_cancellation(class);
        }
        outcome
    }

    fn wait(&self) -> PhysicalMutationOutcome {
        let mut state = self.state();
        loop {
            if state.terminal_visible {
                let terminal = state
                    .terminal
                    .as_ref()
                    .expect("visible terminal state retains its exact fact");
                return terminal.outcome();
            }
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn state(&self) -> MutexGuard<'_, PhysicalMutationAttemptState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn completed_unobserved_event(
    state: &PhysicalMutationAttemptState,
    identity: PhysicalMutationIdentity,
) -> Option<crate::physical_runtime::CompletedUnobservedPhysicalMutation> {
    match state.terminal.as_ref() {
        Some(PhysicalMutationTerminalFact::Completed(fact)) => Some(
            crate::physical_runtime::CompletedUnobservedPhysicalMutation::new(
                identity,
                fact.breadth(),
            ),
        ),
        _ => None,
    }
}
