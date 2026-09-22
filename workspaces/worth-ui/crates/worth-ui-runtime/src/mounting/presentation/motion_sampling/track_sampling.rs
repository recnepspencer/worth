use super::curve::UiMotionCurvePhase;
use super::velocity::{UiPresentationOutgoingCurve, UiPresentationSampleVelocity};

#[derive(Clone, Copy)]
pub(super) struct UiPresentationTrackState {
    pub(super) track: crate::runtime::motion::UiCommittedMotionTrack,
    pub(super) queued: Option<crate::runtime::motion::UiCommittedMotionTrack>,
    start_tick: Option<u64>,
    start_geometry: Option<[f32; 4]>,
    start_opacity_units: u16,
    start_velocity: UiPresentationSampleVelocity,
    duration_ticks: u32,
    pub(super) current_geometry: Option<[f32; 4]>,
    presented_geometry: Option<[f32; 4]>,
    pub(super) current_opacity_units: u16,
    pub(super) current: Option<super::UiPresentationMotionSampleReceipt>,
    pub(super) presented: bool,
    pub(super) active: bool,
}

impl UiPresentationTrackState {
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
            self.current,
            Some(sample),
            "the physically accepted entrance matches the installed initial sample"
        );
        self.presented = true;
    }

    pub(super) fn new(
        track: crate::runtime::motion::UiCommittedMotionTrack,
        start_tick: Option<u64>,
        start_geometry: Option<[f32; 4]>,
        start_opacity_units: u16,
        start_velocity: UiPresentationSampleVelocity,
        duration_ticks: u32,
    ) -> Result<Self, super::UiPresentationGeometrySamplingDenial> {
        let tick = start_tick.unwrap_or(0);
        let host_presented_geometry = track
            .successor_geometry()
            .map(crate::runtime::motion::UiMotionSemanticGeometry::components);
        let current = super::UiPresentationMotionSampleReceipt::from_track_sample(
            track,
            tick,
            track.successor_presentation(),
            start_geometry,
            start_opacity_units,
            super::UiPresentationMotionSamplePosture::Delayed,
            super::UiPresentationMotionDamage::between(host_presented_geometry, start_geometry),
        )?;
        Ok(Self {
            track,
            queued: None,
            start_tick,
            start_geometry,
            start_opacity_units,
            start_velocity,
            duration_ticks,
            current_geometry: start_geometry,
            presented_geometry: host_presented_geometry,
            current_opacity_units: start_opacity_units,
            current: Some(current),
            presented: false,
            active: true,
        })
    }

    pub(super) fn terminal(
        track: crate::runtime::motion::UiCommittedMotionTrack,
        tick: u64,
    ) -> Result<Self, super::UiPresentationGeometrySamplingDenial> {
        let geometry = track
            .successor_geometry()
            .map(crate::runtime::motion::UiMotionSemanticGeometry::components);
        let opacity_units = if track.successor_visible() {
            u16::MAX
        } else {
            0
        };
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
            queued: None,
            start_tick: Some(tick),
            start_geometry: geometry,
            start_opacity_units: opacity_units,
            start_velocity: UiPresentationSampleVelocity::RESTING,
            duration_ticks: 0,
            current_geometry: geometry,
            presented_geometry: geometry,
            current_opacity_units: opacity_units,
            current: Some(current),
            presented: false,
            active: false,
        })
    }

    pub(super) fn sample(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<super::UiPresentationMotionSampleReceipt, super::UiPresentationGeometrySamplingDenial>
    {
        let start = *self.start_tick.get_or_insert(tick);
        let (normalized, posture) = self.horizon_at(tick.saturating_sub(start));
        let phase = self.phase_at(normalized);
        let target_geometry = self.target_geometry();
        let geometry = super::curve::interpolate_geometry(
            self.track.declaration().channels(),
            self.start_geometry,
            target_geometry,
            phase,
        );
        let target_opacity_units = if self.track.successor_visible() {
            u16::MAX
        } else {
            0
        };
        let opacity_units = if self
            .track
            .declaration()
            .channels()
            .contains(crate::runtime::motion::UiMotionPropertyChannel::Opacity)
        {
            super::opacity::interpolate_units(
                self.start_opacity_units,
                target_opacity_units,
                phase.scalar_progress(),
            )
        } else {
            target_opacity_units
        };
        let damage = super::UiPresentationMotionDamage::between(self.presented_geometry, geometry);
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
        self.presented_geometry = geometry;
        self.current_opacity_units = opacity_units;
        self.current = Some(sample);
        self.presented = true;
        Ok(sample)
    }

    /// Where `elapsed` since this track's start sits on its declared horizon,
    /// and what that makes the sample's posture.
    fn horizon_at(&self, elapsed: u64) -> (f32, super::UiPresentationMotionSamplePosture) {
        let delay = u64::from(self.track.declaration().delay_ticks());
        let Some(active_elapsed) = elapsed.checked_sub(delay) else {
            return (0.0, super::UiPresentationMotionSamplePosture::Delayed);
        };
        let duration = u64::from(self.duration_ticks.max(1));
        let normalized = (active_elapsed as f64 / duration as f64).min(1.0) as f32;
        let posture = if normalized >= 1.0 {
            super::UiPresentationMotionSamplePosture::Terminal
        } else {
            super::UiPresentationMotionSamplePosture::Active
        };
        (normalized, posture)
    }

    fn phase_at(&self, normalized: f32) -> UiMotionCurvePhase {
        UiMotionCurvePhase::new(
            self.track.declaration().easing(),
            normalized,
            self.duration_ticks,
            self.start_velocity.components(),
        )
    }

    fn target_geometry(&self) -> Option<[f32; 4]> {
        self.track
            .successor_geometry()
            .map(crate::runtime::motion::UiMotionSemanticGeometry::components)
            .or(self.start_geometry)
    }

    /// The curve this track is running at `tick`, for a successor about to
    /// displace it. A track that never started, is still inside its delay, or
    /// has no geometry basis is not moving and carries no curve.
    pub(super) fn outgoing_curve(&self, tick: u64) -> Option<UiPresentationOutgoingCurve> {
        let start_geometry = self.start_geometry?;
        let (normalized, posture) = self.horizon_at(tick.saturating_sub(self.start_tick?));
        if posture == super::UiPresentationMotionSamplePosture::Delayed {
            return None;
        }
        Some(UiPresentationOutgoingCurve::interrupted_at(
            self.track.declaration().channels(),
            self.phase_at(normalized),
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
        if let Some(queued) = self.queued.take() {
            self.track = queued;
        }
        let geometry = self
            .track
            .successor_geometry()
            .map(crate::runtime::motion::UiMotionSemanticGeometry::components);
        let opacity_units = if self.track.successor_visible() {
            u16::MAX
        } else {
            0
        };
        let damage = super::UiPresentationMotionDamage::between(self.presented_geometry, geometry);
        let sample = super::UiPresentationMotionSampleReceipt::from_track_sample(
            self.track,
            tick,
            presentation,
            geometry,
            opacity_units,
            super::UiPresentationMotionSamplePosture::Terminal,
            damage,
        )?;
        self.current_geometry = geometry;
        self.presented_geometry = geometry;
        self.current_opacity_units = opacity_units;
        self.current = Some(sample);
        self.presented = true;
        self.active = false;
        self.start_velocity = UiPresentationSampleVelocity::RESTING;
        Ok(sample)
    }

    pub(super) fn shorten_system_reduced_motion(&mut self) {
        self.duration_ticks = self.duration_ticks.min(1);
    }

    /// Hand the track over to its queued successor at `tick`. The successor
    /// departs from the sample this track just produced: its position, its
    /// opacity, and the rate the displaced curve was moving at.
    pub(super) fn begin_queued(
        &mut self,
        track: crate::runtime::motion::UiCommittedMotionTrack,
        tick: u64,
    ) {
        let start_velocity = self.outgoing_curve(tick).map_or(
            UiPresentationSampleVelocity::RESTING,
            UiPresentationSampleVelocity::of_outgoing_curve,
        );
        self.track = track;
        self.start_tick = Some(tick);
        self.start_geometry = self.current_geometry;
        self.start_opacity_units = self.current_opacity_units;
        self.start_velocity = start_velocity;
        self.duration_ticks = track.declaration().duration_ticks();
        self.active = true;
    }

    pub(super) fn rebind_published_presentation(
        &mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) {
        self.track = self.track.rebind_published_presentation(presentation);
        self.queued = self
            .queued
            .map(|track| track.rebind_published_presentation(presentation));
        self.current = self
            .current
            .map(|sample| sample.rebind_presentation_basis(presentation));
    }
}
