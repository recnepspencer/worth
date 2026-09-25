//! The tracks a Motion sampler retains, and every edit its owner makes to them.
//!
//! A tick is sampled from a copy of the tracks taken when it is prepared, and
//! lands only once its presentation completes. The owner keeps installing,
//! retiring, rebasing and rebinding tracks in between. Each of those edits is
//! recorded as part of the mutation that makes it, because the fields are
//! private to this table, and the landing reconciles the record by an
//! exhaustive match. A new kind of edit does not compile until its landing is
//! decided.

use std::collections::{BTreeMap, BTreeSet};

use super::super::track_sampling::UiPresentationTrackState;
use crate::runtime::motion::{UiMotionTargetIdentity, UiMotionTrackIdentity};
use worth_ui_host_contract::{UiHostObservationPresentationBasis, UiSemanticSurfaceIdentity};

/// One change the Motion owner made to its tracks outside a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
enum UiMotionOwnerEdit {
    /// A track was installed at the target, over any track it held.
    Installed(UiMotionTargetIdentity),
    /// The target's track was retired.
    Retired(UiMotionTargetIdentity),
    /// The target's Scroll track restarted from a newer presented extent.
    ExtentRebased(UiMotionTargetIdentity),
    /// The host was found already showing the target's published entrance.
    EntranceAccepted(UiMotionTargetIdentity),
    /// A publication rebound every track of `surface` to `presentation`.
    Rebound {
        surface: UiSemanticSurfaceIdentity,
        presentation: UiHostObservationPresentationBasis,
    },
}

/// The tracks a sampler retains, with every edit a tick still in flight may
/// need to reconcile.
pub(super) struct UiMotionTrackTable {
    tracks: BTreeMap<UiMotionTargetIdentity, UiPresentationTrackState>,
    /// Each edit with the number of ticks prepared before it. A tick lands
    /// the edits made since its own preparation.
    edits: Vec<(u64, UiMotionOwnerEdit)>,
    /// How many ticks have been prepared.
    prepared: u64,
    /// The preparation of the latest tick landed. A tick prepared before it
    /// is older than what the tracks already show and lands nothing.
    landed: u64,
}

/// The tracks one prepared tick samples: the table as it stood when the tick
/// was prepared. Only the tick changes them, and they return to the table
/// only through [`UiMotionTrackTable::land`].
pub(super) struct UiMotionTickTracks {
    tracks: BTreeMap<UiMotionTargetIdentity, UiPresentationTrackState>,
    prepared: u64,
}

impl UiMotionTrackTable {
    pub(super) const fn new() -> Self {
        Self {
            tracks: BTreeMap::new(),
            edits: Vec::new(),
            prepared: 0,
            landed: 0,
        }
    }

    pub(super) fn get(&self, target: &UiMotionTargetIdentity) -> Option<&UiPresentationTrackState> {
        self.tracks.get(target)
    }

    pub(super) fn entries(
        &self,
    ) -> impl Iterator<Item = (&UiMotionTargetIdentity, &UiPresentationTrackState)> {
        self.tracks.iter()
    }

    pub(super) fn states(&self) -> impl Iterator<Item = &UiPresentationTrackState> {
        self.tracks.values()
    }

    pub(super) fn len(&self) -> usize {
        self.tracks.len()
    }

    pub(super) fn holds(&self, target: &UiMotionTargetIdentity) -> bool {
        self.tracks.contains_key(target)
    }

    /// Installs `state` at `target`, over any track it held.
    pub(super) fn install(
        &mut self,
        target: UiMotionTargetIdentity,
        state: UiPresentationTrackState,
    ) {
        self.record(UiMotionOwnerEdit::Installed(target));
        self.tracks.insert(target, state);
    }

    /// Retires `target`'s track, if it holds one.
    pub(super) fn retire(
        &mut self,
        target: UiMotionTargetIdentity,
    ) -> Option<UiPresentationTrackState> {
        let retired = self.tracks.remove(&target)?;
        self.record(UiMotionOwnerEdit::Retired(target));
        Some(retired)
    }

    /// Retires every track, returning how many there were.
    pub(super) fn retire_all(&mut self) -> usize {
        let targets = self.tracks.keys().copied().collect::<Vec<_>>();
        targets
            .into_iter()
            .filter_map(|target| self.retire(target))
            .count()
    }

    /// Restarts `target`'s Scroll track from its newer presented extent.
    pub(super) fn rebase_extent(
        &mut self,
        target: UiMotionTargetIdentity,
        tick: u64,
    ) -> Result<(), super::super::UiPresentationGeometrySamplingDenial> {
        self.record(UiMotionOwnerEdit::ExtentRebased(target));
        self.tracks
            .get_mut(&target)
            .expect("extent retarget was installed")
            .rebase_presented_extent(tick)
    }

    /// Records that the host already shows the entrance `sample` of the track
    /// just installed for it.
    pub(super) fn accept_entrance(
        &mut self,
        sample: super::super::UiPresentationMotionSampleReceipt,
    ) {
        let target = sample.target();
        self.tracks
            .get_mut(&target)
            .expect("published entrance was installed from its exact Motion commit")
            .accept_published_entrance(sample);
        self.record(UiMotionOwnerEdit::EntranceAccepted(target));
    }

    /// Rebinds every track of `surface` to the presentation a publication
    /// bound it to. A rebind that reaches no track changes nothing a tick
    /// could land, so it leaves no record.
    pub(super) fn rebind(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        presentation: UiHostObservationPresentationBasis,
    ) {
        let mut rebound = false;
        for state in self
            .tracks
            .values_mut()
            .filter(|state| state.track.target().semantic_surface() == surface)
        {
            state.rebind_published_presentation(presentation);
            rebound = true;
        }
        if rebound {
            self.record(UiMotionOwnerEdit::Rebound {
                surface,
                presentation,
            });
        }
    }

    /// Records `edit` as made after every tick prepared so far. The record
    /// holds one entry for each target and surface: a later edit supersedes
    /// an earlier one of the same kind, since a rebind overwrites the
    /// presentation and a target edit only marks the target as changed.
    fn record(&mut self, edit: UiMotionOwnerEdit) {
        self.edits.retain(|(_, recorded)| match (recorded, &edit) {
            (
                UiMotionOwnerEdit::Rebound { surface, .. },
                UiMotionOwnerEdit::Rebound { surface: next, .. },
            ) => surface != next,
            (recorded, edit) => recorded != edit,
        });
        self.edits.push((self.prepared, edit));
    }

    /// The tracks a tick prepared now samples.
    pub(super) fn prepare_tick(&mut self) -> UiMotionTickTracks {
        self.prepared += 1;
        UiMotionTickTracks {
            tracks: self.tracks.clone(),
            prepared: self.prepared,
        }
    }

    /// Lands the tracks a presented tick sampled, reconciling every edit made
    /// since it was prepared. A target the owner installed, retired, rebased
    /// or accepted keeps its live state: the tick sampled the track that was
    /// there before. `depart` resolves a live track against what the tick
    /// presented of its predecessor. A rebind relabels the presentation a
    /// track is read against and leaves its motion alone, so every other
    /// target takes it on the sample the tick presented. Returns the tracks
    /// whose samples the landing dropped. A tick prepared before the latest
    /// landed one lands nothing, and every track it sampled is dropped.
    pub(super) fn land(
        &mut self,
        sampled: UiMotionTickTracks,
        depart: impl Fn(UiPresentationTrackState, &UiPresentationTrackState) -> UiPresentationTrackState,
    ) -> Vec<UiMotionTrackIdentity> {
        let UiMotionTickTracks {
            tracks: mut successor,
            prepared,
        } = sampled;
        if prepared < self.landed {
            return successor
                .values()
                .map(|state| state.track.identity())
                .collect();
        }
        self.landed = prepared;
        let mut changed = BTreeSet::new();
        let mut rebinds = Vec::new();
        // Edits made before this tick was prepared are already in what it
        // sampled, and no tick prepared earlier can land after it.
        self.edits.retain(|(before, _)| *before >= prepared);
        for (_, edit) in &self.edits {
            match *edit {
                UiMotionOwnerEdit::Installed(target)
                | UiMotionOwnerEdit::Retired(target)
                | UiMotionOwnerEdit::ExtentRebased(target)
                | UiMotionOwnerEdit::EntranceAccepted(target) => {
                    changed.insert(target);
                }
                UiMotionOwnerEdit::Rebound {
                    surface,
                    presentation,
                } => rebinds.push((surface, presentation)),
            }
        }
        for (surface, presentation) in rebinds {
            for (target, state) in &mut successor {
                if target.semantic_surface() == surface && !changed.contains(target) {
                    state.rebind_published_presentation(presentation);
                }
            }
        }
        let mut superseded = Vec::new();
        for target in changed {
            let sampled = successor.remove(&target);
            let live = self.tracks.remove(&target).map(|live| match &sampled {
                Some(sampled) => depart(live, sampled),
                None => live,
            });
            if let Some(sampled) = sampled {
                superseded.push(sampled.track.identity());
            }
            if let Some(live) = live {
                successor.insert(target, live);
            }
        }
        self.tracks = successor;
        superseded
    }
}

impl UiMotionTickTracks {
    /// The running tracks this tick samples.
    pub(super) fn running_mut(&mut self) -> impl Iterator<Item = &mut UiPresentationTrackState> {
        self.tracks.values_mut().filter(|state| state.is_running())
    }

    /// Records `sample` as what the tick presented of its target.
    pub(super) fn present(&mut self, sample: super::super::UiPresentationMotionSampleReceipt) {
        if let Some(state) = self.tracks.get_mut(&sample.target()) {
            state.current = sample;
        }
    }
}
