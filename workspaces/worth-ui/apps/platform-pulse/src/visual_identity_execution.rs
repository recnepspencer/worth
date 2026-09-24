use std::time::{Duration, Instant};

use worth_ui::facade::app::WorthUiNativeApplicationShell;
use worth_ui::facade::inspection::{
    UiCurrentPresentedSurfaceTarget, UiPendingVisualCapture, UiPixelsRequired,
    UiPublishedVisualOverlay, UiVisualSnapshotReceipt,
};

use crate::lifecycle_observation_publication::PlatformPulseObservationPublisher;

mod activation;
mod capture_restart;
mod comparison;
mod denial;
mod frame_affinity;
mod progression;
mod readiness;
mod replacement;
mod state_progression;

pub(crate) use activation::PlatformPulseVisualIdentityExecution;
use denial::PlatformPulseVisualCapturePhase;
pub(crate) use denial::PlatformPulseVisualExecutionDenial;
use state_progression::{
    advance_awaiting_capture, advance_awaiting_comparison, advance_awaiting_rebase,
    advance_awaiting_refresh, next_capture_wake, next_frame_readiness_poll,
    replacement_frame_deadline,
};

const NATIVE_CAPTURE_POLL_INTERVAL: Duration = Duration::from_millis(1);
const REPLACEMENT_FRAME_DEADLINE: Duration = Duration::from_secs(5);

/// The enabled visual identity journey. `Transitioning` stands in for the
/// state while one transition owns it, so a reentrant transition is denied.
pub(crate) struct PlatformPulseVisualIdentityJourney {
    state: PlatformPulseVisualIdentityState,
    readiness: Option<readiness::PlatformPulseVisualReadiness>,
    queued_rebind: Option<worth_ui::facade::rebind::UiRebindReceipt>,
}

enum PlatformPulseVisualIdentityState {
    AwaitingFirstFrame,
    Settling {
        begin_at: Instant,
        deadline: Instant,
    },
    Capturing(PlatformPulseVisualCapture),
    AwaitingCapture {
        budget: capture_restart::PlatformPulseAwaitingCaptureBudget,
    },
    OverlayVisible(PlatformPulseVisibleOverlay),
    ComparisonReady(PlatformPulseRetainedSnapshot),
    AwaitingRebase {
        budget: capture_restart::PlatformPulseAwaitingCaptureBudget,
    },
    AwaitingRefresh {
        predecessor: PlatformPulseRetainedSnapshot,
        budget: capture_restart::PlatformPulseAwaitingCaptureBudget,
    },
    Rebasing(PlatformPulseVisualCapture),
    Refreshing(PlatformPulseVisualRefreshCapture),
    AwaitingComparison {
        predecessor: PlatformPulseRetainedSnapshot,
        rebind: worth_ui::facade::rebind::UiRebindReceipt,
        budget: capture_restart::PlatformPulseAwaitingCaptureBudget,
    },
    Comparing(comparison::PlatformPulseVisualComparisonCapture),
    Transitioning,
    Failed,
    Retired,
}

struct PlatformPulseVisualCapture {
    pending: UiPendingVisualCapture<UiCurrentPresentedSurfaceTarget, UiPixelsRequired>,
    deadline: Instant,
    replacement: replacement::PlatformPulseReplacementPosture,
}

struct PlatformPulseVisualRefreshCapture {
    capture: PlatformPulseVisualCapture,
    predecessor: PlatformPulseRetainedSnapshot,
}

struct PlatformPulseRetainedSnapshot {
    snapshot: UiVisualSnapshotReceipt<UiPixelsRequired>,
    overlay_clear: Option<worth_ui::facade::inspection::UiClearedVisualOverlayReceipt>,
}

struct PlatformPulseVisibleOverlay {
    retained: PlatformPulseRetainedSnapshot,
    published: UiPublishedVisualOverlay,
    clear_at: Instant,
    replacement: replacement::PlatformPulseReplacementPosture,
}

impl PlatformPulseVisualIdentityJourney {
    fn new() -> Self {
        Self {
            state: PlatformPulseVisualIdentityState::AwaitingFirstFrame,
            readiness: None,
            queued_rebind: None,
        }
    }

    fn install_readiness(
        &mut self,
        signal: worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal,
    ) {
        self.readiness = Some(readiness::PlatformPulseVisualReadiness::install(signal));
    }

    fn begin_transition(
        &mut self,
    ) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
        match std::mem::replace(
            &mut self.state,
            PlatformPulseVisualIdentityState::Transitioning,
        ) {
            PlatformPulseVisualIdentityState::Transitioning => {
                Err(PlatformPulseVisualExecutionDenial::ReentrantTransition)
            }
            state => Ok(state),
        }
    }

    pub(crate) fn retains_rebind_receipt(&self) -> bool {
        self.queued_rebind.is_some()
            || matches!(
                &self.state,
                PlatformPulseVisualIdentityState::AwaitingComparison { .. }
                    | PlatformPulseVisualIdentityState::Comparing(_)
            )
    }

    pub(crate) fn shutdown_quiescent(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        self.queued_rebind.take();
        let state = self.begin_transition()?;
        match state {
            PlatformPulseVisualIdentityState::ComparisonReady(retained)
            | PlatformPulseVisualIdentityState::AwaitingRefresh {
                predecessor: retained,
                ..
            } => {
                shell.dispose_visual_snapshot(retained.snapshot);
                self.state = PlatformPulseVisualIdentityState::Retired;
                Ok(())
            }
            PlatformPulseVisualIdentityState::AwaitingFirstFrame
            | PlatformPulseVisualIdentityState::AwaitingCapture { .. }
            | PlatformPulseVisualIdentityState::AwaitingRebase { .. }
            | PlatformPulseVisualIdentityState::Settling { .. }
            | PlatformPulseVisualIdentityState::Failed
            | PlatformPulseVisualIdentityState::Retired => {
                self.state = PlatformPulseVisualIdentityState::Retired;
                Ok(())
            }
            PlatformPulseVisualIdentityState::AwaitingComparison {
                predecessor,
                rebind,
                ..
            } => {
                shell.dispose_visual_snapshot(predecessor.snapshot);
                drop(rebind);
                self.state = PlatformPulseVisualIdentityState::Retired;
                Ok(())
            }
            PlatformPulseVisualIdentityState::Capturing(capture)
            | PlatformPulseVisualIdentityState::Rebasing(capture) => {
                shell.cancel_visual_snapshot(capture.pending);
                self.state = PlatformPulseVisualIdentityState::Retired;
                Ok(())
            }
            PlatformPulseVisualIdentityState::Refreshing(refresh) => {
                shell.cancel_visual_snapshot(refresh.capture.pending);
                shell.dispose_visual_snapshot(refresh.predecessor.snapshot);
                self.state = PlatformPulseVisualIdentityState::Retired;
                Ok(())
            }
            PlatformPulseVisualIdentityState::Comparing(comparison) => {
                comparison.cancel(shell);
                self.state = PlatformPulseVisualIdentityState::Retired;
                Ok(())
            }
            state => {
                self.state = state;
                Err(PlatformPulseVisualExecutionDenial::ShutdownNotQuiescent)
            }
        }
    }

    pub(crate) fn arm_after_first_frame(
        &mut self,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        let state = self.begin_transition()?;
        if !matches!(state, PlatformPulseVisualIdentityState::AwaitingFirstFrame) {
            self.state = state;
            return Err(PlatformPulseVisualExecutionDenial::InitialFrameAlreadyArmed);
        }
        let begin_at = now;
        let deadline = now
            .checked_add(REPLACEMENT_FRAME_DEADLINE)
            .ok_or(PlatformPulseVisualExecutionDenial::ClockOverflow)?;
        self.state = PlatformPulseVisualIdentityState::Settling { begin_at, deadline };
        self.schedule_wake(begin_at);
        Ok(())
    }

    pub(crate) fn advance(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        publisher: &PlatformPulseObservationPublisher,
        tick: &mut u64,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        let state = self.begin_transition()?;
        let next = match state {
            PlatformPulseVisualIdentityState::AwaitingCapture { budget } => {
                advance_awaiting_capture(shell, *tick, now, budget)
            }
            PlatformPulseVisualIdentityState::AwaitingRebase { budget } => {
                advance_awaiting_rebase(shell, *tick, now, budget)
            }
            PlatformPulseVisualIdentityState::AwaitingRefresh {
                predecessor,
                budget,
            } => advance_awaiting_refresh(shell, *tick, now, predecessor, budget),
            PlatformPulseVisualIdentityState::AwaitingComparison {
                predecessor,
                rebind,
                budget,
            } => advance_awaiting_comparison(shell, *tick, now, predecessor, rebind, budget),
            state => progression::advance_state(state, shell, publisher, tick, now),
        };
        self.install_advance_result(next)?;
        self.admit_queued_rebind(shell, *tick, now)
    }

    pub(crate) fn compare_after_rebind(
        &mut self,
        _shell: &mut WorthUiNativeApplicationShell,
        rebind: worth_ui::facade::rebind::UiRebindReceipt,
        _tick: u64,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        let deadline = replacement_frame_deadline(now)?;
        let state = self.begin_transition()?;
        let predecessor = match state {
            PlatformPulseVisualIdentityState::ComparisonReady(predecessor)
            | PlatformPulseVisualIdentityState::AwaitingRefresh { predecessor, .. } => predecessor,
            PlatformPulseVisualIdentityState::Retired
            | PlatformPulseVisualIdentityState::AwaitingRebase { .. } => {
                drop(rebind);
                self.state = PlatformPulseVisualIdentityState::AwaitingRebase {
                    budget: capture_restart::PlatformPulseAwaitingCaptureBudget::fresh(deadline),
                };
                self.schedule_current_wake();
                return Ok(());
            }
            state @ (PlatformPulseVisualIdentityState::AwaitingFirstFrame
            | PlatformPulseVisualIdentityState::Settling { .. }
            | PlatformPulseVisualIdentityState::Capturing(_)
            | PlatformPulseVisualIdentityState::AwaitingCapture { .. }
            | PlatformPulseVisualIdentityState::OverlayVisible(_)
            | PlatformPulseVisualIdentityState::Rebasing(_)
            | PlatformPulseVisualIdentityState::Refreshing(_)
            | PlatformPulseVisualIdentityState::Comparing(_)) => {
                self.state = state;
                self.queued_rebind = Some(rebind);
                return Ok(());
            }
            state => {
                self.state = state;
                return Err(PlatformPulseVisualExecutionDenial::ReplacementBeforeOverlayClear);
            }
        };
        self.state = PlatformPulseVisualIdentityState::AwaitingComparison {
            predecessor,
            rebind,
            budget: capture_restart::PlatformPulseAwaitingCaptureBudget::fresh(deadline),
        };
        self.schedule_current_wake();
        Ok(())
    }

    fn admit_queued_rebind(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        tick: u64,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        if !matches!(
            &self.state,
            PlatformPulseVisualIdentityState::ComparisonReady(_)
                | PlatformPulseVisualIdentityState::AwaitingRefresh { .. }
                | PlatformPulseVisualIdentityState::Retired
                | PlatformPulseVisualIdentityState::AwaitingRebase { .. }
        ) {
            return Ok(());
        }
        let Some(rebind) = self.queued_rebind.take() else {
            return Ok(());
        };
        self.compare_after_rebind(shell, rebind, tick, now)
    }

    fn install_advance_result(
        &mut self,
        next: Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial>,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        match next {
            Ok(next) => {
                self.state = next;
                self.schedule_current_wake();
                Ok(())
            }
            Err(denial) => {
                self.state = PlatformPulseVisualIdentityState::Failed;
                Err(denial)
            }
        }
    }

    fn schedule_current_wake(&self) {
        let deadline = match &self.state {
            PlatformPulseVisualIdentityState::Settling { begin_at, .. } => Some(*begin_at),
            PlatformPulseVisualIdentityState::Capturing(capture)
            | PlatformPulseVisualIdentityState::Rebasing(capture) => Some(next_capture_wake(
                capture.deadline,
                capture.replacement.is_pending(),
            )),
            PlatformPulseVisualIdentityState::AwaitingCapture { budget } => {
                Some(next_frame_readiness_poll(budget.readiness_deadline()))
            }
            PlatformPulseVisualIdentityState::Refreshing(refresh) => Some(next_capture_wake(
                refresh.capture.deadline,
                refresh.capture.replacement.is_pending(),
            )),
            PlatformPulseVisualIdentityState::Comparing(comparison) => Some(next_capture_wake(
                comparison.deadline(),
                comparison.replacement_pending(),
            )),
            PlatformPulseVisualIdentityState::AwaitingComparison { budget, .. } => {
                Some(next_frame_readiness_poll(budget.readiness_deadline()))
            }
            PlatformPulseVisualIdentityState::AwaitingRebase { budget }
            | PlatformPulseVisualIdentityState::AwaitingRefresh { budget, .. } => {
                Some(next_frame_readiness_poll(budget.readiness_deadline()))
            }
            PlatformPulseVisualIdentityState::OverlayVisible(overlay) => Some(overlay.clear_at),
            _ => None,
        };
        if let Some(deadline) = deadline {
            self.schedule_wake(deadline);
        }
    }

    fn schedule_wake(&self, deadline: Instant) {
        if let Some(readiness) = self.readiness.as_ref() {
            readiness.schedule(deadline);
        }
    }
}

#[cfg(test)]
mod tests;
