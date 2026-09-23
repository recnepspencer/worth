use std::fmt;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseQueryProjectionEvidence, PlatformPulseQueryProjectionPosture,
    PlatformPulseQueryProjectionPublished,
};

use crate::external_observation::{
    NativeClientPixelCapture, ProcessBoundNativeClientAreaObservation,
};

use super::{adjudicate_native_color, ExpectedNativeColor, NativeColorFailure, NativeColorVerdict};

#[derive(Debug)]
pub(crate) struct ExecutableQueryCurrentEvidence {
    issued_sequence: u64,
    published_sequence: u64,
    issued: PlatformPulseQueryProjectionEvidence,
    published: PlatformPulseQueryProjectionPublished,
    client: ProcessBoundNativeClientAreaObservation,
    pixels: NativeClientPixelCapture,
    background: NativeColorVerdict,
}

#[derive(Debug)]
pub(crate) enum ExecutableQueryCurrentFailure {
    UnexpectedIssue(String),
    MissingPublication,
    WrongValue,
    WrongOwnerOrder {
        expected: u64,
        observed: u64,
        issued_sequence: u64,
    },
    Correlation,
    ProcessIdentity,
    UnchangedPixels,
    Background(NativeColorFailure),
    WrappingText(super::PlatformPulseWrappingTextFailure),
}

impl fmt::Display for ExecutableQueryCurrentFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedIssue(observed) => write!(formatter, "expected Query current issue, observed {observed}"),
            Self::MissingPublication => formatter.write_str("missing mounted Query publication"),
            Self::WrongValue => formatter.write_str("Query current differs from frozen world input"),
            Self::WrongOwnerOrder {
                expected,
                observed,
                issued_sequence,
            } => write!(
                formatter,
                "Query owner order at lifecycle sequence {issued_sequence}: expected {expected}, observed {observed}"
            ),
            Self::Correlation => formatter.write_str("Query issue and mounted publication diverged"),
            Self::ProcessIdentity => formatter.write_str("Query pixels belong to another process"),
            Self::UnchangedPixels => formatter.write_str("Query value produced no native pixel change"),
            Self::Background(failure) => write!(formatter, "background control: {failure}"),
            Self::WrappingText(failure) => write!(formatter, "wrapping text: {failure:?}"),
        }
    }
}

pub(crate) fn adjudicate_query_current(
    issued: PlatformPulseLifecycleObservationEnvelope,
    published: PlatformPulseLifecycleObservationEnvelope,
    expected_value: &str,
    predecessor_owner_order: u64,
    client: ProcessBoundNativeClientAreaObservation,
    pixels: NativeClientPixelCapture,
    predecessor_pixels: &[u8],
) -> Result<ExecutableQueryCurrentEvidence, ExecutableQueryCurrentFailure> {
    let issued_sequence = issued.sequence().value();
    let published_sequence = published.sequence().value();
    let issued = match issued.outcome() {
        PlatformPulseLifecycleObservation::QueryProjectionIssued(evidence) => evidence.clone(),
        observed => {
            return Err(ExecutableQueryCurrentFailure::UnexpectedIssue(format!(
                "{observed:?}"
            )))
        }
    };
    let published = match published.outcome() {
        PlatformPulseLifecycleObservation::QueryProjectionPublished(evidence) => evidence.clone(),
        _ => return Err(ExecutableQueryCurrentFailure::MissingPublication),
    };
    if issued.posture() != PlatformPulseQueryProjectionPosture::Current
        || issued.native_value() != Some(expected_value)
    {
        return Err(ExecutableQueryCurrentFailure::WrongValue);
    }
    if issued.owner_order() <= predecessor_owner_order {
        return Err(ExecutableQueryCurrentFailure::WrongOwnerOrder {
            expected: predecessor_owner_order.saturating_add(1),
            observed: issued.owner_order(),
            issued_sequence,
        });
    }
    if published.projection() != &issued || published_sequence != issued_sequence.saturating_add(1)
    {
        return Err(ExecutableQueryCurrentFailure::Correlation);
    }
    if client.process_id() != pixels.process_id() {
        return Err(ExecutableQueryCurrentFailure::ProcessIdentity);
    }
    if pixels.rgba() == predecessor_pixels {
        return Err(ExecutableQueryCurrentFailure::UnchangedPixels);
    }
    let background = adjudicate_native_color(&pixels, ExpectedNativeColor::Blue)
        .map_err(ExecutableQueryCurrentFailure::Background)?;
    Ok(ExecutableQueryCurrentEvidence {
        issued_sequence,
        published_sequence,
        issued,
        published,
        client,
        pixels,
        background,
    })
}

impl ExecutableQueryCurrentEvidence {
    pub(crate) fn issued_sequence(&self) -> u64 {
        self.issued_sequence
    }

    pub(crate) fn published_sequence(&self) -> u64 {
        self.published_sequence
    }

    pub(crate) fn projection(&self) -> &PlatformPulseQueryProjectionEvidence {
        &self.issued
    }

    pub(crate) fn publication(&self) -> &PlatformPulseQueryProjectionPublished {
        &self.published
    }

    pub(crate) fn client(&self) -> ProcessBoundNativeClientAreaObservation {
        self.client
    }

    pub(crate) fn pixels(&self) -> &NativeClientPixelCapture {
        &self.pixels
    }

    pub(crate) fn matching_blue_samples(&self) -> usize {
        self.background.matching_samples()
    }

    pub(crate) fn sampled_pixels(&self) -> usize {
        self.background.sampled_pixels()
    }
}
