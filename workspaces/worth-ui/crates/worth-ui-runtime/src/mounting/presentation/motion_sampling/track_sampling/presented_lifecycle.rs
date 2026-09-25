//! How a track's record of what the host shows follows the host's
//! presentation lifecycle: extent rebases, published entrances, retarget
//! departures, and binding rebinds.
use super::{UiTrackGeometry, UiTrackMotion, UiTrackSamplePlace, UiTrackScreen};

impl super::UiPresentationTrackState {
    pub(in crate::mounting::presentation::motion_sampling) fn rebase_presented_extent(
        &mut self,
        tick: u64,
    ) -> Result<(), super::super::UiPresentationGeometrySamplingDenial> {
        let geometry = self
            .track
            .predecessor_geometry()
            .map(UiTrackSamplePlace::Published);
        let damage = geometry.map(UiTrackSamplePlace::damage_components);
        let sample = super::super::UiPresentationMotionSampleReceipt::from_track_sample(
            self.track,
            tick,
            self.track.successor_presentation(),
            geometry,
            self.current_opacity_units,
            super::super::UiPresentationMotionSamplePosture::Active,
            super::super::UiPresentationMotionDamage::between(damage, damage),
        )?;
        let target = self.track.successor_geometry();
        if let UiTrackMotion::Running(curve) = &mut self.motion {
            curve.start_tick = Some(tick);
            curve.start_geometry = sample.geometry();
            curve.start_velocity = curve
                .toward(target)
                .bounded_velocity(curve.start_velocity, curve.duration_ticks);
        }
        self.current = sample;
        self.screen = UiTrackScreen::OnScreen {
            showing: sample.geometry().map(UiTrackGeometry::Accepted),
        };
        Ok(())
    }

    /// The entrance sample is accepted as this track's current sample, but it
    /// is delayed and carries no opacity: the frame that published it drew the
    /// overlay at its successor geometry, not at the entrance offset. Claiming
    /// the entrance geometry as presented erased the only record of what the
    /// host still shows, so the first moving sample damaged its own destination
    /// twice and left the published successor on screen.
    pub(in crate::mounting::presentation::motion_sampling) fn accept_published_entrance(
        &mut self,
        sample: super::super::UiPresentationMotionSampleReceipt,
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
    pub(in crate::mounting::presentation::motion_sampling) fn depart_on_screen(
        &mut self,
        on_screen: &Self,
    ) {
        if let UiTrackScreen::OnScreen { showing } = on_screen.screen {
            let departs_from_screen = match (self.current_geometry(), on_screen.current_geometry())
            {
                (Some(departure), Some(shown)) => departure.occupies_same_rect(shown),
                (departure, shown) => departure.is_none() && shown.is_none(),
            };
            if departs_from_screen {
                self.screen = UiTrackScreen::OnScreen { showing };
            }
        }
    }

    pub(in crate::mounting::presentation::motion_sampling) fn rebind_published_presentation(
        &mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) {
        self.track = self.track.rebind_published_presentation(presentation);
        self.current = self.current.rebind_presentation_basis(presentation);
    }
}
