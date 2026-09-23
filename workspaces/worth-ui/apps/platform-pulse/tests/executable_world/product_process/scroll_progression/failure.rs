use std::fmt;

use crate::adjudication::ScrollChromePixelFailure;
use crate::native_platform::NativePlatformFailure;
use crate::product_process::WatchedPulseObservationFailure;

#[derive(Debug)]
pub(crate) enum PlatformPulseScrollJourneyFailure {
    Native(NativePlatformFailure),
    Pixels(ScrollChromePixelFailure),
    Observation(WatchedPulseObservationFailure),
    UnexpectedObservation(String),
    InputDelivery(&'static str),
    /// The product reported that native input ingress stopped while the click
    /// was being delivered, so no routing outcome can follow.
    InputIngressStopped,
    /// A click landed on rows that moved, yet no hit-test witness was published.
    HitWitnessAbsent,
    HitTargetsIndistinct([u64; 3]),
    HitTargetUnchanged(u64),
    HitTarget {
        expected: u64,
        observed: u64,
    },
    PointOutsideCapture([u32; 2]),
    CaptureExport(String),
}

impl fmt::Display for PlatformPulseScrollJourneyFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(failure) => write!(formatter, "native platform: {failure:?}"),
            Self::Pixels(failure) => write!(formatter, "scroll pixels: {failure}"),
            Self::Observation(failure) => write!(formatter, "observation: {failure:?}"),
            Self::UnexpectedObservation(observation) => {
                write!(formatter, "unexpected observation: {observation}")
            }
            Self::InputDelivery(message) => write!(formatter, "input delivery: {message}"),
            Self::InputIngressStopped => {
                formatter.write_str("native input ingress stopped under the click")
            }
            Self::HitWitnessAbsent => {
                formatter.write_str("no unrouted hit-test witness followed the click")
            }
            Self::HitTargetsIndistinct(hits) => write!(
                formatter,
                "the three resting row dots did not hit-test to distinct nodes: {hits:?}"
            ),
            Self::HitTargetUnchanged(node) => write!(
                formatter,
                "the click after the move still hit node {node}: hit-testing ignored the displayed offset"
            ),
            Self::HitTarget { expected, observed } => write!(
                formatter,
                "the click after the move hit node {observed}, expected node {expected}"
            ),
            Self::PointOutsideCapture(point) => {
                write!(formatter, "point {point:?} lies outside the client capture")
            }
            Self::CaptureExport(failure) => write!(formatter, "capture export: {failure}"),
        }
    }
}
