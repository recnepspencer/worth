//! The milestone's timing criteria, measured on the same live window: how long
//! the product takes to put a coarse wheel notch on screen, and how evenly the
//! frames that carry it arrive.
//!
//! The measurement's resolution is the cost of one probe, so that cost is
//! measured first and travels with the verdict. Raw intervals are kept and
//! reported, never averaged: a stall a mean would hide is the thing worth
//! finding.
use std::fmt::Write as _;
use std::time::{Duration, Instant};

use crate::adjudication::{physical_px, RecentActivityScrollGeometry};
use crate::external_observation::{NativeClientPixelCapture, NativeClientPixelPoint};
use crate::native_platform::{NativePlatformContract, WindowsNativePlatform};

use super::{NativeBoundExecutableWorld, PlatformPulseScrollJourneyFailure};

/// Wheel notches timed, alternating direction so the list stays mid-travel.
const TRIALS: usize = 12;
/// Back-to-back probes timed before the trials, to know the sampling floor.
const PROBE_COST_SAMPLES: usize = 16;
/// A trial that shows nothing within this is a stall, not a slow frame.
const FIRST_CHANGE_DEADLINE: Duration = Duration::from_secs(10);
/// Quiet probing that ends a trial: the list has stopped moving.
const SETTLED_QUIET: Duration = Duration::from_millis(150);
/// A trial stops being followed after this, settled or not.
const TRIAL_DEADLINE: Duration = Duration::from_secs(12);
/// Rest between trials, so each notch starts from a still list.
const BETWEEN_TRIALS: Duration = Duration::from_millis(400);
/// Channel difference that makes a pixel a changed pixel.
const CHANNEL_TOLERANCE: u8 = 8;
/// Changed pixels in the strip that together count as visible movement.
const VISIBLE_CHANGE_PX: usize = 64;
/// The probed strip, logical points inset into the scrolled viewport. Text rows
/// cross it, so anything that moves the list changes it.
const STRIP_INSET_POINTS: [f64; 2] = [16.0, 16.0];
const STRIP_SIZE_POINTS: [f64; 2] = [420.0, 200.0];

/// Raw per-notch intervals, and the probe cost that bounds their resolution.
pub(crate) struct ScrollLatencyEvidence {
    probe_cost: Vec<Duration>,
    preparation: Vec<Duration>,
    first_change: Vec<Duration>,
    frame_gaps: Vec<Duration>,
    settlement: Vec<Duration>,
}

impl ScrollLatencyEvidence {
    pub(crate) fn first_change(&self) -> &[Duration] {
        &self.first_change
    }

    pub(crate) fn frame_gaps(&self) -> &[Duration] {
        &self.frame_gaps
    }

    pub(crate) fn settlement(&self) -> &[Duration] {
        &self.settlement
    }

    /// The worst probe observed: the coarsest this measurement can resolve.
    pub(crate) fn sampling_floor(&self) -> Duration {
        self.probe_cost.iter().copied().max().unwrap_or_default()
    }

    /// Nearest-rank percentile over the raw intervals, nothing smoothed.
    pub(crate) fn first_change_percentile(&self, permille: usize) -> Duration {
        percentile(&self.first_change, permille)
    }

    pub(crate) fn frame_gap_percentile(&self, permille: usize) -> Duration {
        percentile(&self.frame_gaps, permille)
    }

    pub(crate) fn report(&self) -> String {
        let mut report = String::new();
        let _ = write!(
            report,
            "probe cost: p50={:?} p95={:?} worst={:?} over {} samples",
            percentile(&self.probe_cost, 500),
            percentile(&self.probe_cost, 950),
            self.sampling_floor(),
            self.probe_cost.len()
        );
        let _ = write!(
            report,
            "\nharness preparation before each notch (excluded): p50={:?} worst={:?}",
            percentile(&self.preparation, 500),
            percentile(&self.preparation, 1_000)
        );
        let _ = write!(
            report,
            "\ninput to first visible change: p50={:?} p95={:?} p99={:?} worst={:?}",
            self.first_change_percentile(500),
            self.first_change_percentile(950),
            self.first_change_percentile(990),
            percentile(&self.first_change, 1_000)
        );
        let _ = write!(
            report,
            "\naccepted visible-frame gap: p50={:?} p95={:?} p99={:?} worst={:?} over {} gaps",
            self.frame_gap_percentile(500),
            self.frame_gap_percentile(950),
            self.frame_gap_percentile(990),
            percentile(&self.frame_gaps, 1_000),
            self.frame_gaps.len()
        );
        let _ = write!(
            report,
            "\nsettlement after the notch: p50={:?} worst={:?}",
            percentile(&self.settlement, 500),
            percentile(&self.settlement, 1_000)
        );
        let _ = write!(report, "\nraw first-change intervals: {:?}", self.first_change);
        report
    }
}

fn percentile(samples: &[Duration], permille: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let Some(last) = sorted.len().checked_sub(1) else {
        return Duration::ZERO;
    };
    let rank = (permille * sorted.len()).div_ceil(1_000).saturating_sub(1);
    sorted.get(rank.min(last)).copied().unwrap_or_default()
}

/// Deliver `TRIALS` real wheel notches and follow each one to a still list.
pub(super) fn measure_wheel_latency(
    world: &mut NativeBoundExecutableWorld,
    dpi: u32,
    interior: NativeClientPixelPoint,
) -> Result<ScrollLatencyEvidence, PlatformPulseScrollJourneyFailure> {
    let strip = strip_rect_px(dpi);
    let mut probe_cost = Vec::with_capacity(PROBE_COST_SAMPLES);
    {
        let exposed = expose(world)?;
        for _ in 0..PROBE_COST_SAMPLES {
            let started = Instant::now();
            let observed = sample(&world.platform, &exposed, strip)?;
            probe_cost.push(started.elapsed());
            drop(observed);
        }
    }

    let mut preparation = Vec::with_capacity(TRIALS);
    let mut first_change = Vec::with_capacity(TRIALS);
    let mut settlement = Vec::with_capacity(TRIALS);
    let mut frame_gaps = Vec::new();
    for trial in 0..TRIALS {
        std::thread::sleep(BETWEEN_TRIALS);
        // Exposing raises the window and waits for a composition. It is the
        // precondition of every sample in this trial, so it happens once, here,
        // outside every interval the trial measures.
        let exposed = expose(world)?;
        let mut previous = sample(&world.platform, &exposed, strip)?;
        // Alternate direction so the list never reaches an edge, where a notch
        // would legitimately move nothing and time out as a false stall.
        let notches = if trial % 2 == 0 { 1 } else { -1 };
        let requested = Instant::now();
        // The clock starts when the event leaves for the product, not when the
        // harness starts focusing windows and qualifying the pointer for it.
        let issued = world
            .platform
            .deliver_wheel_notches(&world.native_client, interior, notches)
            .map_err(PlatformPulseScrollJourneyFailure::Native)?;
        preparation.push(issued.saturating_duration_since(requested));

        let mut seen_first = None;
        let mut last_change = issued;
        loop {
            let current = sample(&world.platform, &exposed, strip)?;
            let now = Instant::now();
            if changed_pixels(&previous, &current) >= VISIBLE_CHANGE_PX {
                if seen_first.is_none() {
                    seen_first = Some(now.saturating_duration_since(issued));
                } else {
                    frame_gaps.push(now.saturating_duration_since(last_change));
                }
                last_change = now;
                previous = current;
            } else if seen_first.is_some()
                && now.saturating_duration_since(last_change) >= SETTLED_QUIET
            {
                break;
            }
            let waited = now.saturating_duration_since(issued);
            if seen_first.is_none() && waited >= FIRST_CHANGE_DEADLINE {
                return Err(PlatformPulseScrollJourneyFailure::WheelNeverMoved(trial));
            }
            if waited >= TRIAL_DEADLINE {
                break;
            }
        }
        let Some(first) = seen_first else {
            return Err(PlatformPulseScrollJourneyFailure::WheelNeverMoved(trial));
        };
        first_change.push(first);
        settlement.push(last_change.saturating_duration_since(issued));
    }

    Ok(ScrollLatencyEvidence {
        probe_cost,
        preparation,
        first_change,
        frame_gaps,
        settlement,
    })
}

/// Hold the window where a trial's samples can read it.
fn expose(
    world: &NativeBoundExecutableWorld,
) -> Result<
    <WindowsNativePlatform as NativePlatformContract>::ExposedClientArea<'_>,
    PlatformPulseScrollJourneyFailure,
> {
    world
        .platform
        .expose_client_area(&world.native_client)
        .map_err(PlatformPulseScrollJourneyFailure::Native)
}

/// One timing-probe capture: cheap enough to resolve the interval measured.
fn sample<Platform: NativePlatformContract>(
    platform: &Platform,
    exposed: &Platform::ExposedClientArea<'_>,
    strip: [u32; 4],
) -> Result<NativeClientPixelCapture, PlatformPulseScrollJourneyFailure> {
    platform
        .sample_exposed_strip(exposed, strip)
        .map_err(PlatformPulseScrollJourneyFailure::Native)
}

/// A band inside the scrolled viewport, in client pixels.
fn strip_rect_px(dpi: u32) -> [u32; 4] {
    let viewport = RecentActivityScrollGeometry.viewport_points();
    let edges = [
        viewport[0] + STRIP_INSET_POINTS[0],
        viewport[1] + STRIP_INSET_POINTS[1],
        STRIP_SIZE_POINTS[0].min(viewport[2] - STRIP_INSET_POINTS[0] * 2.0),
        STRIP_SIZE_POINTS[1].min(viewport[3] - STRIP_INSET_POINTS[1] * 2.0),
    ];
    edges.map(|value| physical_px(value, dpi).max(0) as u32)
}

/// How many pixels differ between two probes of the same strip.
fn changed_pixels(before: &NativeClientPixelCapture, after: &NativeClientPixelCapture) -> usize {
    if before.width() != after.width() || before.height() != after.height() {
        return usize::MAX;
    }
    before
        .rgba()
        .chunks_exact(4)
        .zip(after.rgba().chunks_exact(4))
        .filter(|(before, after)| {
            before
                .iter()
                .take(3)
                .zip(after.iter().take(3))
                .any(|(before, after)| before.abs_diff(*after) > CHANNEL_TOLERANCE)
        })
        .count()
}
