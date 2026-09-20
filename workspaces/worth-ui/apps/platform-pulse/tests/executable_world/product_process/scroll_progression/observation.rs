use std::time::Instant;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseIntentInteractionFamily, PlatformPulseIntentRoutingStoppedObservation,
    PlatformPulseLifecycleObservation, PlatformPulseNativeInputIngressPosture,
};

use crate::adjudication::{
    adjudicate_content_shift, adjudicate_vertical_thumb, ContentShiftEvidence,
    VerticalThumbEvidence,
};
use crate::external_observation::{
    NativeClientPixelCapture, NativeClientPixelPoint, NativeInputProbeKind,
    PlatformPulseLifecycleStreamFailure,
};
use crate::native_platform::NativePlatformContract;
use crate::product_process::WatchedPulseObservationFailure;

use super::{
    NativeBoundExecutableWorld, PlatformPulseScrollJourneyFailure, LIFECYCLE_IDLE_SLICE,
    PIXEL_POLL_SLICE, TRANSITION_DEADLINE,
};

/// One settled state of the list after a move: the thumb and the content agree
/// with the same expected offset, and the capture that proved it.
pub(super) struct MovedContentEvidence {
    thumb: VerticalThumbEvidence,
    shift: ContentShiftEvidence,
    capture: NativeClientPixelCapture,
}

impl MovedContentEvidence {
    pub(super) const fn thumb(&self) -> VerticalThumbEvidence {
        self.thumb
    }

    pub(super) const fn shift(&self) -> ContentShiftEvidence {
        self.shift
    }

    pub(super) const fn capture(&self) -> &NativeClientPixelCapture {
        &self.capture
    }
}

pub(super) fn capture(
    world: &mut NativeBoundExecutableWorld,
) -> Result<NativeClientPixelCapture, PlatformPulseScrollJourneyFailure> {
    world
        .platform
        .capture_client_area(&world.native_client)
        .map_err(PlatformPulseScrollJourneyFailure::Native)
}

/// Poll until both the thumb and the content agree with `offset_points`.
pub(super) fn await_moved_content(
    world: &mut NativeBoundExecutableWorld,
    baseline: &NativeClientPixelCapture,
    dpi: u32,
    offset_points: f64,
) -> Result<MovedContentEvidence, PlatformPulseScrollJourneyFailure> {
    let deadline = Instant::now() + TRANSITION_DEADLINE;
    loop {
        let current = capture(world)?;
        let verdict = adjudicate_vertical_thumb(&current, dpi, offset_points).and_then(|thumb| {
            adjudicate_content_shift(baseline, &current, dpi, offset_points)
                .map(|shift| (thumb, shift))
        });
        match verdict {
            Ok((thumb, shift)) => {
                return Ok(MovedContentEvidence {
                    thumb,
                    shift,
                    capture: current,
                })
            }
            Err(failure) if Instant::now() >= deadline => {
                return Err(PlatformPulseScrollJourneyFailure::Pixels(failure))
            }
            Err(_) => std::thread::sleep(PIXEL_POLL_SLICE),
        }
    }
}

/// Click `point` and return the graph node the product hit-tested under it.
///
/// Nothing under the Recent activity rows routes Activate, so the product
/// reports the click as unrouted and names the node it landed on. That node is
/// the hit-test witness: it must change exactly as the content moves. The
/// product announces the first pointer button it ever receives once, before
/// routing it; that announcement is accepted only while ingress stays retained.
pub(super) fn await_unrouted_hit(
    world: &mut NativeBoundExecutableWorld,
    point: NativeClientPixelPoint,
) -> Result<u64, PlatformPulseScrollJourneyFailure> {
    let delivery = world
        .platform
        .deliver_pointer_activation(&world.native_client, point)
        .map_err(PlatformPulseScrollJourneyFailure::Native)?;
    if delivery.kind() != NativeInputProbeKind::Pointer
        || delivery.delivered_event_count() != 2
        || delivery.process_id() != world.process.id()
    {
        return Err(PlatformPulseScrollJourneyFailure::InputDelivery(
            "pointer click did not preserve its exact OS event sequence",
        ));
    }
    let deadline = Instant::now() + TRANSITION_DEADLINE;
    loop {
        match world.lifecycle.next(deadline) {
            Ok(envelope) if incidental_visual(envelope.outcome()) => {}
            Ok(envelope) => match envelope.outcome() {
                PlatformPulseLifecycleObservation::NativeInputReached(reached)
                    if reached.pointer_button_events() > 0 =>
                {
                    if reached.posture() != PlatformPulseNativeInputIngressPosture::Retained {
                        return Err(PlatformPulseScrollJourneyFailure::InputIngressStopped);
                    }
                }
                PlatformPulseLifecycleObservation::IntentRoutingStopped(
                    PlatformPulseIntentRoutingStoppedObservation::Unrouted {
                        graph_node,
                        interaction: PlatformPulseIntentInteractionFamily::Activate,
                    },
                ) if *graph_node != 0 => return Ok(*graph_node),
                outcome => return Err(unexpected(outcome)),
            },
            Err(PlatformPulseLifecycleStreamFailure::Deadline) => {
                return Err(PlatformPulseScrollJourneyFailure::HitWitnessAbsent)
            }
            Err(failure) => {
                return Err(PlatformPulseScrollJourneyFailure::Observation(
                    WatchedPulseObservationFailure::Lifecycle(failure),
                ))
            }
        }
    }
}

pub(super) fn drain_until_idle(
    world: &mut NativeBoundExecutableWorld,
) -> Result<u64, PlatformPulseScrollJourneyFailure> {
    loop {
        match world.lifecycle.next(Instant::now() + LIFECYCLE_IDLE_SLICE) {
            Ok(envelope) if incidental_visual(envelope.outcome()) => {}
            Ok(envelope) => return Err(unexpected(envelope.outcome())),
            Err(PlatformPulseLifecycleStreamFailure::Deadline) => {
                return Ok(world.lifecycle.measurement().accepted_events() as u64 + 1)
            }
            Err(failure) => {
                return Err(PlatformPulseScrollJourneyFailure::Observation(
                    WatchedPulseObservationFailure::Lifecycle(failure),
                ))
            }
        }
    }
}

pub(super) fn export_capture(
    name: &str,
    capture: &NativeClientPixelCapture,
) -> Result<(), PlatformPulseScrollJourneyFailure> {
    let Some(directory) = std::env::var_os("WORTH_UI_NATIVE_CAPTURE_DIRECTORY") else {
        return Ok(());
    };
    let path = std::path::PathBuf::from(directory).join(name);
    let export = || -> Result<(), String> {
        let parent = path
            .parent()
            .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
        let bytes = crate::failure_teardown::encode_native_capture_png(capture)
            .map_err(|error| format!("encode {}: {error}", path.display()))?;
        std::fs::write(&path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
    };
    export().map_err(PlatformPulseScrollJourneyFailure::CaptureExport)
}

fn incidental_visual(outcome: &PlatformPulseLifecycleObservation) -> bool {
    matches!(
        outcome,
        PlatformPulseLifecycleObservation::VisualSnapshotCaptured(_)
            | PlatformPulseLifecycleObservation::VisualPointTrace(_)
            | PlatformPulseLifecycleObservation::VisualOverlayPublished(_)
            | PlatformPulseLifecycleObservation::VisualOverlayCleared(_)
            | PlatformPulseLifecycleObservation::VisualSnapshotRetired(_)
            | PlatformPulseLifecycleObservation::VisualComparison(_)
    )
}

fn unexpected(outcome: &PlatformPulseLifecycleObservation) -> PlatformPulseScrollJourneyFailure {
    PlatformPulseScrollJourneyFailure::UnexpectedObservation(format!("{outcome:?}"))
}
