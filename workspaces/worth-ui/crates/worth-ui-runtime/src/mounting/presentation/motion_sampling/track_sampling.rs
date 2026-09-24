use super::curve::UiMotionCurvePhase;
use super::velocity::{UiPresentationOutgoingCurve, UiPresentationSampleVelocity};

/// One track's presentation truth: its current sample, whether the host shows
/// that sample, and whether the track still moves.
#[derive(Clone, Copy)]
pub(super) struct UiPresentationTrackState {
    pub(super) track: crate::runtime::motion::UiCommittedMotionTrack,
    motion: UiTrackMotion,
    screen: UiTrackScreen,
    pub(super) current: super::UiPresentationMotionSampleReceipt,
    pub(super) current_geometry: Option<[f32; 4]>,
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
    start_geometry: Option<[f32; 4]>,
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
    Unpresented { showing: Option<[f32; 4]> },
    /// The host shows the current sample, drawn at `showing`.
    OnScreen { showing: Option<[f32; 4]> },
}

impl UiTrackScreen {
    const fn showing(self) -> Option<[f32; 4]> {
        match self {
            Self::Unpresented { showing } | Self::OnScreen { showing } => showing,
        }
    }
}

impl UiPresentationTrackState {
    /// The current sample, only while the host shows it.
    pub(super) const fn on_screen(&self) -> Option<super::UiPresentationMotionSampleReceipt> {
        match self.screen {
            UiTrackScreen::OnScreen { .. } => Some(self.current),
            UiTrackScreen::Unpresented { .. } => None,
        }
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
    /// set at installation, so only presentation tells; a snapped track is
    /// already terminal and never starts.
    pub(super) const fn is_unstarted(&self) -> bool {
        matches!(
            (self.motion, self.screen),
            (UiTrackMotion::Running(_), UiTrackScreen::Unpresented { .. })
        )
    }

    pub(super) fn rebase_presented_extent(
        &mut self,
        tick: u64,
    ) -> Result<(), super::UiPresentationGeometrySamplingDenial> {
        let geometry = self
            .track
            .predecessor_geometry()
            .map(|geometry| geometry.components());
        let sample = super::UiPresentationMotionSampleReceipt::from_track_sample(
            self.track,
            tick,
            self.track.successor_presentation(),
            geometry,
            self.current_opacity_units,
            super::UiPresentationMotionSamplePosture::Active,
            super::UiPresentationMotionDamage::between(geometry, geometry),
        )?;
        let target = self.target_geometry();
        if let UiTrackMotion::Running(curve) = &mut self.motion {
            if let (Some(start), Some(end)) = (geometry, target) {
                curve.start_velocity =
                    curve
                        .start_velocity
                        .within_extent(start, end, curve.duration_ticks);
            }
            curve.start_tick = Some(tick);
            curve.start_geometry = geometry;
        }
        self.current_geometry = geometry;
        self.current = sample;
        self.screen = UiTrackScreen::OnScreen { showing: geometry };
        Ok(())
    }

    /// The entrance sample is accepted as this track's current sample, but it
    /// is delayed and carries no opacity: the frame that published it drew the
    /// overlay at its successor geometry, not at the entrance offset. Claiming
    /// the entrance geometry as presented erased the only record of what the
    /// host still shows, so the first moving sample damaged its own destination
    /// twice and left the published successor on screen.
    pub(super) fn accept_published_entrance(
        &mut self,
        sample: super::UiPresentationMotionSampleReceipt,
    ) {
        assert_eq!(
            self.current, sample,
            "the physically accepted entrance matches the installed initial sample"
        );
        self.screen = UiTrackScreen::OnScreen {
            showing: self.screen.showing(),
        };
    }

    /// Record that the host already shows this track's initial sample, which
    /// `on_screen` put there. A retarget resolved from a presented sample
    /// starts where that sample left the target, so its departure is what the
    /// screen holds: the displayed pose settles to it, and the next sample
    /// damages the place it leaves rather than the published geometry.
    /// A departure elsewhere is not on screen and stays unpresented.
    pub(super) fn depart_on_screen(&mut self, on_screen: &Self) {
        if let UiTrackScreen::OnScreen { showing } = on_screen.screen {
            if self.current_geometry == on_screen.current_geometry {
                self.screen = UiTrackScreen::OnScreen { showing };
            }
        }
    }

    pub(super) fn new(
        track: crate::runtime::motion::UiCommittedMotionTrack,
        start_tick: Option<u64>,
        start_geometry: Option<[f32; 4]>,
        start_opacity_units: u16,
        start_velocity: UiPresentationSampleVelocity,
        duration_ticks: u32,
    ) -> Result<Self, super::UiPresentationGeometrySamplingDenial> {
        let host_presented_geometry = track
            .successor_geometry()
            .map(crate::runtime::motion::UiMotionSemanticGeometry::components);
        let current = super::UiPresentationMotionSampleReceipt::from_track_sample(
            track,
            start_tick.unwrap_or(0),
            track.successor_presentation(),
            start_geometry,
            start_opacity_units,
            super::UiPresentationMotionSamplePosture::Delayed,
            super::UiPresentationMotionDamage::between(host_presented_geometry, start_geometry),
        )?;
        Ok(Self {
            track,
            motion: UiTrackMotion::Running(UiTrackCurve {
                start_tick,
                start_geometry,
                start_opacity_units,
                start_velocity,
                duration_ticks,
            }),
            screen: UiTrackScreen::Unpresented {
                showing: host_presented_geometry,
            },
            current,
            current_geometry: start_geometry,
            current_opacity_units: start_opacity_units,
        })
    }

    pub(super) fn terminal(
        track: crate::runtime::motion::UiCommittedMotionTrack,
        tick: u64,
    ) -> Result<Self, super::UiPresentationGeometrySamplingDenial> {
        let geometry = track
            .successor_geometry()
            .map(crate::runtime::motion::UiMotionSemanticGeometry::components);
        let opacity_units = target_opacity_units(track);
        let current = super::UiPresentationMotionSampleReceipt::from_track_sample(
            track,
            tick,
            track.successor_presentation(),
            geometry,
            opacity_units,
            super::UiPresentationMotionSamplePosture::Terminal,
            super::UiPresentationMotionDamage::between(None, geometry),
        )?;
        Ok(Self {
            track,
            motion: UiTrackMotion::Settled,
            screen: UiTrackScreen::Unpresented { showing: geometry },
            current,
            current_geometry: geometry,
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
        let target_geometry = self.target_geometry();
        let UiTrackMotion::Running(curve) = &mut self.motion else {
            return None;
        };
        let start = *curve.start_tick.get_or_insert(tick);
        let curve = *curve;
        let (normalized, posture) = self.horizon_at(curve, tick.saturating_sub(start));
        let phase = self.phase_at(curve, normalized);
        let geometry = super::curve::interpolate_geometry(
            self.track.declaration().channels(),
            curve.start_geometry,
            target_geometry,
            phase,
        );
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

    fn target_geometry(&self) -> Option<[f32; 4]> {
        let start = match self.motion {
            UiTrackMotion::Running(curve) => curve.start_geometry,
            UiTrackMotion::Settled => None,
        };
        self.track
            .successor_geometry()
            .map(crate::runtime::motion::UiMotionSemanticGeometry::components)
            .or(start)
    }

    /// The curve this track is running at `tick`, for a successor about to
    /// displace it. A settled track, one whose clock has not started, one still
    /// inside its delay, or one with no geometry basis is not moving and
    /// carries no curve.
    pub(super) fn outgoing_curve(&self, tick: u64) -> Option<UiPresentationOutgoingCurve> {
        let UiTrackMotion::Running(curve) = self.motion else {
            return None;
        };
        let start_geometry = curve.start_geometry?;
        let (normalized, posture) = self.horizon_at(curve, tick.saturating_sub(curve.start_tick?));
        if posture == super::UiPresentationMotionSamplePosture::Delayed {
            return None;
        }
        Some(UiPresentationOutgoingCurve::interrupted_at(
            self.track.declaration().channels(),
            self.phase_at(curve, normalized),
            start_geometry,
            self.target_geometry().unwrap_or(start_geometry),
        ))
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
            .map(crate::runtime::motion::UiMotionSemanticGeometry::components);
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
        geometry: Option<[f32; 4]>,
        opacity_units: u16,
        posture: super::UiPresentationMotionSamplePosture,
    ) -> Result<super::UiPresentationMotionSampleReceipt, super::UiPresentationGeometrySamplingDenial>
    {
        let damage = super::UiPresentationMotionDamage::between(self.screen.showing(), geometry);
        let sample = super::UiPresentationMotionSampleReceipt::from_track_sample(
            self.track,
            tick,
            presentation,
            geometry,
            opacity_units,
            posture,
            damage,
        )?;
        self.current_geometry = geometry;
        self.current_opacity_units = opacity_units;
        self.current = sample;
        self.screen = UiTrackScreen::OnScreen { showing: geometry };
        Ok(sample)
    }

    pub(super) fn rebind_published_presentation(
        &mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) {
        self.track = self.track.rebind_published_presentation(presentation);
        self.current = self.current.rebind_presentation_basis(presentation);
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
