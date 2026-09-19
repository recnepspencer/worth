use std::time::{Duration, Instant};

use worth_ui::facade::app::{
    UiMountedInspectionReceipt, UiMountedInspectionRequest, WorthUiNativeApplicationShell,
};
use worth_ui::facade::inspection::{
    UiPixelsRequired, UiVisualCaptureDeadline, UiVisualCapturePoll, UiVisualSnapshotOutcome,
    UiVisualSnapshotReceipt,
};
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseVisualPointObservation, PlatformPulseVisualPointTraceInput,
};

use crate::lifecycle_observation_publication::PlatformPulseObservationPublisher;
use crate::visual_identity_adjudication::adjudicate_points;

pub(super) use super::frame_affinity::snapshot_matches_current_mounted_frame;
use super::{
    capture_restart, comparison, PlatformPulseRetainedSnapshot, PlatformPulseVisibleOverlay,
    PlatformPulseVisualCapture, PlatformPulseVisualCapturePhase,
    PlatformPulseVisualExecutionDenial, PlatformPulseVisualIdentityState,
    PlatformPulseVisualRefreshCapture,
};

#[path = "progression/retirement.rs"]
mod retirement;
pub(super) use retirement::retire_snapshot;
use retirement::{next_presentation_ticks, retain_refreshed_snapshot, retire_refresh_predecessor};

const CAPTURE_WALL_DEADLINE: Duration = Duration::from_secs(5);
const OVERLAY_VISIBLE_DWELL: Duration = Duration::from_secs(2);
const CAPTURE_TICK_ALLOWANCE: u64 = 1_000;

pub(super) enum PlatformPulseVisualCaptureResolution {
    Captured(UiVisualSnapshotReceipt<UiPixelsRequired>),
    RetryBefore { deadline: Instant },
}
pub(super) fn advance_state(
    state: PlatformPulseVisualIdentityState,
    shell: &mut WorthUiNativeApplicationShell,
    publisher: &PlatformPulseObservationPublisher,
    tick: &mut u64,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    match state {
        PlatformPulseVisualIdentityState::Settling { deadline, .. }
            if super::state_progression::initial_readiness_expired(now, deadline) =>
        {
            Err(PlatformPulseVisualExecutionDenial::SnapshotDeadline(
                PlatformPulseVisualCapturePhase::InitialReadiness,
            ))
        }
        PlatformPulseVisualIdentityState::Settling { begin_at, deadline } if now >= begin_at => {
            match begin_capture(shell, *tick, now) {
                Ok(capture) => Ok(PlatformPulseVisualIdentityState::Capturing(capture)),
                Err(PlatformPulseVisualExecutionDenial::CaptureMountedFrameUnavailable(
                    worth_ui::facade::app::UiMountedInspectionOmission::FrameTransitionInFlight,
                )) if now < deadline => Ok(PlatformPulseVisualIdentityState::Settling {
                    begin_at: now
                        .checked_add(Duration::from_millis(1))
                        .map_or(deadline, |poll| poll.min(deadline)),
                    deadline,
                }),
                Err(denial) => Err(denial),
            }
        }
        PlatformPulseVisualIdentityState::Capturing(capture) => {
            poll_capture(capture, shell, publisher, tick, now)
        }
        PlatformPulseVisualIdentityState::Comparing(comparison) => {
            comparison::poll(comparison, shell, publisher, tick, now)
        }
        PlatformPulseVisualIdentityState::Rebasing(capture) => {
            poll_rebase(capture, shell, publisher, tick, now)
        }
        PlatformPulseVisualIdentityState::Refreshing(refresh) => {
            poll_refresh(refresh, shell, publisher, tick, now)
        }
        PlatformPulseVisualIdentityState::OverlayVisible(overlay) if now >= overlay.clear_at => {
            clear_overlay(overlay, shell, publisher, tick, now)
        }
        PlatformPulseVisualIdentityState::Transitioning => {
            Ok(PlatformPulseVisualIdentityState::Transitioning)
        }
        state => Ok(state),
    }
}

pub(super) fn begin_capture(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    now: Instant,
) -> Result<PlatformPulseVisualCapture, PlatformPulseVisualExecutionDenial> {
    let deadline = capture_wall_deadline(now)?;
    begin_capture_before(shell, tick, deadline)
}
pub(super) fn capture_wall_deadline(
    admitted_at: Instant,
) -> Result<Instant, PlatformPulseVisualExecutionDenial> {
    admitted_at
        .checked_add(CAPTURE_WALL_DEADLINE)
        .ok_or(PlatformPulseVisualExecutionDenial::ClockOverflow)
}

pub(super) fn begin_capture_before(
    shell: &mut WorthUiNativeApplicationShell,
    tick: u64,
    deadline: Instant,
) -> Result<PlatformPulseVisualCapture, PlatformPulseVisualExecutionDenial> {
    let frame = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => frame,
        UiMountedInspectionReceipt::Omitted(omission) => {
            return Err(
                PlatformPulseVisualExecutionDenial::CaptureMountedFrameUnavailable(omission),
            )
        }
    };
    let target = frame
        .current_visual_target()
        .map_err(|_| PlatformPulseVisualExecutionDenial::MountedVisualTarget)?;
    let grant = shell.visual_inspection_authority().issue_pixel_grant();
    let deadline_tick = tick
        .checked_add(CAPTURE_TICK_ALLOWANCE)
        .ok_or(PlatformPulseVisualExecutionDenial::TickExhausted)?;
    let request = worth_ui::facade::inspection::UiVisualSnapshotRequest::for_local_development_unredacted_frame(target)
        .artifacts(UiPixelsRequired::policy())
        .deadline(UiVisualCaptureDeadline::at_tick(deadline_tick));
    let pending = shell
        .begin_visual_pixel_snapshot(&grant, request)
        .map_err(PlatformPulseVisualExecutionDenial::SnapshotAdmission)?;
    Ok(PlatformPulseVisualCapture {
        pending,
        deadline,
        replacement: super::replacement::PlatformPulseReplacementPosture::default(),
    })
}

fn poll_capture(
    capture: PlatformPulseVisualCapture,
    shell: &mut WorthUiNativeApplicationShell,
    publisher: &PlatformPulseObservationPublisher,
    tick: &mut u64,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    if now >= capture.deadline {
        shell.cancel_visual_snapshot(capture.pending);
        if capture.replacement.is_pending() {
            return capture_restart::initial_after_stale_frame(shell, *tick, now);
        }
        return Err(PlatformPulseVisualExecutionDenial::SnapshotDeadline(
            PlatformPulseVisualCapturePhase::Initial,
        ));
    }
    match shell.poll_visual_snapshot(capture.pending, *tick) {
        UiVisualCapturePoll::Pending(pending) => Ok(PlatformPulseVisualIdentityState::Capturing(
            PlatformPulseVisualCapture {
                pending,
                deadline: capture.deadline,
                replacement: capture.replacement,
            },
        )),
        UiVisualCapturePoll::Completed(outcome) => {
            match resolve_capture(outcome, capture.deadline)? {
                PlatformPulseVisualCaptureResolution::Captured(receipt) => {
                    if !snapshot_matches_current_mounted_frame(&receipt, shell)? {
                        shell.dispose_visual_snapshot(receipt);
                        return capture_restart::initial_after_stale_frame(shell, *tick, now);
                    }
                    publish_overlay(receipt, shell, publisher, tick, now)
                }
                PlatformPulseVisualCaptureResolution::RetryBefore { deadline } => {
                    if capture.replacement.is_pending() {
                        capture_restart::initial_after_stale_frame(shell, *tick, now)
                    } else {
                        capture_restart::initial_after_host_supersession(shell, *tick, deadline)
                    }
                }
            }
        }
    }
}

pub(super) fn resolve_capture(
    outcome: UiVisualSnapshotOutcome<UiPixelsRequired>,
    deadline: Instant,
) -> Result<PlatformPulseVisualCaptureResolution, PlatformPulseVisualExecutionDenial> {
    match outcome {
        UiVisualSnapshotOutcome::Captured(receipt) => {
            Ok(PlatformPulseVisualCaptureResolution::Captured(receipt))
        }
        UiVisualSnapshotOutcome::Superseded(_) => {
            Ok(PlatformPulseVisualCaptureResolution::RetryBefore { deadline })
        }
        UiVisualSnapshotOutcome::Omitted(_) => {
            Err(PlatformPulseVisualExecutionDenial::SnapshotOmitted)
        }
        UiVisualSnapshotOutcome::Denied(denial) => {
            Err(PlatformPulseVisualExecutionDenial::SnapshotDenied(denial))
        }
        UiVisualSnapshotOutcome::Indeterminate(_) => {
            Err(PlatformPulseVisualExecutionDenial::SnapshotIndeterminate)
        }
    }
}

fn publish_overlay(
    receipt: UiVisualSnapshotReceipt<UiPixelsRequired>,
    shell: &mut WorthUiNativeApplicationShell,
    publisher: &PlatformPulseObservationPublisher,
    tick: &mut u64,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    publisher
        .visual_snapshot(&receipt)
        .map_err(PlatformPulseVisualExecutionDenial::Observation)?;
    let points = adjudicate_points(&receipt)?;
    publisher
        .visual_point_trace(PlatformPulseVisualPointTraceInput::new(
            &receipt,
            PlatformPulseVisualPointObservation::new(points.target_point, &points.target),
            PlatformPulseVisualPointObservation::new(points.background_point, &points.background),
        ))
        .map_err(PlatformPulseVisualExecutionDenial::Observation)?;
    let target = receipt
        .overlay_target(&points.selected_target)
        .map_err(PlatformPulseVisualExecutionDenial::OverlayTarget)?;
    let grant = shell.visual_inspection_authority().issue_overlay_grant();
    let pending = shell
        .show_identity_overlay(&grant, target)
        .map_err(PlatformPulseVisualExecutionDenial::OverlayAdmission)?;
    let (deadline, current) = next_presentation_ticks(tick)?;
    let published = shell
        .present_visual_overlay(pending, deadline, current)
        .map_err(|failure| {
            PlatformPulseVisualExecutionDenial::OverlayPublication(failure.denial())
        })?;
    publisher
        .visual_overlay_published(&published)
        .map_err(PlatformPulseVisualExecutionDenial::Observation)?;
    let clear_at = now
        .checked_add(OVERLAY_VISIBLE_DWELL)
        .ok_or(PlatformPulseVisualExecutionDenial::ClockOverflow)?;
    Ok(PlatformPulseVisualIdentityState::OverlayVisible(
        PlatformPulseVisibleOverlay {
            retained: PlatformPulseRetainedSnapshot {
                snapshot: receipt,
                overlay_clear: None,
            },
            published,
            clear_at,
            replacement: super::replacement::PlatformPulseReplacementPosture::default(),
        },
    ))
}

fn clear_overlay(
    overlay: PlatformPulseVisibleOverlay,
    shell: &mut WorthUiNativeApplicationShell,
    publisher: &PlatformPulseObservationPublisher,
    tick: &mut u64,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    let (deadline, current) = next_presentation_ticks(tick)?;
    let cleared = shell
        .clear_visual_overlay(overlay.published, deadline, current)
        .map_err(|failure| PlatformPulseVisualExecutionDenial::OverlayClear(failure.denial()))?;
    publisher
        .visual_overlay_cleared(cleared)
        .map_err(PlatformPulseVisualExecutionDenial::Observation)?;
    let mut retained = overlay.retained;
    retained.overlay_clear = Some(cleared);
    if overlay.replacement.is_pending() {
        Ok(PlatformPulseVisualIdentityState::AwaitingRefresh {
            predecessor: retained,
            budget: super::capture_restart::PlatformPulseAwaitingCaptureBudget::fresh(
                super::replacement_frame_deadline(now)?,
            ),
        })
    } else {
        Ok(PlatformPulseVisualIdentityState::ComparisonReady(retained))
    }
}

fn poll_refresh(
    mut refresh: PlatformPulseVisualRefreshCapture,
    shell: &mut WorthUiNativeApplicationShell,
    publisher: &PlatformPulseObservationPublisher,
    tick: &mut u64,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    let capture_deadline = refresh.capture.deadline;
    if now >= capture_deadline {
        shell.cancel_visual_snapshot(refresh.capture.pending);
        if refresh.capture.replacement.is_pending() {
            return capture_restart::refresh_after_stale_frame(
                shell,
                *tick,
                refresh.predecessor,
                now,
            );
        }
        shell.dispose_visual_snapshot(refresh.predecessor.snapshot);
        return Err(PlatformPulseVisualExecutionDenial::SnapshotDeadline(
            PlatformPulseVisualCapturePhase::Refresh,
        ));
    }
    match shell.poll_visual_snapshot(refresh.capture.pending, *tick) {
        UiVisualCapturePoll::Pending(pending) => {
            refresh.capture.pending = pending;
            Ok(PlatformPulseVisualIdentityState::Refreshing(refresh))
        }
        UiVisualCapturePoll::Completed(outcome) => {
            match resolve_capture(outcome, capture_deadline)? {
                PlatformPulseVisualCaptureResolution::Captured(successor) => {
                    if !snapshot_matches_current_mounted_frame(&successor, shell)? {
                        shell.dispose_visual_snapshot(successor);
                        return capture_restart::refresh_after_stale_frame(
                            shell,
                            *tick,
                            refresh.predecessor,
                            now,
                        );
                    }
                    retire_refresh_predecessor(refresh.predecessor, &successor, shell, publisher)?;
                    retain_refreshed_snapshot(successor, publisher)
                }
                PlatformPulseVisualCaptureResolution::RetryBefore { deadline } => {
                    if refresh.capture.replacement.is_pending() {
                        capture_restart::refresh_after_stale_frame(
                            shell,
                            *tick,
                            refresh.predecessor,
                            now,
                        )
                    } else {
                        capture_restart::refresh_after_host_supersession(
                            shell,
                            *tick,
                            refresh.predecessor,
                            deadline,
                        )
                    }
                }
            }
        }
    }
}

fn poll_rebase(
    capture: PlatformPulseVisualCapture,
    shell: &mut WorthUiNativeApplicationShell,
    publisher: &PlatformPulseObservationPublisher,
    tick: &mut u64,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    if now >= capture.deadline {
        shell.cancel_visual_snapshot(capture.pending);
        if capture.replacement.is_pending() {
            return capture_restart::rebase_after_stale_frame(shell, *tick, now);
        }
        return Err(PlatformPulseVisualExecutionDenial::SnapshotDeadline(
            PlatformPulseVisualCapturePhase::Rebase,
        ));
    }
    match shell.poll_visual_snapshot(capture.pending, *tick) {
        UiVisualCapturePoll::Pending(pending) => Ok(PlatformPulseVisualIdentityState::Rebasing(
            PlatformPulseVisualCapture {
                pending,
                deadline: capture.deadline,
                replacement: capture.replacement,
            },
        )),
        UiVisualCapturePoll::Completed(outcome) => {
            match resolve_capture(outcome, capture.deadline)? {
                PlatformPulseVisualCaptureResolution::Captured(receipt) => {
                    if !snapshot_matches_current_mounted_frame(&receipt, shell)? {
                        shell.dispose_visual_snapshot(receipt);
                        return capture_restart::rebase_after_stale_frame(shell, *tick, now);
                    }
                    retain_refreshed_snapshot(receipt, publisher)
                }
                PlatformPulseVisualCaptureResolution::RetryBefore { deadline } => {
                    if capture.replacement.is_pending() {
                        capture_restart::rebase_after_stale_frame(shell, *tick, now)
                    } else {
                        capture_restart::rebase_after_host_supersession(shell, *tick, deadline)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "progression/tests.rs"]
mod tests;
