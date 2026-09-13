use std::fmt;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseReplacementDenialFamily, PlatformPulseReplacementPreserved,
};

use crate::external_observation::{
    NativeClientPixelCapture, ProcessBoundNativeClientAreaObservation,
    StableProcessLivenessObservation,
};
use crate::source_delta::{
    AppliedPulseSourceDelta, MalformedPulseSourceDelta, PulseSourceDeltaIdentity,
};

use super::{
    adjudicate_native_color, ExecutablePublishedIdentity, ExpectedNativeColor, NativeColorFailure,
    NativeColorVerdict,
};

#[derive(Debug)]
pub(crate) struct ExecutablePredecessorPreservationEvidence {
    action: AppliedPulseSourceDelta<MalformedPulseSourceDelta>,
    envelope: PlatformPulseLifecycleObservationEnvelope,
    preserved: PlatformPulseReplacementPreserved,
    identity: ExecutablePublishedIdentity,
    liveness: StableProcessLivenessObservation,
    pixels: NativeClientPixelCapture,
    color: NativeColorVerdict,
}

pub(crate) struct CausalPredecessorPreservationObservationSet {
    action: AppliedPulseSourceDelta<MalformedPulseSourceDelta>,
    predecessor: ExecutablePublishedIdentity,
    predecessor_frame: u64,
    envelope: PlatformPulseLifecycleObservationEnvelope,
}

pub(crate) struct ExecutablePredecessorPreservationObservationSet {
    causal: CausalPredecessorPreservationObservationSet,
    client: ProcessBoundNativeClientAreaObservation,
    liveness: StableProcessLivenessObservation,
    pixels: NativeClientPixelCapture,
}

#[derive(Debug)]
pub(crate) enum ExecutablePredecessorPreservationFailure {
    WrongLifecycleOutcome,
    WrongRun,
    WrongSourceAction,
    SourceDidNotAdvance,
    SourceMeaningDidNotChange,
    WrongDenialFamily(PlatformPulseReplacementDenialFamily),
    ActiveGenerationChanged,
    ActiveFrameChanged,
    ProcessIdentityMismatch,
    NativeWindowIdentityMismatch,
    ClientCaptureSizeMismatch,
    NativeColor(NativeColorFailure),
}

pub(crate) fn adjudicate_predecessor_preservation(
    observations: ExecutablePredecessorPreservationObservationSet,
) -> Result<ExecutablePredecessorPreservationEvidence, ExecutablePredecessorPreservationFailure> {
    let ExecutablePredecessorPreservationObservationSet {
        causal,
        client,
        liveness,
        pixels,
    } = observations;
    let preserved = preserved_outcome(&causal.envelope)?;
    require_causal_preservation(&causal, preserved)?;
    require_same_external_world(&causal.predecessor, client, liveness, &pixels)?;
    let color = adjudicate_native_color(&pixels, ExpectedNativeColor::Green)
        .map_err(ExecutablePredecessorPreservationFailure::NativeColor)?;
    let identity = ExecutablePublishedIdentity::from_preservation(
        preserved,
        causal.envelope.run().value(),
        client,
    );
    Ok(ExecutablePredecessorPreservationEvidence {
        action: causal.action,
        envelope: causal.envelope,
        preserved,
        identity,
        liveness,
        pixels,
        color,
    })
}

impl CausalPredecessorPreservationObservationSet {
    pub(crate) fn new(
        action: AppliedPulseSourceDelta<MalformedPulseSourceDelta>,
        predecessor: ExecutablePublishedIdentity,
        predecessor_frame: u64,
        envelope: PlatformPulseLifecycleObservationEnvelope,
    ) -> Self {
        Self {
            action,
            predecessor,
            predecessor_frame,
            envelope,
        }
    }

    pub(crate) fn join_native(
        self,
        client: ProcessBoundNativeClientAreaObservation,
        liveness: StableProcessLivenessObservation,
        pixels: NativeClientPixelCapture,
    ) -> ExecutablePredecessorPreservationObservationSet {
        ExecutablePredecessorPreservationObservationSet {
            causal: self,
            client,
            liveness,
            pixels,
        }
    }
}

impl ExecutablePredecessorPreservationEvidence {
    pub(crate) fn action(&self) -> &AppliedPulseSourceDelta<MalformedPulseSourceDelta> {
        &self.action
    }

    pub(crate) fn preserved(&self) -> PlatformPulseReplacementPreserved {
        self.preserved
    }

    pub(crate) fn identity(&self) -> &ExecutablePublishedIdentity {
        &self.identity
    }

    pub(crate) fn liveness(&self) -> StableProcessLivenessObservation {
        self.liveness
    }

    pub(crate) fn matching_green_samples(&self) -> usize {
        self.color.matching_samples()
    }

    pub(crate) fn sampled_pixels(&self) -> usize {
        self.color.sampled_pixels()
    }

    pub(crate) fn expected_color(&self) -> ExpectedNativeColor {
        self.color.expected()
    }

    pub(crate) fn sequence(&self) -> u64 {
        self.envelope.sequence().value()
    }

    pub(crate) fn capture_count(&self) -> u32 {
        self.pixels.capture_count()
    }
}

fn preserved_outcome(
    envelope: &PlatformPulseLifecycleObservationEnvelope,
) -> Result<PlatformPulseReplacementPreserved, ExecutablePredecessorPreservationFailure> {
    match envelope.outcome() {
        PlatformPulseLifecycleObservation::RebindDeniedPreserving(preserved) => Ok(*preserved),
        _ => Err(ExecutablePredecessorPreservationFailure::WrongLifecycleOutcome),
    }
}

fn require_causal_preservation(
    causal: &CausalPredecessorPreservationObservationSet,
    preserved: PlatformPulseReplacementPreserved,
) -> Result<(), ExecutablePredecessorPreservationFailure> {
    if causal.envelope.run().value() != causal.predecessor.run() {
        return Err(ExecutablePredecessorPreservationFailure::WrongRun);
    }
    if causal.action.identity() != PulseSourceDeltaIdentity::Malformed {
        return Err(ExecutablePredecessorPreservationFailure::WrongSourceAction);
    }
    if preserved.source().source_sequence() <= causal.predecessor.source().source_sequence() {
        return Err(ExecutablePredecessorPreservationFailure::SourceDidNotAdvance);
    }
    if preserved.source().final_package_digest()
        == causal.predecessor.source().final_package_digest()
    {
        return Err(ExecutablePredecessorPreservationFailure::SourceMeaningDidNotChange);
    }
    if preserved.denial_family() != PlatformPulseReplacementDenialFamily::DslCompilation {
        return Err(ExecutablePredecessorPreservationFailure::WrongDenialFamily(
            preserved.denial_family(),
        ));
    }
    if preserved.active_generation() != causal.predecessor.generation() {
        return Err(ExecutablePredecessorPreservationFailure::ActiveGenerationChanged);
    }
    if preserved.active_frame().diagnostic_value() != causal.predecessor_frame {
        return Err(ExecutablePredecessorPreservationFailure::ActiveFrameChanged);
    }
    Ok(())
}

fn require_same_external_world(
    predecessor: &ExecutablePublishedIdentity,
    client: ProcessBoundNativeClientAreaObservation,
    liveness: StableProcessLivenessObservation,
    pixels: &NativeClientPixelCapture,
) -> Result<(), ExecutablePredecessorPreservationFailure> {
    if client.process_id() != predecessor.process_id()
        || liveness.process_id() != predecessor.process_id()
        || pixels.process_id() != predecessor.process_id()
    {
        return Err(ExecutablePredecessorPreservationFailure::ProcessIdentityMismatch);
    }
    if client.window() != predecessor.window() {
        return Err(ExecutablePredecessorPreservationFailure::NativeWindowIdentityMismatch);
    }
    let bounds = client.bounds();
    if pixels.width() != bounds.width() || pixels.height() != bounds.height() {
        return Err(ExecutablePredecessorPreservationFailure::ClientCaptureSizeMismatch);
    }
    Ok(())
}

impl fmt::Display for ExecutablePredecessorPreservationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLifecycleOutcome => {
                formatter.write_str("expected predecessor-preserving denial")
            }
            Self::WrongRun => formatter.write_str("preservation belongs to another process run"),
            Self::WrongSourceAction => {
                formatter.write_str("preservation followed the wrong source action")
            }
            Self::SourceDidNotAdvance => {
                formatter.write_str("denied source sequence did not advance")
            }
            Self::SourceMeaningDidNotChange => {
                formatter.write_str("malformed source meaning did not change")
            }
            Self::WrongDenialFamily(family) => {
                write!(formatter, "expected DSL denial, observed {family:?}")
            }
            Self::ActiveGenerationChanged => {
                formatter.write_str("denial changed the active generation")
            }
            Self::ActiveFrameChanged => formatter.write_str("denial changed the active frame"),
            Self::ProcessIdentityMismatch => {
                formatter.write_str("preservation observations identify different processes")
            }
            Self::NativeWindowIdentityMismatch => {
                formatter.write_str("preservation observations identify a different native window")
            }
            Self::ClientCaptureSizeMismatch => {
                formatter.write_str("preservation capture does not match the client area")
            }
            Self::NativeColor(failure) => write!(formatter, "preserved native color: {failure}"),
        }
    }
}
