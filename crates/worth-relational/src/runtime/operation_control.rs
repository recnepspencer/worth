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
pub struct RelationalPatchPositionReservationGate {
    opened: std::sync::Mutex<bool>,
    signal: std::sync::Condvar,
}

impl RelationalPatchPositionReservationGate {
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
    release: Arc<RelationalPatchPositionReservationGate>,
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

    pub fn with_injected_interruption(
        mut self,
        boundary: RelationalInterruptionBoundary,
        interruption: RelationalOperationInterruption,
        trigger_on_visit: usize,
    ) -> Self {
        assert!(
            trigger_on_visit > 0,
            "an interruption seam needs a positive visit"
        );
        self.injected_interruption = Some(RelationalInjectedInterruption {
            boundary,
            interruption,
            trigger_on_visit,
            visits: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        });
        self
    }

    pub fn with_post_linearization_pause(
        mut self,
        reached: Arc<std::sync::Barrier>,
        release: Arc<std::sync::Barrier>,
    ) -> Self {
        self.post_linearization_pause = Some(RelationalPostLinearizationPause { reached, release });
        self
    }

    pub(crate) fn pause_after_linearization(&self) {
        if let Some(pause) = &self.post_linearization_pause {
            pause.reached.wait();
            pause.release.wait();
        }
    }

    pub fn with_critical_section_pause(
        mut self,
        reached: Arc<std::sync::Barrier>,
        release: Arc<std::sync::Barrier>,
    ) -> Self {
        self.critical_section_pause = Some(RelationalCriticalSectionPause { reached, release });
        self
    }

    pub(crate) fn pause_inside_critical_section(&self) {
        if let Some(pause) = &self.critical_section_pause {
            pause.reached.wait();
            pause.release.wait();
        }
    }

    /// Pause inside the caller-owned publication cutover, the one region that
    /// provably runs while the global patch-position reservation is held.
    /// Holding the reservation there makes a second publisher's contention
    /// deferral a scheduled fact rather than a timing coincidence.
    ///
    /// The handshake is bounded on both sides. The observer is told the seam was
    /// reached over `reached`, so a seam that never runs is a timed-out receive
    /// rather than a stalled run, and the publisher stops waiting once the
    /// observer is gone.
    pub fn with_patch_position_reservation_pause(
        mut self,
        reached: std::sync::mpsc::SyncSender<()>,
        release: Arc<RelationalPatchPositionReservationGate>,
    ) -> Self {
        self.patch_position_reservation_pause =
            Some(RelationalPatchPositionReservationPause { reached, release });
        self
    }

    pub(crate) fn pause_holding_patch_position_reservation(&self) {
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

    pub fn with_boundary_pause(
        mut self,
        boundary: RelationalInterruptionBoundary,
        reached: Arc<std::sync::Barrier>,
        release: Arc<std::sync::Barrier>,
    ) -> Self {
        self.boundary_pause = Some(RelationalBoundaryPause {
            boundary,
            reached,
            release,
            used: Arc::new(AtomicBool::new(false)),
        });
        self
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
