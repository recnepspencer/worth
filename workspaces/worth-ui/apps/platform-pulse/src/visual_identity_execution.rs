use std::time::{Duration, Instant};

use worth_ui::facade::app::WorthUiNativeApplicationShell;
use worth_ui::facade::inspection::{
    UiCurrentPresentedSurfaceTarget, UiPendingVisualCapture, UiPixelsRequired,
    UiPublishedVisualOverlay, UiVisualSnapshotReceipt,
};

use crate::lifecycle_observation_publication::PlatformPulseObservationPublisher;

mod capture_restart;
mod comparison;
mod denial;
mod frame_affinity;
mod progression;
mod readiness;
mod replacement;
mod state_progression;

use denial::PlatformPulseVisualCapturePhase;
pub(crate) use denial::PlatformPulseVisualExecutionDenial;
use state_progression::{
    advance_awaiting_capture, advance_awaiting_comparison, advance_awaiting_rebase,
    advance_awaiting_refresh, next_capture_wake, next_frame_readiness_poll,
    replacement_frame_deadline,
};

const NATIVE_CAPTURE_POLL_INTERVAL: Duration = Duration::from_millis(1);
const REPLACEMENT_FRAME_DEADLINE: Duration = Duration::from_secs(5);

pub(crate) struct PlatformPulseVisualIdentityExecution {
    state: Option<PlatformPulseVisualIdentityState>,
    enabled: bool,
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

impl PlatformPulseVisualIdentityExecution {
    pub(crate) fn new() -> Self {
        Self {
            state: Some(PlatformPulseVisualIdentityState::AwaitingFirstFrame),
            enabled: std::env::var_os("WORTH_UI_VISUAL_IDENTITY_JOURNEY")
                .is_some_and(|value| value == "1"),
            readiness: None,
            queued_rebind: None,
        }
    }

    pub(crate) fn install_readiness(
        &mut self,
        signal: worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal,
    ) {
        if self.enabled {
            self.readiness = Some(readiness::PlatformPulseVisualReadiness::install(signal));
        }
    }

    pub(crate) fn retains_rebind_receipt(&self) -> bool {
        self.queued_rebind.is_some()
            || matches!(
                self.state.as_ref(),
                Some(
                    PlatformPulseVisualIdentityState::AwaitingComparison { .. }
                        | PlatformPulseVisualIdentityState::Comparing(_)
                )
            )
    }

    pub(crate) fn shutdown_quiescent(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        self.queued_rebind.take();
        let state = self
            .state
            .take()
            .ok_or(PlatformPulseVisualExecutionDenial::ReentrantTransition)?;
        match state {
            PlatformPulseVisualIdentityState::ComparisonReady(retained)
            | PlatformPulseVisualIdentityState::AwaitingRefresh {
                predecessor: retained,
                ..
            } => {
                shell.dispose_visual_snapshot(retained.snapshot);
                self.state = Some(PlatformPulseVisualIdentityState::Retired);
                Ok(())
            }
            PlatformPulseVisualIdentityState::AwaitingFirstFrame
            | PlatformPulseVisualIdentityState::AwaitingCapture { .. }
            | PlatformPulseVisualIdentityState::AwaitingRebase { .. }
            | PlatformPulseVisualIdentityState::Settling { .. }
            | PlatformPulseVisualIdentityState::Failed
            | PlatformPulseVisualIdentityState::Retired => {
                self.state = Some(PlatformPulseVisualIdentityState::Retired);
                Ok(())
            }
            PlatformPulseVisualIdentityState::AwaitingComparison {
                predecessor,
                rebind,
                ..
            } => {
                shell.dispose_visual_snapshot(predecessor.snapshot);
                drop(rebind);
                self.state = Some(PlatformPulseVisualIdentityState::Retired);
                Ok(())
            }
            PlatformPulseVisualIdentityState::Capturing(capture)
            | PlatformPulseVisualIdentityState::Rebasing(capture) => {
                shell.cancel_visual_snapshot(capture.pending);
                self.state = Some(PlatformPulseVisualIdentityState::Retired);
                Ok(())
            }
            PlatformPulseVisualIdentityState::Refreshing(refresh) => {
                shell.cancel_visual_snapshot(refresh.capture.pending);
                shell.dispose_visual_snapshot(refresh.predecessor.snapshot);
                self.state = Some(PlatformPulseVisualIdentityState::Retired);
                Ok(())
            }
            PlatformPulseVisualIdentityState::Comparing(comparison) => {
                comparison.cancel(shell);
                self.state = Some(PlatformPulseVisualIdentityState::Retired);
                Ok(())
            }
            state => {
                self.state = Some(state);
                Err(PlatformPulseVisualExecutionDenial::ShutdownNotQuiescent)
            }
        }
    }

    pub(crate) fn arm_after_first_frame(
        &mut self,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        let state = self
            .state
            .take()
            .ok_or(PlatformPulseVisualExecutionDenial::ReentrantTransition)?;
        if !matches!(state, PlatformPulseVisualIdentityState::AwaitingFirstFrame) {
            self.state = Some(state);
            return Err(PlatformPulseVisualExecutionDenial::InitialFrameAlreadyArmed);
        }
        if !self.enabled {
            self.state = Some(PlatformPulseVisualIdentityState::Retired);
            return Ok(());
        }
        let begin_at = now;
        let deadline = now
            .checked_add(REPLACEMENT_FRAME_DEADLINE)
            .ok_or(PlatformPulseVisualExecutionDenial::ClockOverflow)?;
        self.state = Some(PlatformPulseVisualIdentityState::Settling { begin_at, deadline });
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
        let state = self
            .state
            .replace(PlatformPulseVisualIdentityState::Transitioning)
            .ok_or(PlatformPulseVisualExecutionDenial::ReentrantTransition)?;
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
        if !self.enabled {
            return Ok(());
        }
        let deadline = replacement_frame_deadline(now)?;
        let state = self
            .state
            .replace(PlatformPulseVisualIdentityState::Transitioning)
            .ok_or(PlatformPulseVisualExecutionDenial::ReentrantTransition)?;
        let predecessor = match state {
            PlatformPulseVisualIdentityState::ComparisonReady(predecessor)
            | PlatformPulseVisualIdentityState::AwaitingRefresh { predecessor, .. } => predecessor,
            PlatformPulseVisualIdentityState::Retired
            | PlatformPulseVisualIdentityState::AwaitingRebase { .. } => {
                drop(rebind);
                self.state = Some(PlatformPulseVisualIdentityState::AwaitingRebase {
                    budget: capture_restart::PlatformPulseAwaitingCaptureBudget::fresh(deadline),
                });
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
                self.state = Some(state);
                self.queued_rebind = Some(rebind);
                return Ok(());
            }
            state => {
                self.state = Some(state);
                return Err(PlatformPulseVisualExecutionDenial::ReplacementBeforeOverlayClear);
            }
        };
        self.state = Some(PlatformPulseVisualIdentityState::AwaitingComparison {
            predecessor,
            rebind,
            budget: capture_restart::PlatformPulseAwaitingCaptureBudget::fresh(deadline),
        });
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
            self.state.as_ref(),
            Some(PlatformPulseVisualIdentityState::ComparisonReady(_))
                | Some(PlatformPulseVisualIdentityState::AwaitingRefresh { .. })
                | Some(PlatformPulseVisualIdentityState::Retired)
                | Some(PlatformPulseVisualIdentityState::AwaitingRebase { .. })
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
                self.state = Some(next);
                self.schedule_current_wake();
                Ok(())
            }
            Err(denial) => {
                self.state = Some(PlatformPulseVisualIdentityState::Failed);
                Err(denial)
            }
        }
    }

    fn schedule_current_wake(&self) {
        let deadline = match self.state.as_ref() {
            Some(PlatformPulseVisualIdentityState::Settling { begin_at, .. }) => Some(*begin_at),
            Some(PlatformPulseVisualIdentityState::Capturing(capture))
            | Some(PlatformPulseVisualIdentityState::Rebasing(capture)) => Some(next_capture_wake(
                capture.deadline,
                capture.replacement.is_pending(),
            )),
            Some(PlatformPulseVisualIdentityState::AwaitingCapture { budget }) => {
                Some(next_frame_readiness_poll(budget.readiness_deadline()))
            }
            Some(PlatformPulseVisualIdentityState::Refreshing(refresh)) => Some(next_capture_wake(
                refresh.capture.deadline,
                refresh.capture.replacement.is_pending(),
            )),
            Some(PlatformPulseVisualIdentityState::Comparing(comparison)) => Some(
                next_capture_wake(comparison.deadline(), comparison.replacement_pending()),
            ),
            Some(PlatformPulseVisualIdentityState::AwaitingComparison { budget, .. }) => {
                Some(next_frame_readiness_poll(budget.readiness_deadline()))
            }
            Some(PlatformPulseVisualIdentityState::AwaitingRebase { budget })
            | Some(PlatformPulseVisualIdentityState::AwaitingRefresh { budget, .. }) => {
                Some(next_frame_readiness_poll(budget.readiness_deadline()))
            }
            Some(PlatformPulseVisualIdentityState::OverlayVisible(overlay)) => {
                Some(overlay.clear_at)
            }
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
