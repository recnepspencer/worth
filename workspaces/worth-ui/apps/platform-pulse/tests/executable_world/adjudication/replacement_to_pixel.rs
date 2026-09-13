use std::fmt;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseReplacementPublished, PlatformPulseTerminalFailureFamily,
};

use crate::external_observation::{
    NativeClientPixelCapture, ProcessBoundNativeClientAreaObservation,
    StableProcessLivenessObservation,
};
use crate::source_delta::{AppliedPulseSourceDelta, PulseSourceDeltaIdentity};

use super::{
    adjudicate_native_color, ExecutablePublishedIdentity, ExpectedNativeColor, NativeColorFailure,
    NativeColorVerdict,
};

#[derive(Debug)]
pub(crate) struct ExecutableReplacementEvidence<Kind> {
    action: AppliedPulseSourceDelta<Kind>,
    envelope: PlatformPulseLifecycleObservationEnvelope,
    replacement: PlatformPulseReplacementPublished,
    identity: ExecutablePublishedIdentity,
    liveness: StableProcessLivenessObservation,
    pixels: NativeClientPixelCapture,
    color: NativeColorVerdict,
}

pub(crate) struct CausalReplacementObservationSet<Kind> {
    action: AppliedPulseSourceDelta<Kind>,
    predecessor: ExecutablePublishedIdentity,
    envelope: PlatformPulseLifecycleObservationEnvelope,
    expectation: ReplacementExpectation,
}

pub(crate) struct ExecutableReplacementObservationSet<Kind> {
    causal: CausalReplacementObservationSet<Kind>,
    client: ProcessBoundNativeClientAreaObservation,
    liveness: StableProcessLivenessObservation,
    pixels: NativeClientPixelCapture,
    expected_color: ExpectedNativeColor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReplacementSourceExpectation {
    ChangedFromPredecessor,
    CanonicalDigest(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReplacementExpectation {
    action: PulseSourceDeltaIdentity,
    source: ReplacementSourceExpectation,
}

#[derive(Debug)]
pub(crate) enum ExecutableReplacementFailure {
    ProductDenied(PlatformPulseTerminalFailureFamily),
    WrongLifecycleOutcome,
    WrongRun,
    WrongSourceAction,
    SourceDidNotAdvance,
    SourceMeaningDidNotChange,
    CanonicalRecoveryMismatch,
    PredecessorGenerationMismatch,
    ActiveGenerationDidNotChange,
    SuccessorFrameDidNotChange,
    MissingNativeEffect,
    ProcessIdentityMismatch,
    NativeWindowIdentityMismatch,
    ClientCaptureSizeMismatch,
    NativeColor(NativeColorFailure),
}

pub(crate) fn adjudicate_replacement<Kind>(
    observations: ExecutableReplacementObservationSet<Kind>,
) -> Result<ExecutableReplacementEvidence<Kind>, ExecutableReplacementFailure> {
    let ExecutableReplacementObservationSet {
        causal,
        client,
        liveness,
        pixels,
        expected_color,
    } = observations;
    let replacement = replacement_outcome(&causal.envelope)?;
    require_causal_replacement(&causal, replacement)?;
    require_same_external_world(&causal.predecessor, client, liveness, &pixels)?;
    let color = adjudicate_native_color(&pixels, expected_color)
        .map_err(ExecutableReplacementFailure::NativeColor)?;
    let identity = ExecutablePublishedIdentity::from_replacement(
        replacement,
        causal.envelope.run().value(),
        client,
    );
    Ok(ExecutableReplacementEvidence {
        action: causal.action,
        envelope: causal.envelope,
        replacement,
        identity,
        liveness,
        pixels,
        color,
    })
}

impl<Kind> CausalReplacementObservationSet<Kind> {
    pub(crate) fn new(
        action: AppliedPulseSourceDelta<Kind>,
        predecessor: ExecutablePublishedIdentity,
        envelope: PlatformPulseLifecycleObservationEnvelope,
        expectation: ReplacementExpectation,
    ) -> Self {
        Self {
            action,
            predecessor,
            envelope,
            expectation,
        }
    }

    pub(crate) fn join_native(
        self,
        client: ProcessBoundNativeClientAreaObservation,
        liveness: StableProcessLivenessObservation,
        pixels: NativeClientPixelCapture,
        expected_color: ExpectedNativeColor,
    ) -> ExecutableReplacementObservationSet<Kind> {
        ExecutableReplacementObservationSet {
            causal: self,
            client,
            liveness,
            pixels,
            expected_color,
        }
    }
}

impl ReplacementExpectation {
    pub(crate) fn green_successor() -> Self {
        Self {
            action: PulseSourceDeltaIdentity::Green,
            source: ReplacementSourceExpectation::ChangedFromPredecessor,
        }
    }

    pub(crate) fn canonical_recovery(digest: u64) -> Self {
        Self {
            action: PulseSourceDeltaIdentity::CanonicalBlueRecovery,
            source: ReplacementSourceExpectation::CanonicalDigest(digest),
        }
    }

    pub(crate) fn revision_schema() -> Self {
        Self {
            action: PulseSourceDeltaIdentity::RevisionSchema,
            source: ReplacementSourceExpectation::ChangedFromPredecessor,
        }
    }

    pub(crate) fn status_schema_recovery(digest: u64) -> Self {
        Self {
            action: PulseSourceDeltaIdentity::StatusSchemaRecovery,
            source: ReplacementSourceExpectation::CanonicalDigest(digest),
        }
    }
}

impl<Kind> ExecutableReplacementEvidence<Kind> {
    pub(crate) fn action(&self) -> &AppliedPulseSourceDelta<Kind> {
        &self.action
    }

    pub(crate) fn replacement(&self) -> PlatformPulseReplacementPublished {
        self.replacement
    }

    pub(crate) fn identity(&self) -> &ExecutablePublishedIdentity {
        &self.identity
    }

    pub(crate) fn liveness(&self) -> StableProcessLivenessObservation {
        self.liveness
    }

    pub(crate) fn matching_color_samples(&self) -> usize {
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

    pub(crate) fn pixels(&self) -> &NativeClientPixelCapture {
        &self.pixels
    }
}

pub(crate) fn require_replacement_lifecycle(
    envelope: &PlatformPulseLifecycleObservationEnvelope,
) -> Result<(), ExecutableReplacementFailure> {
    replacement_outcome(envelope).map(|_| ())
}

fn replacement_outcome(
    envelope: &PlatformPulseLifecycleObservationEnvelope,
) -> Result<PlatformPulseReplacementPublished, ExecutableReplacementFailure> {
    match envelope.outcome() {
        PlatformPulseLifecycleObservation::RebindPublished(replacement) => Ok(*replacement),
        PlatformPulseLifecycleObservation::TerminalFailure(failure) => Err(
            ExecutableReplacementFailure::ProductDenied(failure.family()),
        ),
        _ => Err(ExecutableReplacementFailure::WrongLifecycleOutcome),
    }
}

fn require_causal_replacement<Kind>(
    causal: &CausalReplacementObservationSet<Kind>,
    replacement: PlatformPulseReplacementPublished,
) -> Result<(), ExecutableReplacementFailure> {
    if causal.envelope.run().value() != causal.predecessor.run() {
        return Err(ExecutableReplacementFailure::WrongRun);
    }
    if causal.action.identity() != causal.expectation.action {
        return Err(ExecutableReplacementFailure::WrongSourceAction);
    }
    if replacement.source().source_sequence() <= causal.predecessor.source().source_sequence() {
        return Err(ExecutableReplacementFailure::SourceDidNotAdvance);
    }
    match causal.expectation.source {
        ReplacementSourceExpectation::ChangedFromPredecessor
            if replacement.source().final_package_digest()
                == causal.predecessor.source().final_package_digest() =>
        {
            return Err(ExecutableReplacementFailure::SourceMeaningDidNotChange)
        }
        ReplacementSourceExpectation::CanonicalDigest(expected)
            if replacement.source().final_package_digest() != expected =>
        {
            return Err(ExecutableReplacementFailure::CanonicalRecoveryMismatch)
        }
        _ => {}
    }
    if replacement.predecessor_generation() != causal.predecessor.generation() {
        return Err(ExecutableReplacementFailure::PredecessorGenerationMismatch);
    }
    if replacement.active_generation() == causal.predecessor.generation() {
        return Err(ExecutableReplacementFailure::ActiveGenerationDidNotChange);
    }
    if replacement.successor_frame() == causal.predecessor.frame() {
        return Err(ExecutableReplacementFailure::SuccessorFrameDidNotChange);
    }
    if replacement.actual_native_effect_count() == 0 {
        return Err(ExecutableReplacementFailure::MissingNativeEffect);
    }
    Ok(())
}

fn require_same_external_world(
    predecessor: &ExecutablePublishedIdentity,
    client: ProcessBoundNativeClientAreaObservation,
    liveness: StableProcessLivenessObservation,
    pixels: &NativeClientPixelCapture,
) -> Result<(), ExecutableReplacementFailure> {
    if client.process_id() != predecessor.process_id()
        || liveness.process_id() != predecessor.process_id()
        || pixels.process_id() != predecessor.process_id()
    {
        return Err(ExecutableReplacementFailure::ProcessIdentityMismatch);
    }
    if client.window() != predecessor.window() {
        return Err(ExecutableReplacementFailure::NativeWindowIdentityMismatch);
    }
    let bounds = client.bounds();
    if pixels.width() != bounds.width() || pixels.height() != bounds.height() {
        return Err(ExecutableReplacementFailure::ClientCaptureSizeMismatch);
    }
    Ok(())
}

impl fmt::Display for ExecutableReplacementFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProductDenied(family) => {
                write!(formatter, "product denied replacement: {family:?}")
            }
            Self::WrongLifecycleOutcome => formatter.write_str("expected replacement publication"),
            Self::WrongRun => formatter.write_str("replacement belongs to another process run"),
            Self::WrongSourceAction => {
                formatter.write_str("replacement followed the wrong source action")
            }
            Self::SourceDidNotAdvance => {
                formatter.write_str("replacement source sequence did not advance")
            }
            Self::SourceMeaningDidNotChange => {
                formatter.write_str("replacement source meaning did not change")
            }
            Self::CanonicalRecoveryMismatch => {
                formatter.write_str("recovery did not restore canonical source meaning")
            }
            Self::PredecessorGenerationMismatch => {
                formatter.write_str("replacement named the wrong predecessor generation")
            }
            Self::ActiveGenerationDidNotChange => {
                formatter.write_str("replacement kept the predecessor generation")
            }
            Self::SuccessorFrameDidNotChange => {
                formatter.write_str("replacement kept the predecessor frame")
            }
            Self::MissingNativeEffect => {
                formatter.write_str("replacement reported no native effect")
            }
            Self::ProcessIdentityMismatch => {
                formatter.write_str("replacement observations identify different processes")
            }
            Self::NativeWindowIdentityMismatch => {
                formatter.write_str("replacement observations identify a different native window")
            }
            Self::ClientCaptureSizeMismatch => {
                formatter.write_str("replacement capture does not match the client area")
            }
            Self::NativeColor(failure) => write!(formatter, "replacement native color: {failure}"),
        }
    }
}
