use std::time::Instant;

use worth_ui::facade::app::{UiMountedInspectionOmission, WorthUiNativeApplicationShell};

use super::{
    progression, replacement_frame_deadline, PlatformPulseRetainedSnapshot,
    PlatformPulseVisualExecutionDenial, PlatformPulseVisualIdentityState,
    PlatformPulseVisualRefreshCapture,
};

pub(super) fn initial_after_stale_frame(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    restart(
        shell,
        tick,
        CaptureRestartBudget::FreshAt(now),
        CaptureRestartKind::Initial,
    )
}

pub(super) fn rebase_after_stale_frame(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    restart(
        shell,
        tick,
        CaptureRestartBudget::FreshAt(now),
        CaptureRestartKind::Rebase,
    )
}

pub(super) fn refresh_after_stale_frame(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    predecessor: PlatformPulseRetainedSnapshot,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    restart_refresh(shell, tick, predecessor, CaptureRestartBudget::FreshAt(now))
}

fn restart_refresh(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    predecessor: PlatformPulseRetainedSnapshot,
    budget: CaptureRestartBudget,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    match progression::begin_capture_before(shell, tick, budget.capture_deadline()?) {
        Ok(capture) => Ok(PlatformPulseVisualIdentityState::Refreshing(
            PlatformPulseVisualRefreshCapture {
                capture,
                predecessor,
            },
        )),
        Err(PlatformPulseVisualExecutionDenial::CaptureMountedFrameUnavailable(
            UiMountedInspectionOmission::FrameTransitionInFlight,
        )) => Ok(PlatformPulseVisualIdentityState::AwaitingRefresh {
            predecessor,
            budget: budget.awaiting_budget()?,
        }),
        Err(denial) => Err(denial),
    }
}

pub(super) fn initial_after_host_supersession(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    deadline: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    restart(
        shell,
        tick,
        CaptureRestartBudget::PreserveUntil(deadline),
        CaptureRestartKind::Initial,
    )
}

pub(super) fn rebase_after_host_supersession(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    deadline: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    restart(
        shell,
        tick,
        CaptureRestartBudget::PreserveUntil(deadline),
        CaptureRestartKind::Rebase,
    )
}

pub(super) fn refresh_after_host_supersession(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    predecessor: PlatformPulseRetainedSnapshot,
    deadline: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    restart_refresh(
        shell,
        tick,
        predecessor,
        CaptureRestartBudget::PreserveUntil(deadline),
    )
}

fn restart(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    budget: CaptureRestartBudget,
    kind: CaptureRestartKind,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    match progression::begin_capture_before(shell, tick, budget.capture_deadline()?) {
        Ok(capture) => Ok(kind.captured(capture)),
        Err(PlatformPulseVisualExecutionDenial::CaptureMountedFrameUnavailable(
            UiMountedInspectionOmission::FrameTransitionInFlight,
        )) => Ok(kind.awaiting(budget.awaiting_budget()?)),
        Err(denial) => Err(denial),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaptureRestartBudget {
    FreshAt(Instant),
    PreserveUntil(Instant),
}

impl CaptureRestartBudget {
    fn capture_deadline(self) -> Result<Instant, PlatformPulseVisualExecutionDenial> {
        match self {
            Self::FreshAt(admitted_at) => progression::capture_wall_deadline(admitted_at),
            Self::PreserveUntil(deadline) => Ok(deadline),
        }
    }

    fn readiness_deadline(self) -> Result<Instant, PlatformPulseVisualExecutionDenial> {
        match self {
            Self::FreshAt(observed_at) => replacement_frame_deadline(observed_at),
            Self::PreserveUntil(deadline) => Ok(deadline),
        }
    }

    fn awaiting_budget(
        self,
    ) -> Result<PlatformPulseAwaitingCaptureBudget, PlatformPulseVisualExecutionDenial> {
        Ok(match self {
            Self::FreshAt(_) => {
                PlatformPulseAwaitingCaptureBudget::fresh(self.readiness_deadline()?)
            }
            Self::PreserveUntil(deadline) => {
                PlatformPulseAwaitingCaptureBudget::preserved(deadline)
            }
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PlatformPulseAwaitingCaptureBudget {
    FreshAfterReadiness { readiness_deadline: Instant },
    PreserveUntil { capture_deadline: Instant },
}

impl PlatformPulseAwaitingCaptureBudget {
    pub(super) const fn fresh(readiness_deadline: Instant) -> Self {
        Self::FreshAfterReadiness { readiness_deadline }
    }

    pub(super) const fn preserved(capture_deadline: Instant) -> Self {
        Self::PreserveUntil { capture_deadline }
    }

    pub(super) const fn readiness_deadline(self) -> Instant {
        match self {
            Self::FreshAfterReadiness { readiness_deadline } => readiness_deadline,
            Self::PreserveUntil { capture_deadline } => capture_deadline,
        }
    }

    pub(super) fn capture_deadline(
        self,
        admitted_at: Instant,
    ) -> Result<Instant, PlatformPulseVisualExecutionDenial> {
        match self {
            Self::FreshAfterReadiness { .. } => progression::capture_wall_deadline(admitted_at),
            Self::PreserveUntil { capture_deadline } => Ok(capture_deadline),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::CaptureRestartBudget;

    #[test]
    fn stale_and_host_superseded_restarts_select_distinct_budget_authority() {
        let observed = Instant::now();
        let preserved = observed + Duration::from_millis(275);
        let fresh = CaptureRestartBudget::FreshAt(observed);
        let host_superseded = CaptureRestartBudget::PreserveUntil(preserved);

        assert_eq!(
            fresh.capture_deadline().expect("fresh capture budget"),
            observed + Duration::from_secs(5),
        );
        assert_eq!(
            fresh.readiness_deadline().expect("fresh readiness budget"),
            observed + Duration::from_secs(5),
        );
        assert_eq!(
            host_superseded
                .capture_deadline()
                .expect("preserved capture budget"),
            preserved,
        );
        assert_eq!(
            fresh
                .awaiting_budget()
                .expect("fresh awaiting budget")
                .capture_deadline(observed + Duration::from_secs(2))
                .expect("fresh budget at eventual admission"),
            observed + Duration::from_secs(7),
        );
        assert_eq!(
            host_superseded
                .awaiting_budget()
                .expect("preserved awaiting budget")
                .capture_deadline(observed + Duration::from_secs(2))
                .expect("preserved budget at eventual admission"),
            preserved,
        );
        assert_eq!(
            host_superseded
                .readiness_deadline()
                .expect("preserved readiness budget"),
            preserved,
        );
    }
}

#[derive(Clone, Copy)]
enum CaptureRestartKind {
    Initial,
    Rebase,
}

impl CaptureRestartKind {
    fn captured(
        self,
        capture: super::PlatformPulseVisualCapture,
    ) -> PlatformPulseVisualIdentityState {
        match self {
            Self::Initial => PlatformPulseVisualIdentityState::Capturing(capture),
            Self::Rebase => PlatformPulseVisualIdentityState::Rebasing(capture),
        }
    }

    fn awaiting(
        self,
        budget: PlatformPulseAwaitingCaptureBudget,
    ) -> PlatformPulseVisualIdentityState {
        match self {
            Self::Initial => PlatformPulseVisualIdentityState::AwaitingCapture { budget },
            Self::Rebase => PlatformPulseVisualIdentityState::AwaitingRebase { budget },
        }
    }
}
