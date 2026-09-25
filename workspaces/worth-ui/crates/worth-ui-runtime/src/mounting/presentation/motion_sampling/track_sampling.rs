use super::curve::UiMotionCurvePhase;
use super::track_geometry::{UiTrackCurveSpan, UiTrackGeometry, UiTrackSamplePlace};
use super::velocity::{UiPresentationOutgoingCurve, UiPresentationSampleVelocity};

#[path = "track_sampling/presented_lifecycle.rs"]
mod presented_lifecycle;

/// One track's presentation truth: its current sample, whether the host shows
/// that sample, and whether the track still moves.
#[derive(Clone, Copy)]
pub(super) struct UiPresentationTrackState {
    pub(super) track: crate::runtime::motion::UiCommittedMotionTrack,
    motion: UiTrackMotion,
    screen: UiTrackScreen,
    pub(super) current: super::UiPresentationMotionSampleReceipt,
    pub(super) current_opacity_units: u16,
}

/// Whether a track still moves, and the curve it moves along.
#[derive(Clone, Copy)]
enum UiTrackMotion {
    /// Sampled each tick along its curve.
    Running(UiTrackCurve),
    /// At rest: completed, snapped to its target, or rebound away.
    Settled,
}

/// Where a running track departed from and how it travels to its target.
#[derive(Clone, Copy)]
struct UiTrackCurve {
    /// The tick the curve's clock started, or `None` while it starts at the
    /// first tick that samples it.
    start_tick: Option<u64>,
    start_geometry: Option<crate::mounting::presentation::UiAcceptedRect>,
    start_opacity_units: u16,
    start_velocity: UiPresentationSampleVelocity,
    duration_ticks: u32,
}

/// Whether the host shows this track's current sample, and the geometry it
/// shows, which the next sample damages as it leaves.
#[derive(Clone, Copy)]
enum UiTrackScreen {
    /// The current sample has not reached the screen; the host still shows
    /// `showing`, the published geometry.
    Unpresented { showing: Option<UiTrackGeometry> },
    /// The host shows the current sample, drawn at `showing`.
    OnScreen { showing: Option<UiTrackGeometry> },
    /// The host shows the sample this track departed from, drawn at
    /// `showing`, and no tick has sampled the track since. A tick that was
    /// in flight when the track departed may still land and show a later
    /// sample to depart from instead.
    Departed { showing: Option<UiTrackGeometry> },
}

impl UiTrackScreen {
    const fn showing(self) -> Option<UiTrackGeometry> {
        match self {
            Self::Unpresented { showing }
            | Self::OnScreen { showing }
            | Self::Departed { showing } => showing,
        }
    }

    fn showing_damage(self) -> Option<[f32; 4]> {
        self.showing().map(UiTrackGeometry::damage_components)
    }
}

impl UiPresentationTrackState {
    /// The current sample, only while the host shows it.
    pub(super) const fn on_screen(&self) -> Option<super::UiPresentationMotionSampleReceipt> {
        match self.screen {
            UiTrackScreen::OnScreen { .. } | UiTrackScreen::Departed { .. } => Some(self.current),
            UiTrackScreen::Unpresented { .. } => None,
        }
    }

    /// Where the current sample puts the target.
    pub(super) const fn current_geometry(
        &self,
    ) -> Option<crate::mounting::presentation::UiAcceptedRect> {
        self.current.geometry()
    }

    pub(super) const fn is_running(&self) -> bool {
        matches!(self.motion, UiTrackMotion::Running(_))
    }

    /// Bring the track to rest where it stands.
    pub(super) fn settle(&mut self) {
        self.motion = UiTrackMotion::Settled;
    }

    /// Whether this track is exactly as it was installed: no tick has sampled
    /// it and no frame has accepted its initial sample. A retarget's clock is
    /// set at installation, so only presentation tells. A track that departed
    /// from the sample on screen shows that sample, not one of its own, so it
    /// is still unstarted. A snapped track is already terminal and never
    /// starts.
    pub(super) const fn is_unstarted(&self) -> bool {
        matches!(
            (self.motion, self.screen),
            (
                UiTrackMotion::Running(_),
                UiTrackScreen::Unpresented { .. } | UiTrackScreen::Departed { .. }
            )
        )
    }

    pub(super) fn new(
        track: crate::runtime::motion::UiCommittedMotionTrack,
        start_tick: Option<u64>,
        start_geometry: Option<UiTrackGeometry>,
        start_opacity_units: u16,
        start_velocity: UiPresentationSampleVelocity,
        duration_ticks: u32,
    ) -> Result<Self, super::UiPresentationGeometrySamplingDenial> {
        let host_presented_geometry = track.successor_geometry().map(UiTrackGeometry::Published);
        let start = start_geometry.map(UiTrackSamplePlace::from);
        let current = super::UiPresentationMotionSampleReceipt::from_track_sample(
            track,
            start_tick.unwrap_or(0),
            track.successor_presentation(),
            start,
            start_opacity_units,
            super::UiPresentationMotionSamplePosture::Delayed,
            super::UiPresentationMotionDamage::between(
                host_presented_geometry.map(UiTrackGeometry::damage_components),
                start.map(UiTrackSamplePlace::damage_components),
            ),
        )?;
        Ok(Self {
            track,
            motion: UiTrackMotion::Running(UiTrackCurve {
                start_tick,
                start_geometry: current.geometry(),
                start_opacity_units,
                start_velocity,
                duration_ticks,
            }),
            screen: UiTrackScreen::Unpresented {
                showing: host_presented_geometry,
            },
            current,
            current_opacity_units: start_opacity_units,
        })
    }

    pub(super) fn terminal(
        track: crate::runtime::motion::UiCommittedMotionTrack,
        tick: u64,
    ) -> Result<Self, super::UiPresentationGeometrySamplingDenial> {
        let geometry = track.successor_geometry();
        let place = geometry.map(UiTrackSamplePlace::Published);
        let opacity_units = target_opacity_units(track);
        let current = super::UiPresentationMotionSampleReceipt::from_track_sample(
            track,
            tick,
            track.successor_presentation(),
            place,
            opacity_units,
            super::UiPresentationMotionSamplePosture::Terminal,
            super::UiPresentationMotionDamage::between(
                None,
                place.map(UiTrackSamplePlace::damage_components),
            ),
        )?;
        Ok(Self {
            track,
            motion: UiTrackMotion::Settled,
            screen: UiTrackScreen::Unpresented {
                showing: geometry.map(UiTrackGeometry::Published),
            },
            current,
            current_opacity_units: opacity_units,
        })
    }

    /// Sample a running track at `tick`. A settled track has no curve to
    /// sample and answers `None`.
    pub(super) fn sample(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Option<
        Result<
            super::UiPresentationMotionSampleReceipt,
            super::UiPresentationGeometrySamplingDenial,
        >,
    > {
        let UiTrackMotion::Running(curve) = &mut self.motion else {
            return None;
        };
        let start = *curve.start_tick.get_or_insert(tick);
        let curve = *curve;
        let (normalized, posture) = self.horizon_at(curve, tick.saturating_sub(start));
        let phase = self.phase_at(curve, normalized);
        let geometry = curve
            .toward(self.track.successor_geometry())
            .point_at(self.track.declaration().channels(), phase)
            .map(UiTrackSamplePlace::Curve);
        let target_opacity_units = target_opacity_units(self.track);
        let opacity_units = if self
            .track
            .declaration()
            .channels()
            .contains(crate::runtime::motion::UiMotionPropertyChannel::Opacity)
        {
            super::opacity::interpolate_units(
                curve.start_opacity_units,
                target_opacity_units,
                phase.scalar_progress(),
            )
        } else {
            target_opacity_units
        };
        Some(self.present_sample(tick, presentation, geometry, opacity_units, posture))
    }

    /// Where `elapsed` since `curve`'s start sits on its declared horizon, and
    /// what that makes the sample's posture.
    fn horizon_at(
        &self,
        curve: UiTrackCurve,
        elapsed: u64,
    ) -> (f32, super::UiPresentationMotionSamplePosture) {
        let delay = u64::from(self.track.declaration().delay_ticks());
        let Some(active_elapsed) = elapsed.checked_sub(delay) else {
            return (0.0, super::UiPresentationMotionSamplePosture::Delayed);
        };
        let duration = u64::from(curve.duration_ticks.max(1));
        let normalized = (active_elapsed as f64 / duration as f64).min(1.0) as f32;
        let posture = if normalized >= 1.0 {
            super::UiPresentationMotionSamplePosture::Terminal
        } else {
            super::UiPresentationMotionSamplePosture::Active
        };
        (normalized, posture)
    }

    fn phase_at(&self, curve: UiTrackCurve, normalized: f32) -> UiMotionCurvePhase {
        UiMotionCurvePhase::new(
            self.track.declaration().easing(),
            normalized,
            curve.duration_ticks,
            curve.start_velocity.components(),
        )
    }

    /// The curve this track is running at `tick`, for a successor about to
    /// displace it. A settled track, one whose clock has not started, one still
    /// inside its delay, or one with no geometry basis is not moving and
    /// carries no curve.
    pub(super) fn outgoing_curve(&self, tick: u64) -> Option<UiPresentationOutgoingCurve> {
        let UiTrackMotion::Running(curve) = self.motion else {
            return None;
        };
        let (normalized, posture) = self.horizon_at(curve, tick.saturating_sub(curve.start_tick?));
        if posture == super::UiPresentationMotionSamplePosture::Delayed {
            return None;
        }
        curve.toward(self.track.successor_geometry()).outgoing_at(
            self.track.declaration().channels(),
            self.phase_at(curve, normalized),
        )
    }

    pub(super) fn snap_system_reduced_motion(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<super::UiPresentationMotionSampleReceipt, super::UiPresentationGeometrySamplingDenial>
    {
        let geometry = self
            .track
            .successor_geometry()
            .map(UiTrackSamplePlace::Published);
        let opacity_units = target_opacity_units(self.track);
        let sample = self.present_sample(
            tick,
            presentation,
            geometry,
            opacity_units,
            super::UiPresentationMotionSamplePosture::Terminal,
        )?;
        self.motion = UiTrackMotion::Settled;
        Ok(sample)
    }

    pub(super) fn shorten_system_reduced_motion(&mut self) {
        if let UiTrackMotion::Running(curve) = &mut self.motion {
            curve.duration_ticks = curve.duration_ticks.min(1);
        }
    }

    /// Make `geometry` this track's current sample, damaging what the host
    /// shows as the sample leaves it.
    fn present_sample(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        geometry: Option<UiTrackSamplePlace>,
        opacity_units: u16,
        posture: super::UiPresentationMotionSamplePosture,
    ) -> Result<super::UiPresentationMotionSampleReceipt, super::UiPresentationGeometrySamplingDenial>
    {
        let damage = super::UiPresentationMotionDamage::between(
            self.screen.showing_damage(),
            geometry.map(UiTrackSamplePlace::damage_components),
        );
        let sample = super::UiPresentationMotionSampleReceipt::from_track_sample(
            self.track,
            tick,
            presentation,
            geometry,
            opacity_units,
            posture,
            damage,
        )?;
        self.current_opacity_units = opacity_units;
        self.current = sample;
        self.screen = UiTrackScreen::OnScreen {
            showing: sample.geometry().map(UiTrackGeometry::Accepted),
        };
        Ok(sample)
    }
}

impl UiTrackCurve {
    /// The span this curve travels from its departure to `target`.
    fn toward(
        self,
        target: Option<crate::mounting::presentation::UiPublishedRect>,
    ) -> UiTrackCurveSpan {
        UiTrackCurveSpan::new(self.start_geometry, target)
    }
}

/// The opacity `track` rests at once it reaches its target.
fn target_opacity_units(track: crate::runtime::motion::UiCommittedMotionTrack) -> u16 {
    if track.successor_visible() {
        u16::MAX
    } else {
        0
    }
}
