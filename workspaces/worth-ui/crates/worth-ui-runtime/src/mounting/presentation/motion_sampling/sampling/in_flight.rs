//! A tick is sampled from the tracks as they stood when it was prepared and
//! lands only when its presentation completes. The owner keeps installing,
//! retiring and rebinding tracks in between, and the landing must not undo any
//! of it.

use super::super::interruption::{
    resolve, UiPresentationInterruptedSample, UiPresentationMotionInstallation,
};
use super::super::track_sampling::UiPresentationTrackState;
#[cfg(doc)]
use super::track_table::UiMotionTrackTable;
use super::{
    UiMountedMotionSampler, UiPreparedMotionSampling, UiPresentationMotionSamplingDenial,
    UiPresentedMotionSampling,
};
use crate::runtime::motion::UiMotionTrackIdentity;

impl UiMountedMotionSampler {
    pub(crate) fn prepare_tick(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<UiPreparedMotionSampling, UiPresentationMotionSamplingDenial> {
        if self.last_tick.is_some_and(|previous| tick <= previous) {
            return self.deny(UiPresentationMotionSamplingDenial::NonMonotonicTick);
        }
        let mut successor = self.tracks.prepare_tick();
        match successor.sample(tick, presentation, self.reduced_motion) {
            Ok(receipt) => Ok(UiPreparedMotionSampling {
                successor,
                tick,
                receipt,
                prepared_at: presentation,
            }),
            Err(denial) => self.deny(denial),
        }
    }

    /// Lands a presented tick. The tracks take what the tick sampled, because
    /// that is what the host now shows, reconciled with every edit the owner
    /// made while the tick was in flight; see [`UiMotionTrackTable::land`].
    /// What the tick sampled of a target the owner changed is dropped with
    /// it, so no terminal is claimed for a track the Motion owner no longer
    /// holds. Everything else the sampler holds is the owner's alone.
    pub(crate) fn commit_prepared(
        &mut self,
        prepared: UiPresentedMotionSampling,
    ) -> super::super::UiPresentationMotionSamplingReceipt {
        let UiPresentedMotionSampling {
            successor,
            tick,
            mut receipt,
        } = prepared;
        self.last_tick = self.last_tick.max(Some(tick));
        let reduced_motion = self.reduced_motion;
        let superseded = self.tracks.land(successor, |live, sampled| {
            departing_from_presented(live, sampled, tick, reduced_motion)
        });
        receipt.drop_tracks(&superseded);
        receipt
    }

    /// Removes the track state holding `track` when `retirable` admits it.
    pub(super) fn retire_track_where(
        &mut self,
        track: UiMotionTrackIdentity,
        retirable: impl Fn(&UiPresentationTrackState) -> bool,
    ) -> bool {
        let Some(target) = self.tracks.entries().find_map(|(target, state)| {
            (retirable(state) && state.track.identity() == track).then_some(*target)
        }) else {
            return false;
        };
        self.tracks.retire(target).is_some()
    }
}

/// The owner installed `live` while a tick was in flight, interrupting the
/// track as it stood before the tick. The tick has since reached the screen,
/// so the interruption is resolved again from what it presented at `tick`;
/// departing from the earlier sample would pull the content back by a frame's
/// worth of motion. The departure is that presented frame, so the track is
/// recorded as on screen there; left unpresented, it hid the frame from the
/// Scroll settle and the displayed pose fell behind the host. A track already
/// sampled, rebased or accepted since its installation keeps the start it has.
fn departing_from_presented(
    live: UiPresentationTrackState,
    sampled: &UiPresentationTrackState,
    tick: u64,
    reduced_motion: super::super::UiPresentationReducedMotionPosture,
) -> UiPresentationTrackState {
    if !live.is_unstarted() || live.track.identity() == sampled.track.identity() {
        return live;
    }
    let presented = sampled
        .is_running()
        .then(|| UiPresentationInterruptedSample {
            tick,
            geometry: sampled.current_geometry(),
            opacity_units: sampled.current_opacity_units,
            outgoing: sampled.outgoing_curve(tick),
        });
    match resolve(live.track, presented, reduced_motion) {
        UiPresentationMotionInstallation::Install {
            geometry,
            opacity_units,
            start_velocity,
            duration_ticks,
            start_tick,
        } => UiPresentationTrackState::new(
            live.track,
            start_tick,
            geometry,
            opacity_units,
            start_velocity,
            duration_ticks,
        )
        .map_or(live, |mut departed| {
            departed.depart_on_screen(sampled);
            departed
        }),
        UiPresentationMotionInstallation::SnapToTarget => live,
    }
}
