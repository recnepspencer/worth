use std::fmt;
use std::time::Duration;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseFirstFramePublished, PlatformPulseLifecycleObservation,
    PlatformPulseLifecycleObservationEnvelope, PlatformPulseQueryProjectionEvidence,
    PlatformPulseQueryProjectionPosture, PlatformPulseQueryProjectionPublished,
};

use crate::external_observation::{
    NativeClientPixelCapture, ProcessBoundNativeClientAreaObservation,
    StableProcessLivenessObservation,
};

use super::{
    adjudicate_native_color, ExecutablePublishedIdentity, ExpectedNativeColor, NativeColorFailure,
    NativeColorVerdict,
};

#[derive(Debug)]
pub(crate) struct ExecutableFirstFrameEvidence {
    process_started: PlatformPulseLifecycleObservationEnvelope,
    pending_issued: PlatformPulseQueryProjectionEvidence,
    first_frame_envelope: PlatformPulseLifecycleObservationEnvelope,
    pending_published: PlatformPulseQueryProjectionPublished,
    first_frame: PlatformPulseFirstFramePublished,
    client_area: ProcessBoundNativeClientAreaObservation,
    liveness: StableProcessLivenessObservation,
    pixels: NativeClientPixelCapture,
    color: NativeColorVerdict,
}

pub(crate) struct CausalFirstFrameObservationSet {
    process_id: u32,
    process_started: PlatformPulseLifecycleObservationEnvelope,
    pending_issued: PlatformPulseLifecycleObservationEnvelope,
    first_frame_envelope: PlatformPulseLifecycleObservationEnvelope,
    pending_published: PlatformPulseLifecycleObservationEnvelope,
}

pub(crate) struct ExecutableFirstFrameObservationSet {
    causal: CausalFirstFrameObservationSet,
    client_area: ProcessBoundNativeClientAreaObservation,
    liveness: StableProcessLivenessObservation,
    pixels: NativeClientPixelCapture,
}

#[derive(Debug)]
pub(crate) enum ExecutableFirstFrameFailure {
    MissingProcessStart,
    MissingPendingIssue,
    MissingFirstFrame,
    MissingPendingPublication,
    PendingCorrelation,
    RunDoesNotIdentifyChild,
    MissingNativeEffect,
    ProcessIdentityMismatch,
    ClientCaptureSizeMismatch,
    LivenessHoldTooShort(Duration),
    NativeColor(NativeColorFailure),
    Appearance(super::FirstFrameAppearanceFailure),
}

impl fmt::Display for ExecutableFirstFrameFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingProcessStart => {
                formatter.write_str("first lifecycle event was not process start")
            }
            Self::MissingFirstFrame => {
                formatter.write_str("third lifecycle event was not first-frame publication")
            }
            Self::MissingPendingIssue => {
                formatter.write_str("second lifecycle event was not pending Query issue")
            }
            Self::MissingPendingPublication => {
                formatter.write_str("fourth lifecycle event was not pending Query publication")
            }
            Self::PendingCorrelation => {
                formatter.write_str("pending Query issue, mounted publication, and frame diverged")
            }
            Self::RunDoesNotIdentifyChild => {
                formatter.write_str("lifecycle run does not identify the launched child")
            }
            Self::MissingNativeEffect => {
                formatter.write_str("first-frame receipt reports no native effect")
            }
            Self::ProcessIdentityMismatch => formatter.write_str(
                "lifecycle, native window, liveness, and pixel observations do not identify one process",
            ),
            Self::ClientCaptureSizeMismatch => {
                formatter.write_str("captured pixels do not match the native client-area bounds")
            }
            Self::LivenessHoldTooShort(duration) => write!(
                formatter,
                "child liveness hold was too short: {} ms",
                duration.as_millis()
            ),
            Self::NativeColor(failure) => write!(formatter, "native color: {failure}"),
            Self::Appearance(failure) => write!(formatter, "appearance: {failure}"),
        }
    }
}

pub(crate) fn adjudicate_first_frame(
    observations: ExecutableFirstFrameObservationSet,
) -> Result<ExecutableFirstFrameEvidence, ExecutableFirstFrameFailure> {
    let ExecutableFirstFrameObservationSet {
        causal,
        client_area,
        liveness,
        pixels,
    } = observations;
    let (pending_issued, first_frame, pending_published) = require_causal_publication(&causal)?;
    require_native_effect(first_frame)?;
    require_one_process_identity(&causal, client_area, liveness, &pixels)?;
    require_client_capture_size(client_area, &pixels)?;
    require_stable_liveness(liveness)?;
    let color = adjudicate_native_color(&pixels, ExpectedNativeColor::Blue)
        .map_err(ExecutableFirstFrameFailure::NativeColor)?;
    super::adjudicate_first_frame_appearance(&pixels)
        .map_err(ExecutableFirstFrameFailure::Appearance)?;
    Ok(ExecutableFirstFrameEvidence {
        process_started: causal.process_started,
        pending_issued,
        first_frame_envelope: causal.first_frame_envelope,
        pending_published,
        first_frame,
        client_area,
        liveness,
        pixels,
        color,
    })
}

impl CausalFirstFrameObservationSet {
    pub(crate) fn new(
        process_id: u32,
        process_started: PlatformPulseLifecycleObservationEnvelope,
        pending_issued: PlatformPulseLifecycleObservationEnvelope,
        first_frame_envelope: PlatformPulseLifecycleObservationEnvelope,
        pending_published: PlatformPulseLifecycleObservationEnvelope,
    ) -> Self {
        Self {
            process_id,
            process_started,
            pending_issued,
            first_frame_envelope,
            pending_published,
        }
    }

    pub(crate) fn join_native(
        self,
        client_area: ProcessBoundNativeClientAreaObservation,
        liveness: StableProcessLivenessObservation,
        pixels: NativeClientPixelCapture,
    ) -> ExecutableFirstFrameObservationSet {
        ExecutableFirstFrameObservationSet {
            causal: self,
            client_area,
            liveness,
            pixels,
        }
    }
}

impl ExecutableFirstFrameEvidence {
    pub(crate) fn first_frame(&self) -> PlatformPulseFirstFramePublished {
        self.first_frame
    }

    pub(crate) fn client_area(&self) -> ProcessBoundNativeClientAreaObservation {
        self.client_area
    }

    pub(crate) fn liveness(&self) -> StableProcessLivenessObservation {
        self.liveness
    }

    pub(crate) fn matching_blue_samples(&self) -> usize {
        self.color.matching_samples()
    }

    pub(crate) fn sampled_pixels(&self) -> usize {
        self.color.sampled_pixels()
    }

    pub(crate) fn sequence_quad(&self) -> (u64, u64, u64, u64) {
        (
            self.process_started.sequence().value(),
            self.pending_issued.owner_order(),
            self.first_frame_envelope.sequence().value(),
            self.pending_published.projection().owner_order(),
        )
    }

    pub(crate) fn pending_projection(&self) -> &PlatformPulseQueryProjectionEvidence {
        &self.pending_issued
    }

    pub(crate) fn capture_count(&self) -> u32 {
        self.pixels.capture_count()
    }

    pub(crate) fn pixels(&self) -> &NativeClientPixelCapture {
        &self.pixels
    }

    pub(crate) fn run_identity(&self) -> &str {
        self.first_frame_envelope.run().value()
    }

    pub(crate) fn published_identity(&self) -> ExecutablePublishedIdentity {
        ExecutablePublishedIdentity::from_first_frame(
            self.first_frame,
            self.run_identity(),
            self.client_area,
        )
    }
}

fn require_causal_publication(
    causal: &CausalFirstFrameObservationSet,
) -> Result<
    (
        PlatformPulseQueryProjectionEvidence,
        PlatformPulseFirstFramePublished,
        PlatformPulseQueryProjectionPublished,
    ),
    ExecutableFirstFrameFailure,
> {
    if !matches!(
        causal.process_started.outcome(),
        PlatformPulseLifecycleObservation::ProcessStarted(_)
    ) {
        return Err(ExecutableFirstFrameFailure::MissingProcessStart);
    }
    let pending_issued = match causal.pending_issued.outcome() {
        PlatformPulseLifecycleObservation::QueryProjectionIssued(projection) => projection.clone(),
        _ => return Err(ExecutableFirstFrameFailure::MissingPendingIssue),
    };
    let first_frame = match causal.first_frame_envelope.outcome() {
        PlatformPulseLifecycleObservation::FirstFramePublished(first_frame) => *first_frame,
        _ => return Err(ExecutableFirstFrameFailure::MissingFirstFrame),
    };
    let pending_published = match causal.pending_published.outcome() {
        PlatformPulseLifecycleObservation::QueryProjectionPublished(projection) => {
            projection.clone()
        }
        _ => return Err(ExecutableFirstFrameFailure::MissingPendingPublication),
    };
    let expected_run_prefix = format!("{:08x}-", causal.process_id);
    if !causal
        .process_started
        .run()
        .value()
        .starts_with(&expected_run_prefix)
        || [
            &causal.pending_issued,
            &causal.first_frame_envelope,
            &causal.pending_published,
        ]
        .iter()
        .any(|envelope| envelope.run().value() != causal.process_started.run().value())
    {
        return Err(ExecutableFirstFrameFailure::RunDoesNotIdentifyChild);
    }
    if pending_issued.posture() != PlatformPulseQueryProjectionPosture::Pending
        || pending_issued.native_value().is_some()
        || pending_published.projection() != &pending_issued
        || pending_published.frame() != first_frame.frame()
    {
        return Err(ExecutableFirstFrameFailure::PendingCorrelation);
    }
    Ok((pending_issued, first_frame, pending_published))
}

fn require_native_effect(
    first_frame: PlatformPulseFirstFramePublished,
) -> Result<(), ExecutableFirstFrameFailure> {
    if first_frame.actual_native_effect_count() == 0 {
        Err(ExecutableFirstFrameFailure::MissingNativeEffect)
    } else {
        Ok(())
    }
}

fn require_one_process_identity(
    causal: &CausalFirstFrameObservationSet,
    client_area: ProcessBoundNativeClientAreaObservation,
    liveness: StableProcessLivenessObservation,
    pixels: &NativeClientPixelCapture,
) -> Result<(), ExecutableFirstFrameFailure> {
    if client_area.process_id() == causal.process_id
        && liveness.process_id() == causal.process_id
        && pixels.process_id() == causal.process_id
    {
        Ok(())
    } else {
        Err(ExecutableFirstFrameFailure::ProcessIdentityMismatch)
    }
}

fn require_client_capture_size(
    client_area: ProcessBoundNativeClientAreaObservation,
    pixels: &NativeClientPixelCapture,
) -> Result<(), ExecutableFirstFrameFailure> {
    let bounds = client_area.bounds();
    if pixels.width() == bounds.width() && pixels.height() == bounds.height() {
        Ok(())
    } else {
        Err(ExecutableFirstFrameFailure::ClientCaptureSizeMismatch)
    }
}

fn require_stable_liveness(
    liveness: StableProcessLivenessObservation,
) -> Result<(), ExecutableFirstFrameFailure> {
    if liveness.held_for() < Duration::from_millis(500) {
        Err(ExecutableFirstFrameFailure::LivenessHoldTooShort(
            liveness.held_for(),
        ))
    } else {
        Ok(())
    }
}
