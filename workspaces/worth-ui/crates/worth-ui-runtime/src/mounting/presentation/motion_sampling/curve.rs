//! Evaluation of the declared curve families: where each animated geometry
//! component sits at a phase of its horizon, and how fast it is moving there.

use crate::runtime::motion::{UiMotionEasing, UiMotionPropertyChannel, UiMotionPropertyChannels};

/// One sampled point on a declared curve, carrying everything both its
/// position and its derivative need: which family, how far along its horizon,
/// how long that horizon is, and the rate the curve started at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiMotionCurvePhase {
    easing: UiMotionEasing,
    normalized: f32,
    duration_ticks: f32,
    start_velocity: [f32; 4],
}

impl UiMotionCurvePhase {
    pub(super) fn new(
        easing: UiMotionEasing,
        normalized: f32,
        duration_ticks: u32,
        start_velocity: [f32; 4],
    ) -> Self {
        Self {
            easing,
            normalized: normalized.clamp(0.0, 1.0),
            duration_ticks: duration_ticks.max(1) as f32,
            start_velocity,
        }
    }

    /// The position of one component whose curve runs from `start` to `end`.
    fn place(self, index: usize, start: f32, end: f32) -> f32 {
        match self.easing {
            UiMotionEasing::EaseOutCubic => start + (end - start) * ease_out_cubic(self.normalized),
            UiMotionEasing::VelocityMatchedCubic => {
                velocity_matched_cubic(start, self.velocity_of(index), end, self)
            }
        }
    }

    /// The per-tick rate of one component whose curve runs from `start` to
    /// `end`. Both declared families arrive at rest, so this is zero once the
    /// horizon is spent.
    fn rate(self, index: usize, start: f32, end: f32) -> f32 {
        match self.easing {
            UiMotionEasing::EaseOutCubic => {
                (end - start) * 3.0 * (1.0 - self.normalized).powi(2) / self.duration_ticks
            }
            UiMotionEasing::VelocityMatchedCubic => {
                velocity_matched_cubic_rate(start, self.velocity_of(index), end, self)
            }
        }
    }

    /// The family's progress for a channel that carries no velocity of its
    /// own, such as opacity: the same declared curve departing from rest.
    pub(super) fn scalar_progress(self) -> f32 {
        match self.easing {
            UiMotionEasing::EaseOutCubic => ease_out_cubic(self.normalized),
            UiMotionEasing::VelocityMatchedCubic => velocity_matched_cubic(0.0, 0.0, 1.0, self),
        }
    }

    const fn velocity_of(self, index: usize) -> f32 {
        self.start_velocity[index]
    }
}

/// Cubic Hermite from `start` moving at `start_velocity` per tick to `end` at
/// rest over the phase's horizon.
fn velocity_matched_cubic(
    start: f32,
    start_velocity: f32,
    end: f32,
    phase: UiMotionCurvePhase,
) -> f32 {
    let s = phase.normalized;
    let position_start = 2.0 * s * s * s - 3.0 * s * s + 1.0;
    let position_end = -2.0 * s * s * s + 3.0 * s * s;
    let departure_rate = s * s * s - 2.0 * s * s + s;
    position_start * start
        + position_end * end
        + phase.duration_ticks * start_velocity * departure_rate
}

/// The per-tick derivative of [`velocity_matched_cubic`] at the same phase.
fn velocity_matched_cubic_rate(
    start: f32,
    start_velocity: f32,
    end: f32,
    phase: UiMotionCurvePhase,
) -> f32 {
    let s = phase.normalized;
    (6.0 * s * s - 6.0 * s) * (start - end) / phase.duration_ticks
        + start_velocity * (3.0 * s * s - 4.0 * s + 1.0)
}

fn ease_out_cubic(progress: f32) -> f32 {
    1.0 - (1.0 - progress).powi(3)
}

pub(super) fn interpolate_geometry(
    channels: UiMotionPropertyChannels,
    start: Option<[f32; 4]>,
    end: Option<[f32; 4]>,
    phase: UiMotionCurvePhase,
) -> Option<[f32; 4]> {
    let (Some(start), Some(end)) = (start, end) else {
        return end.or(start);
    };
    let mut result = end;
    for index in 0..4 {
        if animates(channels, index) {
            result[index] = phase.place(index, start[index], end[index]);
        }
    }
    Some(result)
}

/// The per-tick rate of every geometry component at `phase`. A component this
/// declaration does not animate is not moving, so its rate is zero.
pub(super) fn geometry_rate(
    channels: UiMotionPropertyChannels,
    start: [f32; 4],
    end: [f32; 4],
    phase: UiMotionCurvePhase,
) -> [f32; 4] {
    let mut rate = [0.0; 4];
    for index in 0..4 {
        if animates(channels, index) {
            rate[index] = phase.rate(index, start[index], end[index]);
        }
    }
    rate
}

const fn animates(channels: UiMotionPropertyChannels, index: usize) -> bool {
    if channels.contains(UiMotionPropertyChannel::Geometry) {
        return true;
    }
    match index {
        0 => channels.contains(UiMotionPropertyChannel::TranslationX),
        1 => channels.contains(UiMotionPropertyChannel::TranslationY),
        _ => false,
    }
}
