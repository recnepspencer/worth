//! A tick is sampled from a successor cloned when it is prepared and lands only
//! when its presentation completes. The owner keeps installing, retiring and
//! rebinding tracks in between, and the landing must not undo any of it.

use super::super::interruption::{
    resolve, UiPresentationInterruptedSample, UiPresentationMotionInstallation,
};
use super::super::track_sampling::UiPresentationTrackState;
use super::{
    UiMountedMotionSampler, UiPreparedMotionSampling, UiPresentationMotionSamplingDenial,
    UiPresentedMotionSampling,
};
use crate::runtime::motion::{UiMotionTargetIdentity, UiMotionTrackIdentity};

impl UiMountedMotionSampler {
    pub(crate) fn prepare_tick(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<UiPreparedMotionSampling, UiPresentationMotionSamplingDenial> {
        if self.last_tick.is_some_and(|previous| tick <= previous) {
            return self.deny(UiPresentationMotionSamplingDenial::NonMonotonicTick);
        }
        // Only changes after this clone can diverge from it. A tick that was
        // prepared and discarded leaves nothing for this one to reconcile.
        self.changed_since_prepare.clear();
        self.rebound_since_prepare.clear();
        let mut successor = self.clone();
        match successor.sample_tick(tick, presentation) {
            Ok(receipt) => Ok(UiPreparedMotionSampling {
                successor,
                receipt,
                prepared_at: presentation,
            }),
            Err(denial) => self.deny(denial),
        }
    }

    /// Lands a presented tick. A target the owner changed while the tick was
    /// in flight keeps its live state: the successor sampled the track that
    /// was there before, and the owner has already settled that track with
    /// its replacement or retirement. What the tick sampled of such a target
    /// is dropped with it, so no terminal is claimed for a track the Motion
    /// owner no longer holds. A track the owner installed over the one the
    /// tick sampled departs from what the tick presented.
    ///
    /// Every other target keeps what the tick sampled, because that is what
    /// the host now shows. A publication that rebound it meanwhile is applied
    /// to that sample; keeping the pre-tick state instead would record the
    /// frame before the one on screen as presented.
    pub(crate) fn commit_prepared(
        &mut self,
        prepared: UiPresentedMotionSampling,
    ) -> super::super::UiPresentationMotionSamplingReceipt {
        let UiPresentedMotionSampling {
            mut successor,
            mut receipt,
        } = prepared;
        let changed = std::mem::take(&mut self.changed_since_prepare);
        for (surface, presentation) in std::mem::take(&mut self.rebound_since_prepare) {
            for (target, state) in &mut successor.tracks {
                if target.semantic_surface() == surface && !changed.contains(target) {
                    state.rebind_published_presentation(presentation);
                }
            }
        }
        let mut superseded = Vec::new();
        for target in changed {
            let sampled = successor.tracks.remove(&target);
            let live = self.tracks.remove(&target).map(|live| match &sampled {
                Some(sampled) => successor.departing_from_presented(live, sampled),
                None => live,
            });
            if let Some(sampled) = sampled {
                superseded.push(sampled.track.identity());
                superseded.extend(sampled.queued.map(|queued| queued.identity()));
            }
            if let Some(live) = live {
                successor.tracks.insert(target, live);
            }
        }
        receipt.drop_tracks(&superseded);
        *self = successor;
        receipt
    }

    /// The owner installed `live` while this tick was in flight, interrupting
    /// the track as it stood before the tick. The tick has since reached the
    /// screen, so the interruption is resolved again from what it presented;
    /// departing from the earlier sample would pull the content back by a
    /// frame's worth of motion. The departure is that presented frame, so the
    /// track is recorded as on screen there; left unpresented, it hid the
    /// frame from the Scroll settle and the displayed pose fell behind the
    /// host. A track already sampled, rebased or accepted since its
    /// installation keeps the start it has.
    fn departing_from_presented(
        &self,
        live: UiPresentationTrackState,
        sampled: &UiPresentationTrackState,
    ) -> UiPresentationTrackState {
        if !live.is_unstarted() || live.track.identity() == sampled.track.identity() {
            return live;
        }
        let tick = self.last_tick.unwrap_or(0);
        let presented = sampled.active.then(|| UiPresentationInterruptedSample {
            tick,
            geometry: sampled.current_geometry,
            opacity_units: sampled.current_opacity_units,
            outgoing: sampled.outgoing_curve(tick),
        });
        match resolve(live.track, presented, self.reduced_motion) {
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

    /// Records that the owner changed `target` outside the tick in flight.
    pub(super) fn note_owner_change(&mut self, target: UiMotionTargetIdentity) {
        self.changed_since_prepare.insert(target);
    }

    /// Removes the track state holding `track` when `retirable` admits it.
    pub(super) fn retire_track_where(
        &mut self,
        track: UiMotionTrackIdentity,
        retirable: impl Fn(&UiPresentationTrackState) -> bool,
    ) -> bool {
        let Some(target) = self.tracks.iter().find_map(|(target, state)| {
            (retirable(state) && state.track.identity() == track).then_some(*target)
        }) else {
            return false;
        };
        self.note_owner_change(target);
        self.tracks.remove(&target).is_some()
    }
}
