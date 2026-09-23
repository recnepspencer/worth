//! The Recent activity scroll journey: one real wheel notch, a real thumb drag,
//! and a hit-test click after each, adjudicated against independent geometry.
use std::time::Duration;

use crate::adjudication::{
    notch_points, physical_px, platform_wheel_lines_per_notch, ContentShiftEvidence,
    RecentActivityScrollGeometry, VerticalThumbEvidence,
};
use crate::external_observation::{NativeClientPixelCapture, NativeClientPixelPoint};
use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};
use crate::native_platform::NativePlatformContract;

use super::{DashboardAtRest, NativeBoundExecutableWorld, Published, PulseExecutableWorld};

mod axes;
mod failure;
mod moving_thumb;
mod observation;
mod thumb_observation;
pub(crate) use failure::PlatformPulseScrollJourneyFailure;
use observation::{
    await_moved_content, await_unrouted_hit, capture, drain_until_idle, export_capture,
    MovedContentEvidence,
};

const TRANSITION_DEADLINE: Duration = Duration::from_secs(5);
const PIXEL_POLL_SLICE: Duration = Duration::from_millis(10);
const LIFECYCLE_IDLE_SLICE: Duration = Duration::from_millis(100);
const CLICK_LANDING_TOLERANCE_PX: u32 = 2;
/// Authored Recent activity row table: content origin, dot rectangle, row pitch.
const CONTENT_ORIGIN_POINTS: [f64; 2] = [290.0, 693.0];
const DOT_CENTER_LOCAL_POINTS: [f64; 2] = [3.0 + 7.5, 10.0 + 7.5];
const ROW_PITCH_POINTS: f64 = 56.0;
/// How far the thumb is dragged, physical pixels along the track.
const THUMB_DRAG_DELTA_PX: i64 = 30;

pub(crate) struct PlatformPulseScrollJourneyEvidence {
    dpi: u32,
    lines_per_notch: u32,
    notch_points: f64,
    wheel: MovedContentEvidence,
    drag_offset_points: f64,
    drag: MovedContentEvidence,
    resting_hits: [u64; 3],
    hit_after_wheel: u64,
    hit_after_drag: u64,
    expected_shutdown_sequence: u64,
}

pub(crate) struct CompletedPlatformPulseScrollJourney {
    ready: PulseExecutableWorld<Published<DashboardAtRest>>,
    evidence: PlatformPulseScrollJourneyEvidence,
}

impl PulseExecutableWorld<Published<DashboardAtRest>> {
    pub(crate) fn complete_scroll_journey(
        self,
    ) -> Result<CompletedPlatformPulseScrollJourney, PulseExecutableWorldFailureReport> {
        let Published { mut world, stage } = self.state;
        match complete(&mut world) {
            Ok(evidence) => Ok(CompletedPlatformPulseScrollJourney {
                ready: PulseExecutableWorld {
                    state: Published { world, stage },
                },
                evidence,
            }),
            Err(failure) => Err(teardown_native_bound_world(
                PulseExecutableWorldFailure::ScrollJourney(failure),
                world.into_failure_resources(),
            )),
        }
    }
}

fn complete(
    world: &mut NativeBoundExecutableWorld,
) -> Result<PlatformPulseScrollJourneyEvidence, PlatformPulseScrollJourneyFailure> {
    let lines_per_notch =
        platform_wheel_lines_per_notch().map_err(PlatformPulseScrollJourneyFailure::Pixels)?;
    let notch = notch_points(lines_per_notch);
    let geometry = RecentActivityScrollGeometry;

    let dpi = world
        .platform
        .observe_bound_client_area(&world.native_client)
        .map_err(PlatformPulseScrollJourneyFailure::Native)?
        .dpi();
    // The dashboard entry already adjudicated this frame at rest; the journey
    // takes its own baseline so every shift is measured from pixels it observed.
    let baseline = capture(world)?;
    export_capture("03-scroll-rest.png", &baseline)?;

    let mut resting_hits = [0_u64; 3];
    for (band, hit) in (0_u32..).zip(resting_hits.iter_mut()) {
        *hit = await_unrouted_hit(world, dot_center_point(&baseline, dpi, band)?)?;
    }
    if resting_hits[0] == resting_hits[1] || resting_hits[1] == resting_hits[2] {
        return Err(PlatformPulseScrollJourneyFailure::HitTargetsIndistinct(
            resting_hits,
        ));
    }

    moving_thumb::verify(world, &baseline, dpi, notch)?;
    axes::verify(world, &baseline, dpi, notch)?;
    let interior = client_point(&baseline, dpi, geometry.content_interior_points())?;
    let _issued = world
        .platform
        .deliver_wheel_notches(&world.native_client, interior, 1)
        .map_err(PlatformPulseScrollJourneyFailure::Native)?;
    let wheel = await_moved_content(world, &baseline, dpi, notch)?;
    export_capture("04-scroll-wheel.png", wheel.capture())?;
    let hit_after_wheel = await_unrouted_hit(world, dot_center_point(&baseline, dpi, 0)?)?;
    require_hit(hit_after_wheel, resting_hits[1], resting_hits[0])?;

    let from = client_point(&baseline, dpi, geometry.thumb_center_points(notch))?;
    let (from_x, from_y) = from.coordinates();
    let to_y = u32::try_from(i64::from(from_y) + THUMB_DRAG_DELTA_PX)
        .map_err(|_| PlatformPulseScrollJourneyFailure::PointOutsideCapture([from_x, from_y]))?;
    let to = NativeClientPixelPoint::interior(&baseline, from_x, to_y, CLICK_LANDING_TOLERANCE_PX)
        .ok_or(PlatformPulseScrollJourneyFailure::PointOutsideCapture([
            from_x, to_y,
        ]))?;
    world
        .platform
        .deliver_pointer_drag(&world.native_client, from, to)
        .map_err(PlatformPulseScrollJourneyFailure::Native)?;
    let drag_delta_points = THUMB_DRAG_DELTA_PX as f64 * 96.0 / f64::from(dpi);
    let drag_offset_points = geometry.offset_after_thumb_drag(notch, drag_delta_points);
    let drag = await_moved_content(world, &baseline, dpi, drag_offset_points)?;
    export_capture("05-scroll-drag.png", drag.capture())?;
    let hit_after_drag = await_unrouted_hit(world, dot_center_point(&baseline, dpi, 0)?)?;
    require_hit(hit_after_drag, resting_hits[2], resting_hits[1])?;

    let expected_shutdown_sequence = drain_until_idle(world)?;
    Ok(PlatformPulseScrollJourneyEvidence {
        dpi,
        lines_per_notch,
        notch_points: notch,
        wheel,
        drag_offset_points,
        drag,
        resting_hits,
        hit_after_wheel,
        hit_after_drag,
        expected_shutdown_sequence,
    })
}

fn require_hit(
    observed: u64,
    expected: u64,
    previous: u64,
) -> Result<(), PlatformPulseScrollJourneyFailure> {
    if observed == previous {
        return Err(PlatformPulseScrollJourneyFailure::HitTargetUnchanged(
            observed,
        ));
    }
    if observed != expected {
        return Err(PlatformPulseScrollJourneyFailure::HitTarget { expected, observed });
    }
    Ok(())
}

/// The screen point of the band-`band` row dot centre at rest, physical pixels.
fn dot_center_point(
    capture: &NativeClientPixelCapture,
    dpi: u32,
    band: u32,
) -> Result<NativeClientPixelPoint, PlatformPulseScrollJourneyFailure> {
    client_point(
        capture,
        dpi,
        [
            CONTENT_ORIGIN_POINTS[0] + DOT_CENTER_LOCAL_POINTS[0],
            CONTENT_ORIGIN_POINTS[1]
                + DOT_CENTER_LOCAL_POINTS[1]
                + f64::from(band) * ROW_PITCH_POINTS,
        ],
    )
}

fn client_point(
    capture: &NativeClientPixelCapture,
    dpi: u32,
    points: [f64; 2],
) -> Result<NativeClientPixelPoint, PlatformPulseScrollJourneyFailure> {
    let pixels = points.map(|value| physical_px(value, dpi));
    let x = u32::try_from(pixels[0]).ok();
    let y = u32::try_from(pixels[1]).ok();
    let (Some(x), Some(y)) = (x, y) else {
        return Err(PlatformPulseScrollJourneyFailure::PointOutsideCapture([
            0, 0,
        ]));
    };
    NativeClientPixelPoint::interior(capture, x, y, CLICK_LANDING_TOLERANCE_PX).ok_or(
        PlatformPulseScrollJourneyFailure::PointOutsideCapture([x, y]),
    )
}

impl CompletedPlatformPulseScrollJourney {
    pub(crate) fn evidence(&self) -> &PlatformPulseScrollJourneyEvidence {
        &self.evidence
    }

    pub(crate) fn into_ready(self) -> PulseExecutableWorld<Published<DashboardAtRest>> {
        self.ready
    }
}

impl PlatformPulseScrollJourneyEvidence {
    pub(crate) const fn dpi(&self) -> u32 {
        self.dpi
    }

    pub(crate) const fn lines_per_notch(&self) -> u32 {
        self.lines_per_notch
    }

    pub(crate) const fn notch_points(&self) -> f64 {
        self.notch_points
    }

    pub(crate) const fn wheel_thumb(&self) -> VerticalThumbEvidence {
        self.wheel.thumb()
    }

    pub(crate) const fn wheel_shift(&self) -> ContentShiftEvidence {
        self.wheel.shift()
    }

    pub(crate) const fn drag_offset_points(&self) -> f64 {
        self.drag_offset_points
    }

    pub(crate) const fn drag_thumb(&self) -> VerticalThumbEvidence {
        self.drag.thumb()
    }

    pub(crate) const fn drag_shift(&self) -> ContentShiftEvidence {
        self.drag.shift()
    }

    pub(crate) const fn resting_hits(&self) -> [u64; 3] {
        self.resting_hits
    }

    pub(crate) const fn hits_after_motion(&self) -> [u64; 2] {
        [self.hit_after_wheel, self.hit_after_drag]
    }

    pub(crate) const fn expected_shutdown_sequence(&self) -> u64 {
        self.expected_shutdown_sequence
    }
}
