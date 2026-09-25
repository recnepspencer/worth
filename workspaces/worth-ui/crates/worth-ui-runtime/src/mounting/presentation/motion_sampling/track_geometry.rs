use super::curve::UiMotionCurvePhase;
use super::velocity::{UiPresentationOutgoingCurve, UiPresentationSampleVelocity};
use crate::mounting::presentation::{UiAcceptedRect, UiPublishedRect};
use crate::runtime::motion::UiMotionPropertyChannels;
use worth_ui_host_contract::UiHostObservationPresentationBasis;

/// Geometry a track departs from, or that the host shows beneath its next
/// sample: the published layout a frame drew, or a sample presentation
/// accepted. The two differ in truth status, so the track names which one it
/// holds rather than keeping bare components.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum UiTrackGeometry {
    Published(UiPublishedRect),
    Accepted(UiAcceptedRect),
}

/// Where a track sample puts its target, named by the truth it comes from.
/// A sample admits each kind through its own crossing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum UiTrackSamplePlace {
    /// Held at published geometry: a start at the predecessor's layout, or
    /// rest at the target's.
    Published(UiPublishedRect),
    /// A sample carried from an earlier basis of the same surface binding.
    Carried(UiAcceptedRect),
    /// Where the curve puts the target this tick.
    Curve(UiTrackCurvePoint),
}

/// A point on a running track's curve. Only curve evaluation mints it, and
/// it becomes truth only when a sample admits it as accepted geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiTrackCurvePoint([f32; 4]);

/// The span a running track's curve travels: from the accepted sample it
/// departed from to the published target it heads for. Its ends differ in
/// truth status, and the curve is the one owner that reads them together:
/// their components never leave it except as a `UiTrackCurvePoint`, a bounded
/// velocity, or the outgoing curve a retarget differentiates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiTrackCurveSpan {
    departure: Option<UiAcceptedRect>,
    target: Option<UiPublishedRect>,
}

impl UiTrackCurveSpan {
    pub(super) const fn new(
        departure: Option<UiAcceptedRect>,
        target: Option<UiPublishedRect>,
    ) -> Self {
        Self { departure, target }
    }

    /// Where the curve puts its target at `phase`.
    pub(super) fn point_at(
        self,
        channels: UiMotionPropertyChannels,
        phase: UiMotionCurvePhase,
    ) -> Option<UiTrackCurvePoint> {
        let (departure, target) = self.ends();
        super::curve::interpolate_geometry(channels, departure, target, phase)
            .map(UiTrackCurvePoint)
    }

    /// `velocity`, bounded so the curve cannot overshoot this span.
    pub(super) fn bounded_velocity(
        self,
        velocity: UiPresentationSampleVelocity,
        ticks: u32,
    ) -> UiPresentationSampleVelocity {
        match self.ends() {
            (Some(start), Some(end)) => velocity.within_extent(start, end, ticks),
            _ => velocity,
        }
    }

    /// The curve as a retarget interrupting it at `phase` differentiates it.
    /// A span with no departure is not moving and carries no curve.
    pub(super) fn outgoing_at(
        self,
        channels: UiMotionPropertyChannels,
        phase: UiMotionCurvePhase,
    ) -> Option<UiPresentationOutgoingCurve> {
        let (Some(start), Some(end)) = self.ends() else {
            return None;
        };
        Some(UiPresentationOutgoingCurve::interrupted_at(
            channels, phase, start, end,
        ))
    }

    /// A target without geometry leaves the curve where it departed.
    fn ends(self) -> (Option<[f32; 4]>, Option<[f32; 4]>) {
        let start = self.departure.map(UiAcceptedRect::components);
        (
            start,
            self.target.map(UiPublishedRect::components).or(start),
        )
    }
}

impl UiTrackSamplePlace {
    /// This place as a sample drawn on `presentation`. Published geometry is
    /// held there, a carried sample must rebase within its surface binding,
    /// and a curve point is sampled toward `target`, the published geometry
    /// the track heads for.
    pub(super) fn accepted_on(
        self,
        target: UiPublishedRect,
        presentation: UiHostObservationPresentationBasis,
    ) -> Result<UiAcceptedRect, super::UiPresentationGeometrySamplingDenial> {
        match self {
            Self::Published(rect) => Ok(UiAcceptedRect::holding(rect, presentation)),
            Self::Carried(rect) => Ok(rect.rebased(presentation)?),
            Self::Curve(UiTrackCurvePoint(components)) => {
                Ok(UiAcceptedRect::sampled(components, target, presentation)?)
            }
        }
    }

    /// The components the sample's damage hands the host as a region to
    /// redraw.
    pub(super) fn damage_components(self) -> [f32; 4] {
        match self {
            Self::Published(rect) => rect.components(),
            Self::Carried(rect) => rect.components(),
            Self::Curve(UiTrackCurvePoint(components)) => components,
        }
    }
}

impl From<UiTrackGeometry> for UiTrackSamplePlace {
    fn from(geometry: UiTrackGeometry) -> Self {
        match geometry {
            UiTrackGeometry::Published(rect) => Self::Published(rect),
            UiTrackGeometry::Accepted(rect) => Self::Carried(rect),
        }
    }
}

impl UiTrackGeometry {
    /// The components damage hands the host as a region to redraw.
    pub(super) fn damage_components(self) -> [f32; 4] {
        UiTrackSamplePlace::from(self).damage_components()
    }
}
