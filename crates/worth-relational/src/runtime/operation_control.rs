use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Typed reason why owner work stopped before its effect boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalOperationInterruption {
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalInterruptionBoundary {
    ObservationAdmission,
    TransactionAdmission,
    ProposalValidation,
    CandidatePreparation,
    PublicationPreflight,
    BeforeCriticalSection,
    AfterLinearization,
    Settlement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalInterruptionEvent {
    interruption: RelationalOperationInterruption,
    boundary: RelationalInterruptionBoundary,
}

impl RelationalInterruptionEvent {
    /// Observes the caller's cancellation or deadline at a named boundary.
    /// This consults borrowed request control only; it creates no request and
    /// performs no storage work. Readers still charge their own native work.
    /// Retains the interruption and boundary of an already-produced request stop.
    /// Work and nested stops remain the caller's typed stop rather than an event.
    pub fn from_request_stop(
        stop: worth_execution::MapKernelStop,
        boundary: RelationalInterruptionBoundary,
    ) -> Result<Self, worth_execution::MapKernelStop> {
        use worth_execution::MapKernelStop as Stop;
        let interruption = match stop {
            Stop::Cancelled => RelationalOperationInterruption::Cancelled,
            Stop::DeadlineElapsed => RelationalOperationInterruption::TimedOut,
            Stop::WorkCounterOverflow | Stop::WorkCeiling | Stop::NestedStopped => return Err(stop),
        };
        Ok(Self { interruption, boundary })
    }

    #[cfg(test)]
    pub(crate) fn observe_request(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        boundary: RelationalInterruptionBoundary,
    ) -> Option<Self> {
        match execution.check_live() {
            Ok(()) => None,
            Err(stop) => Self::from_request_stop(stop, boundary).ok(),
        }
    }

    pub const fn interruption(self) -> RelationalOperationInterruption {
        self.interruption
    }

    pub const fn boundary(self) -> RelationalInterruptionBoundary {
        self.boundary
    }
}

/// Caller-owned cancellation source. Tokens are cheap, read-only shares.
#[derive(Debug, Clone, Default)]
pub struct RelationalCancellationSource {
    requested: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct RelationalCancellationToken {
    requested: Arc<AtomicBool>,
}

/// Non-serializable in-process control carried across one Relational attempt.
#[derive(Debug, Clone)]
pub struct RelationalOperationControl {
    cancellation: RelationalCancellationToken,
    deadline: Option<Instant>,
    injected_interruption: Option<RelationalInjectedInterruption>,
    post_linearization_pause: Option<RelationalPostLinearizationPause>,
    critical_section_pause: Option<RelationalCriticalSectionPause>,
    patch_position_reservation_pause: Option<RelationalPatchPositionReservationPause>,
    boundary_pause: Option<RelationalBoundaryPause>,
}

#[derive(Debug, Clone)]
struct RelationalInjectedInterruption {
    boundary: RelationalInterruptionBoundary,
    interruption: RelationalOperationInterruption,
    trigger_on_visit: usize,
    visits: Arc<std::sync::atomic::AtomicUsize>,
}

#[derive(Debug, Clone)]
struct RelationalPostLinearizationPause {
    reached: Arc<std::sync::Barrier>,
    release: Arc<std::sync::Barrier>,
}

#[derive(Debug, Clone)]
struct RelationalCriticalSectionPause {
    reached: Arc<std::sync::Barrier>,
    release: Arc<std::sync::Barrier>,
}

/// Failure budget for a publisher nobody releases. It is not the ordering
/// mechanism: the observer opens the gate, and this only bounds the damage when
/// the observer never gets there.
const PATCH_POSITION_RESERVATION_PAUSE_BUDGET: std::time::Duration =
    std::time::Duration::from_secs(60);

/// Release gate for the reservation-held pause. Opening never blocks, so an
/// observer can free the publisher before it asserts, and waiting is bounded, so
/// a seam that is moved out of the reservation window fails a run by name
/// instead of hanging it.
#[derive(Debug, Default)]
pub struct RelationalPatchPositionReservationGateForTest {
    opened: std::sync::Mutex<bool>,
    signal: std::sync::Condvar,
}

impl RelationalPatchPositionReservationGateForTest {
    pub fn open(&self) {
        let mut opened = self.opened.lock().expect("the release gate stays usable");
        *opened = true;
        self.signal.notify_all();
    }

    fn wait_for_open(&self, budget: std::time::Duration) {
        let deadline = Instant::now() + budget;
        let mut opened = self.opened.lock().expect("the release gate stays usable");
        while !*opened {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return;
            };
            opened = self
                .signal
                .wait_timeout(opened, remaining)
                .expect("the release gate stays usable")
                .0;
        }
    }
}

#[derive(Debug, Clone)]
struct RelationalPatchPositionReservationPause {
    reached: std::sync::mpsc::SyncSender<()>,
    release: Arc<RelationalPatchPositionReservationGateForTest>,
}

#[derive(Debug, Clone)]
struct RelationalBoundaryPause {
    boundary: RelationalInterruptionBoundary,
    reached: Arc<std::sync::Barrier>,
    release: Arc<std::sync::Barrier>,
    used: Arc<AtomicBool>,
}

impl RelationalCancellationSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(&self) -> RelationalCancellationToken {
        RelationalCancellationToken {
            requested: Arc::clone(&self.requested),
        }
    }

    pub fn cancel(&self) {
        self.requested.store(true, Ordering::Release);
    }
}

impl RelationalOperationControl {
    pub fn uninterrupted() -> Self {
        RelationalCancellationSource::new().token().into()
    }

    pub fn with_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn interruption(&self) -> Option<RelationalOperationInterruption> {
        if self.cancellation.requested.load(Ordering::Acquire) {
            return Some(RelationalOperationInterruption::Cancelled);
        }
        self.deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
            .then_some(RelationalOperationInterruption::TimedOut)
    }

    pub fn observe(
        &self,
        boundary: RelationalInterruptionBoundary,
    ) -> Option<RelationalInterruptionEvent> {
        if let Some(pause) = &self.boundary_pause {
            if pause.boundary == boundary
                && pause
                    .used
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            {
                pause.reached.wait();
                pause.release.wait();
            }
        }
        if let Some(injected) = &self.injected_interruption {
            if injected.boundary == boundary
                && injected.visits.fetch_add(1, Ordering::AcqRel) + 1 == injected.trigger_on_visit
            {
                return Some(RelationalInterruptionEvent {
                    interruption: injected.interruption,
                    boundary,
                });
            }
        }
        self.interruption()
            .map(|interruption| RelationalInterruptionEvent {
                interruption,
                boundary,
            })
    }

    pub(crate) fn pause_after_linearization_for_test(&self) {
        if let Some(pause) = &self.post_linearization_pause {
            pause.reached.wait();
            pause.release.wait();
        }
    }

    pub(crate) fn pause_inside_critical_section_for_test(&self) {
        if let Some(pause) = &self.critical_section_pause {
            pause.reached.wait();
            pause.release.wait();
        }
    }

    pub(crate) fn pause_holding_patch_position_reservation_for_test(&self) {
        if let Some(pause) = &self.patch_position_reservation_pause {
            // A dropped observer is an observer that already gave up, so the
            // publisher must not wait on a gate nobody is left to open.
            if pause.reached.send(()).is_ok() {
                pause
                    .release
                    .wait_for_open(PATCH_POSITION_RESERVATION_PAUSE_BUDGET);
            }
        }
    }
}

impl From<RelationalCancellationToken> for RelationalOperationControl {
    fn from(cancellation: RelationalCancellationToken) -> Self {
        Self {
            cancellation,
            deadline: None,
            injected_interruption: None,
            post_linearization_pause: None,
            critical_section_pause: None,
            patch_position_reservation_pause: None,
            boundary_pause: None,
        }
    }
}

#[cfg(test)]
#[path = "operation_control/request_observation_tests.rs"]
mod request_observation_tests;

mod fault_injection;
