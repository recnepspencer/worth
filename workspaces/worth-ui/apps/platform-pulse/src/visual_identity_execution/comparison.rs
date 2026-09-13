use std::time::Instant;

use worth_ui::facade::app::WorthUiNativeApplicationShell;
use worth_ui::facade::inspection::{
    UiUnbudgetedVisualSnapshotComparisonRequest, UiVisualCapturePoll,
    UiVisualComparisonPixelPolicy, UiVisualSnapshotComparisonBudget,
    UiVisualSnapshotComparisonOutcome,
};
use worth_ui::facade::rebind::UiRebindReceipt;

use crate::lifecycle_observation_publication::PlatformPulseObservationPublisher;

use super::progression::{
    begin_capture_before, resolve_capture, retire_snapshot, snapshot_matches_current_mounted_frame,
    PlatformPulseVisualCaptureResolution,
};
use super::{
    replacement_frame_deadline, PlatformPulseRetainedSnapshot, PlatformPulseVisualCapture,
    PlatformPulseVisualCapturePhase, PlatformPulseVisualExecutionDenial,
    PlatformPulseVisualIdentityState,
};

pub(super) struct PlatformPulseVisualComparisonCapture {
    capture: PlatformPulseVisualCapture,
    predecessor: PlatformPulseRetainedSnapshot,
    rebind: UiRebindReceipt,
}

pub(super) fn begin(
    predecessor: PlatformPulseRetainedSnapshot,
    rebind: UiRebindReceipt,
    capture: PlatformPulseVisualCapture,
) -> PlatformPulseVisualComparisonCapture {
    PlatformPulseVisualComparisonCapture {
        capture,
        predecessor,
        rebind,
    }
}

impl PlatformPulseVisualComparisonCapture {
    pub(super) fn cancel(self, shell: &mut WorthUiNativeApplicationShell) {
        shell.cancel_visual_snapshot(self.capture.pending);
        shell.dispose_visual_snapshot(self.predecessor.snapshot);
        drop(self.rebind);
    }

    pub(super) const fn deadline(&self) -> Instant {
        self.capture.deadline
    }

    pub(super) fn note_presentation_replacement(&mut self) {
        self.capture.replacement.note();
    }

    pub(super) const fn replacement_pending(&self) -> bool {
        self.capture.replacement.is_pending()
    }
}

pub(super) fn poll(
    mut pending: PlatformPulseVisualComparisonCapture,
    shell: &mut WorthUiNativeApplicationShell,
    publisher: &PlatformPulseObservationPublisher,
    tick: &mut u64,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    if now >= pending.capture.deadline {
        shell.cancel_visual_snapshot(pending.capture.pending);
        if pending.capture.replacement.is_pending() {
            drop(pending.rebind);
            return refresh_after_replacement(pending.predecessor, shell, now);
        }
        shell.dispose_visual_snapshot(pending.predecessor.snapshot);
        drop(pending.rebind);
        return Err(PlatformPulseVisualExecutionDenial::SnapshotDeadline(
            PlatformPulseVisualCapturePhase::Comparison,
        ));
    }
    match shell.poll_visual_snapshot(pending.capture.pending, *tick) {
        UiVisualCapturePoll::Pending(capture) => {
            pending.capture.pending = capture;
            Ok(PlatformPulseVisualIdentityState::Comparing(pending))
        }
        UiVisualCapturePoll::Completed(outcome) => {
            match resolve_capture(outcome, pending.capture.deadline)? {
                PlatformPulseVisualCaptureResolution::Captured(successor) => {
                    let successor_is_current =
                        snapshot_matches_current_mounted_frame(&successor, shell)?;
                    if pending.capture.replacement.is_pending() || !successor_is_current {
                        shell.dispose_visual_snapshot(successor);
                        drop(pending.rebind);
                        return refresh_after_replacement(pending.predecessor, shell, now);
                    }
                    publisher
                        .successor_visual_snapshot(&successor)
                        .map_err(PlatformPulseVisualExecutionDenial::Observation)?;
                    compare_and_dispose(
                        pending.predecessor,
                        successor,
                        pending.rebind,
                        shell,
                        publisher,
                    )
                }
                PlatformPulseVisualCaptureResolution::RetryBefore { deadline } => {
                    if pending.capture.replacement.is_pending() {
                        drop(pending.rebind);
                        return refresh_after_replacement(pending.predecessor, shell, now);
                    }
                    match begin_capture_before(shell, *tick, deadline) {
                        Ok(capture) => {
                            pending.capture = capture;
                            Ok(PlatformPulseVisualIdentityState::Comparing(pending))
                        }
                        Err(PlatformPulseVisualExecutionDenial::CaptureMountedFrameUnavailable(
                            worth_ui::facade::app::UiMountedInspectionOmission::FrameTransitionInFlight,
                        )) => Ok(PlatformPulseVisualIdentityState::AwaitingComparison {
                            predecessor: pending.predecessor,
                            rebind: pending.rebind,
                            budget: super::capture_restart::PlatformPulseAwaitingCaptureBudget::preserved(deadline),
                        }),
                        Err(denial) => Err(denial),
                    }
                }
            }
        }
    }
}

fn refresh_after_replacement(
    predecessor: PlatformPulseRetainedSnapshot,
    _shell: &WorthUiNativeApplicationShell,
    now: Instant,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    Ok(PlatformPulseVisualIdentityState::AwaitingRefresh {
        predecessor,
        budget: super::capture_restart::PlatformPulseAwaitingCaptureBudget::fresh(
            replacement_frame_deadline(now)?,
        ),
    })
}

fn compare_and_dispose(
    predecessor: PlatformPulseRetainedSnapshot,
    successor: worth_ui::facade::inspection::UiVisualSnapshotReceipt<
        worth_ui::facade::inspection::UiPixelsRequired,
    >,
    rebind: UiRebindReceipt,
    shell: &mut WorthUiNativeApplicationShell,
    publisher: &PlatformPulseObservationPublisher,
) -> Result<PlatformPulseVisualIdentityState, PlatformPulseVisualExecutionDenial> {
    let grant = shell.visual_inspection_authority().issue_comparison_grant();
    let budget = UiVisualSnapshotComparisonBudget::bounded(128)
        .expect("the fixed Platform Pulse comparison budget is nonzero");
    let request = UiUnbudgetedVisualSnapshotComparisonRequest::between(
        &predecessor.snapshot,
        &successor,
        &rebind,
    );
    let request = match predecessor.overlay_clear {
        Some(cleared) => request.through_cleared_predecessor_overlay(cleared),
        None => request,
    }
    .with_pixel_observation(UiVisualComparisonPixelPolicy::IfAlreadyRetained)
    .with_budget(budget);
    let comparison = match shell.compare_visual_snapshots(&grant, request) {
        UiVisualSnapshotComparisonOutcome::Compared(comparison) => comparison,
        UiVisualSnapshotComparisonOutcome::Omitted(omission) => {
            return Err(PlatformPulseVisualExecutionDenial::ComparisonOmitted(
                omission,
            ))
        }
        UiVisualSnapshotComparisonOutcome::Expired(expiry) => {
            return Err(PlatformPulseVisualExecutionDenial::ComparisonExpired(
                expiry,
            ))
        }
        UiVisualSnapshotComparisonOutcome::Incompatible(incompatibility) => {
            return Err(PlatformPulseVisualExecutionDenial::ComparisonIncompatible(
                incompatibility,
            ))
        }
        UiVisualSnapshotComparisonOutcome::Denied(denial) => {
            return Err(PlatformPulseVisualExecutionDenial::ComparisonDenied(denial))
        }
    };
    publisher
        .visual_comparison(comparison)
        .map_err(PlatformPulseVisualExecutionDenial::Observation)?;
    retire_snapshot(predecessor, shell, publisher)?;
    shell.dispose_visual_snapshot(successor);
    drop(rebind);
    Ok(PlatformPulseVisualIdentityState::Retired)
}
