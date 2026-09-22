//! The velocity a retarget carries out of the curve it interrupts.

use super::curve::{geometry_rate, UiMotionCurvePhase};
use crate::runtime::motion::UiMotionPropertyChannels;

/// Per-tick rate of change of the four geometry components at one sampled
/// tick. A retarget installs this as its successor's departure rate, so an
/// interruption mid-flight neither drops the content back to rest nor steps
/// its speed discontinuously.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct UiPresentationSampleVelocity([f32; 4]);

/// The still-running curve an interruption displaces, in the exact terms its
/// sampler was evaluating it at the interruption tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiPresentationOutgoingCurve {
    channels: UiMotionPropertyChannels,
    phase: UiMotionCurvePhase,
    start: [f32; 4],
    end: [f32; 4],
}

impl UiPresentationOutgoingCurve {
    pub(super) const fn interrupted_at(
        channels: UiMotionPropertyChannels,
        phase: UiMotionCurvePhase,
        start: [f32; 4],
        end: [f32; 4],
    ) -> Self {
        Self {
            channels,
            phase,
            start,
            end,
        }
    }
}

impl UiPresentationSampleVelocity {
    pub(super) const RESTING: Self = Self([0.0; 4]);

    pub(super) const fn components(self) -> [f32; 4] {
        self.0
    }

    /// Bound the cubic tangent by the stopping distance left by an accepted
    /// extent, so interpolation cannot leave the lawful endpoint interval.
    pub(super) fn within_extent(self, start: [f32; 4], end: [f32; 4], ticks: u32) -> Self {
        let mut velocity = self.0;
        for axis in 0..4 {
            let limit = 3.0 * (end[axis] - start[axis]) / ticks.max(1) as f32;
            velocity[axis] = velocity[axis].clamp(limit.min(0.0), limit.max(0.0));
        }
        Self(velocity)
    }

    /// Differentiate the displaced curve where the interruption caught it. A
    /// non-finite rate is no evidence of motion, so it settles to rest rather
    /// than poisoning the successor's geometry.
    pub(super) fn of_outgoing_curve(curve: UiPresentationOutgoingCurve) -> Self {
        let rate = geometry_rate(curve.channels, curve.start, curve.end, curve.phase);
        if rate.iter().any(|component| !component.is_finite()) {
            return Self::RESTING;
        }
        Self(rate)
    }
}
