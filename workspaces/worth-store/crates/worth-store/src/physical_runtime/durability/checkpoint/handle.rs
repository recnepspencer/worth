use std::sync::{Arc, Condvar, Mutex};

use super::{
    PhysicalCheckpointCancellationOutcome, PhysicalCheckpointDeadline, PhysicalCheckpointDisposal,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointPoll,
    PhysicalCheckpointProgress, PhysicalCheckpointProgressPhase,
};
use worth_store_physical_format::{PhysicalCheckpointIdentity, PhysicalCheckpointSource};

pub struct PhysicalCheckpointHandle {
    attempt: Arc<PhysicalCheckpointAttempt>,
}

pub(in crate::physical_runtime) struct PhysicalCheckpointAttempt {
    idempotency: PhysicalCheckpointIdempotencyKey,
    deadline: PhysicalCheckpointDeadline,
    state: Mutex<PhysicalCheckpointAttemptState>,
    changed: Condvar,
}

struct PhysicalCheckpointAttemptState {
    progress: PhysicalCheckpointProgress,
    publication_started: bool,
    runtime_closing: bool,
    terminal: Option<PhysicalCheckpointOutcome>,
    interruption: Option<Arc<worth_execution::ExecutionInterruptionView>>,
}

pub(super) struct PhysicalCheckpointCaptureAllocation<'attempt> {
    attempt: &'attempt PhysicalCheckpointAttempt,
}

impl PhysicalCheckpointHandle {
    pub(super) fn new(attempt: &Arc<PhysicalCheckpointAttempt>) -> Self {
        Self {
            attempt: Arc::clone(attempt),
        }
    }

    pub fn identity(&self) -> PhysicalCheckpointIdentity {
        self.attempt.identity()
    }

    pub fn source(&self) -> PhysicalCheckpointSource {
        self.attempt.source()
    }

    pub fn deadline(&self) -> PhysicalCheckpointDeadline {
        self.attempt.deadline()
    }

    pub fn progress(&self) -> PhysicalCheckpointProgress {
        self.attempt.progress()
    }

    pub fn poll(&self) -> PhysicalCheckpointPoll {
        self.attempt.poll()
    }

    pub fn request_cancellation(&self) -> PhysicalCheckpointCancellationOutcome {
        self.attempt.request_cancellation()
    }

    pub fn wait(self) -> PhysicalCheckpointOutcome {
        self.attempt.wait()
    }

    pub fn dispose(self) -> PhysicalCheckpointDisposal {
        self.attempt.dispose()
    }
}

impl PhysicalCheckpointAttempt {
    pub(super) fn new(
        idempotency: PhysicalCheckpointIdempotencyKey,
        deadline: PhysicalCheckpointDeadline,
        source: PhysicalCheckpointSource,
        interruption: Option<Arc<worth_execution::ExecutionInterruptionView>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            idempotency,
            deadline,
            state: Mutex::new(PhysicalCheckpointAttemptState {
                progress: PhysicalCheckpointProgress::admitted(source),
                publication_started: false,
                runtime_closing: false,
                terminal: None,
                interruption,
            }),
            changed: Condvar::new(),
        })
    }

    pub(super) const fn idempotency_key(&self) -> PhysicalCheckpointIdempotencyKey {
        self.idempotency
    }

    pub(super) const fn deadline(&self) -> PhysicalCheckpointDeadline {
        self.deadline
    }

    pub(super) fn identity(&self) -> PhysicalCheckpointIdentity {
        self.state().progress.identity()
    }

    pub(super) fn source(&self) -> PhysicalCheckpointSource {
        self.state().progress.source()
    }

    pub(super) fn progress(&self) -> PhysicalCheckpointProgress {
        self.state().progress
    }

    pub(super) fn poll(&self) -> PhysicalCheckpointPoll {
        let state = self.state();
        state.terminal.clone().map_or(
            PhysicalCheckpointPoll::Pending(state.progress),
            PhysicalCheckpointPoll::Terminal,
        )
    }

    pub(super) fn enter(&self, phase: PhysicalCheckpointProgressPhase) {
        let mut state = self.state();
        state.progress.enter(phase);
    }

    pub(super) fn record_capture(&self, dirty_frames: u64, encoded_bytes: u64) {
        let mut state = self.state();
        state.progress.record_capture(dirty_frames, encoded_bytes);
    }

    pub(super) fn begin_capture_allocation(
        &self,
        bytes: u64,
    ) -> PhysicalCheckpointCaptureAllocation<'_> {
        let mut state = self.state();
        state.progress.begin_capture_allocation(bytes);
        PhysicalCheckpointCaptureAllocation { attempt: self }
    }

    pub(super) fn cancellation_requested(&self) -> bool {
        self.pre_publication_stop_cause(false).is_some()
    }

    pub(super) fn pre_publication_stop_cause(
        &self,
        candidate_created: bool,
    ) -> Option<super::PhysicalCheckpointProvenNoEffectCause> {
        let state = self.state();
        if state.publication_started {
            return None;
        }
        let stop = state
            .interruption
            .as_ref()
            .and_then(|view| view.check_live().err());
        if matches!(stop, Some(worth_execution::MapKernelStop::DeadlineElapsed)) {
            return Some(if candidate_created {
                super::PhysicalCheckpointProvenNoEffectCause::OriginalRequestDeadlineElapsedAndCandidateRemoved
            } else {
                super::PhysicalCheckpointProvenNoEffectCause::OriginalRequestDeadlineElapsedBeforeCandidate
            });
        }
        if state.progress.cancellation_requested() || stop.is_some() {
            Some(if candidate_created {
                super::PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved
            } else {
                super::PhysicalCheckpointProvenNoEffectCause::CancelledBeforeCandidate
            })
        } else {
            None
        }
    }

    pub(super) fn begin_publication(&self) -> bool {
        let mut state = self.state();
        if state.progress.cancellation_requested()
            || state
                .interruption
                .as_ref()
                .is_some_and(|view| view.check_live().is_err())
        {
            return false;
        }
        state.publication_started = true;
        state
            .progress
            .enter(PhysicalCheckpointProgressPhase::PublicationReplacement);
        true
    }

    pub(super) fn complete(&self, terminal: PhysicalCheckpointOutcome) {
        let mut state = self.state();
        state
            .progress
            .enter(PhysicalCheckpointProgressPhase::Terminal);
        state.terminal = Some(terminal);
        state.interruption = None;
        self.changed.notify_all();
    }

    pub(super) fn mark_runtime_closing(&self) {
        let mut state = self.state();
        state.runtime_closing = true;
        if !state.publication_started && state.terminal.is_none() {
            state.progress.request_cancellation();
        }
    }

    fn request_cancellation(&self) -> PhysicalCheckpointCancellationOutcome {
        let mut state = self.state();
        if let Some(terminal) = state.terminal.clone() {
            return PhysicalCheckpointCancellationOutcome::AlreadyTerminal(terminal);
        }
        if state.runtime_closing {
            return PhysicalCheckpointCancellationOutcome::RuntimeClosing {
                identity: state.progress.identity(),
            };
        }
        if state.publication_started {
            return PhysicalCheckpointCancellationOutcome::PublicationAlreadyEffectful {
                identity: state.progress.identity(),
            };
        }
        state.progress.request_cancellation();
        PhysicalCheckpointCancellationOutcome::Accepted {
            identity: state.progress.identity(),
        }
    }

    fn wait(&self) -> PhysicalCheckpointOutcome {
        let mut state = self.state();
        loop {
            if let Some(terminal) = state.terminal.clone() {
                return terminal;
            }
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn dispose(&self) -> PhysicalCheckpointDisposal {
        let state = self.state();
        state.terminal.clone().map_or(
            PhysicalCheckpointDisposal::ObservationAbandoned {
                identity: state.progress.identity(),
            },
            PhysicalCheckpointDisposal::Terminal,
        )
    }

    fn state(&self) -> std::sync::MutexGuard<'_, PhysicalCheckpointAttemptState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for PhysicalCheckpointCaptureAllocation<'_> {
    fn drop(&mut self) {
        let mut state = self.attempt.state();
        state.progress.end_capture_allocation();
    }
}
