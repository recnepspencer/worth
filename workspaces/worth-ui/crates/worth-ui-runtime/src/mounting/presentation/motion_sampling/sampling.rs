use std::collections::{BTreeMap, BTreeSet};

mod in_flight;
mod presented_samples;
mod scroll_group;

pub(super) const MAX_PRESENTATION_TRACKS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPresentationMotionSamplingDenial {
    NonMonotonicTick,
    TrackCapacityExceeded,
    PresentationTruthUnavailable,
    InvalidSampleGeometry(super::UiPresentationGeometrySamplingDenial),
}

#[derive(Clone)]
pub(crate) struct UiMountedMotionSampler {
    tracks: BTreeMap<
        crate::runtime::motion::UiMotionTargetIdentity,
        super::track_sampling::UiPresentationTrackState,
    >,
    last_tick: Option<u64>,
    reduced_motion: super::UiPresentationReducedMotionPosture,
    denial_count: u64,
    last_denial: Option<UiPresentationMotionSamplingDenial>,
    /// Targets the owner installed or retired since the last tick was
    /// prepared. That tick was sampled from a clone taken before these
    /// changes, so the commit that lands it keeps their live state.
    changed_since_prepare: BTreeSet<crate::runtime::motion::UiMotionTargetIdentity>,
    /// Publications that rebound a surface's tracks since the last tick was
    /// prepared. A rebind relabels the presentation a track is read against
    /// and leaves its motion alone, so the landing tick takes it too.
    rebound_since_prepare: Vec<(
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        worth_ui_host_contract::UiHostObservationPresentationBasis,
    )>,
}

#[must_use = "prepared motion samples must be committed after presentation or discarded"]
pub(crate) struct UiPreparedMotionSampling {
    successor: UiMountedMotionSampler,
    receipt: super::UiPresentationMotionSamplingReceipt,
    prepared_at: worth_ui_host_contract::UiHostObservationPresentationBasis,
}

/// A prepared tick that may land: a host acknowledgement proved its samples
/// on the surface generation they were sampled for, or it sampled nothing.
/// [`UiMountedMotionSampler::commit_prepared`] accepts nothing else.
#[must_use = "presented motion samples must be committed"]
pub(crate) struct UiPresentedMotionSampling {
    successor: UiMountedMotionSampler,
    receipt: super::UiPresentationMotionSamplingReceipt,
}

/// What a prepared tick still needs before it may land.
pub(crate) enum UiPreparedMotionWork {
    /// The tick sampled nothing, so there is nothing for a host to present.
    Unsampled(UiPresentedMotionSampling),
    /// The tick's samples land only through a host acknowledgement.
    NeedsPresentation(UiPreparedMotionSampling),
}

impl UiPreparedMotionSampling {
    pub(crate) const fn receipt(&self) -> &super::UiPresentationMotionSamplingReceipt {
        &self.receipt
    }

    /// Lands the samples at the displayed basis `witness` proved. A witness for
    /// another host surface, binding, or frame proves nothing about this tick.
    pub(crate) fn into_presented(
        mut self,
        witness: &crate::mounting::presentation::UiPresentedSurfaceWitness,
    ) -> Option<UiPresentedMotionSampling> {
        let presentation = witness.displayed_basis().basis();
        if presentation.host_surface() != self.prepared_at.host_surface()
            || presentation.binding() != self.prepared_at.binding()
            || presentation.frame() != self.prepared_at.frame()
        {
            return None;
        }
        for sample in self.receipt.samples.iter_mut() {
            *sample = sample.with_presentation_basis(presentation).ok()?;
            if let Some(state) = self.successor.tracks.get_mut(&sample.target()) {
                state.current = Some(*sample);
            }
        }
        Some(UiPresentedMotionSampling {
            successor: self.successor,
            receipt: self.receipt,
        })
    }

    /// A tick that sampled nothing has nothing for a host to present.
    pub(crate) fn into_work(self) -> UiPreparedMotionWork {
        if !self.receipt.samples().is_empty() {
            return UiPreparedMotionWork::NeedsPresentation(self);
        }
        UiPreparedMotionWork::Unsampled(UiPresentedMotionSampling {
            successor: self.successor,
            receipt: self.receipt,
        })
    }

    /// Presents this tick at the basis it was prepared against, through the
    /// runtime's own issue, acknowledgement, and admission boundary.
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn presented_for_certification(self) -> UiPresentedMotionSampling {
        let witness = crate::mounting::presentation::presented_surface_witness_for_certification(
            self.prepared_at,
        );
        self.into_presented(&witness)
            .expect("a witness at the prepared basis presents the tick")
    }
}

impl Default for UiMountedMotionSampler {
    fn default() -> Self {
        Self {
            tracks: BTreeMap::new(),
            last_tick: None,
            reduced_motion: super::UiPresentationReducedMotionPosture::NoPreference,
            denial_count: 0,
            last_denial: None,
            changed_since_prepare: BTreeSet::new(),
            rebound_since_prepare: Vec::new(),
        }
    }
}

impl UiMountedMotionSampler {
    pub(in crate::mounting) fn accept_published_entrance(
        &mut self,
        sample: super::UiPresentationMotionSampleReceipt,
    ) {
        self.tracks
            .get_mut(&sample.target())
            .expect("published entrance was installed from its exact Motion commit")
            .accept_published_entrance(sample);
    }

    pub(in crate::mounting) fn retained_targets(
        &self,
    ) -> Vec<crate::runtime::motion::UiMotionTargetIdentity> {
        self.tracks.keys().copied().collect()
    }

    pub(in crate::mounting) fn rebind_published_presentation(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) {
        for state in self
            .tracks
            .values_mut()
            .filter(|state| state.track.target().semantic_surface() == surface)
        {
            state.rebind_published_presentation(presentation);
        }
        self.rebound_since_prepare.push((surface, presentation));
    }

    pub(crate) fn install(
        &mut self,
        receipt: crate::runtime::motion::UiMotionCommitReceipt,
    ) -> Result<super::UiPresentationMotionInstallationReceipt, UiPresentationMotionSamplingDenial>
    {
        let track = receipt.track();
        let target = track.target();
        self.note_owner_change(target);
        if !self.tracks.contains_key(&target) && self.tracks.len() == MAX_PRESENTATION_TRACKS {
            let settled = self
                .tracks
                .iter()
                .find_map(|(target, state)| (!state.active).then_some(*target));
            if let Some(settled) = settled {
                self.note_owner_change(settled);
                self.tracks.remove(&settled);
            } else {
                return self.deny(UiPresentationMotionSamplingDenial::TrackCapacityExceeded);
            }
        }
        let interruption_tick = self.last_tick.unwrap_or(0);
        let current = self.tracks.get(&target).and_then(|state| {
            state
                .active
                .then(|| super::interruption::UiPresentationInterruptedSample {
                    tick: interruption_tick,
                    geometry: state.current_geometry,
                    opacity_units: state.current_opacity_units,
                    outgoing: state.outgoing_curve(interruption_tick),
                })
        });
        match super::interruption::resolve(track, current, self.reduced_motion) {
            super::interruption::UiPresentationMotionInstallation::Install {
                geometry,
                opacity_units,
                start_velocity,
                duration_ticks,
                start_tick,
            } => {
                let state = match super::track_sampling::UiPresentationTrackState::new(
                    track,
                    start_tick,
                    geometry,
                    opacity_units,
                    start_velocity,
                    duration_ticks,
                ) {
                    Ok(state) => state,
                    Err(denial) => {
                        return self.deny(
                            UiPresentationMotionSamplingDenial::InvalidSampleGeometry(denial),
                        )
                    }
                };
                let sample = state.current;
                self.tracks.insert(target, state);
                Ok(super::UiPresentationMotionInstallationReceipt::new(
                    sample, None,
                ))
            }
            super::interruption::UiPresentationMotionInstallation::SnapToTarget => {
                let state = match super::track_sampling::UiPresentationTrackState::terminal(
                    track,
                    self.last_tick.unwrap_or(0),
                ) {
                    Ok(state) => state,
                    Err(denial) => {
                        return self.deny(
                            UiPresentationMotionSamplingDenial::InvalidSampleGeometry(denial),
                        )
                    }
                };
                let sample = state.current;
                self.tracks.insert(target, state);
                Ok(super::UiPresentationMotionInstallationReceipt::new(
                    sample,
                    Some(super::UiPresentationMotionTerminalRequest::new(
                        track.identity(),
                        crate::runtime::motion::UiMotionTerminalCause::SnappedToTarget,
                    )),
                ))
            }
        }
    }

    pub(crate) fn reject_presentation_truth_unavailable(
        &mut self,
    ) -> Result<UiPreparedMotionSampling, UiPresentationMotionSamplingDenial> {
        self.deny(UiPresentationMotionSamplingDenial::PresentationTruthUnavailable)
    }

    fn sample_tick(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<super::UiPresentationMotionSamplingReceipt, UiPresentationMotionSamplingDenial>
    {
        self.last_tick = Some(tick);
        let mut samples = Vec::new();
        let mut terminals = Vec::new();
        let mut considered = 0;
        for state in self.tracks.values_mut().filter(|state| state.active) {
            considered += 1;
            if !same_surface_binding(state.track.successor_presentation(), presentation) {
                state.active = false;
                terminals.push(super::UiPresentationMotionTerminalRequest::new(
                    state.queued.unwrap_or(state.track).identity(),
                    crate::runtime::motion::UiMotionTerminalCause::ReboundAway,
                ));
                continue;
            }
            if self.reduced_motion == super::UiPresentationReducedMotionPosture::Reduce {
                let declaration = state.track.declaration();
                if declaration.settles_directly_under_reduced_motion() {
                    let sample = state
                        .snap_system_reduced_motion(tick, presentation)
                        .map_err(UiPresentationMotionSamplingDenial::InvalidSampleGeometry)?;
                    samples.push(sample);
                    terminals.push(super::UiPresentationMotionTerminalRequest::new(
                        sample.track(),
                        crate::runtime::motion::UiMotionTerminalCause::SnappedToTarget,
                    ));
                    continue;
                }
                if declaration.shortens_under_reduced_motion() {
                    state.shorten_system_reduced_motion();
                }
            }
            let sample = match state.sample(tick, presentation) {
                Ok(sample) => sample,
                Err(denial) => {
                    return Err(UiPresentationMotionSamplingDenial::InvalidSampleGeometry(
                        denial,
                    ));
                }
            };
            samples.push(sample);
            if sample.posture() == super::UiPresentationMotionSamplePosture::Terminal {
                if let Some(queued) = state.queued.take() {
                    state.begin_queued(queued, tick);
                } else {
                    state.active = false;
                    terminals.push(super::UiPresentationMotionTerminalRequest::new(
                        state.track.identity(),
                        crate::runtime::motion::UiMotionTerminalCause::Completed,
                    ));
                }
            }
        }
        Ok(super::UiPresentationMotionSamplingReceipt::new(
            samples, terminals, considered,
        ))
    }

    pub(crate) fn retire_terminal_track(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.retire_track_where(track, |state| !state.active && state.queued.is_none())
    }

    pub(crate) fn retire_rebound_track(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.retire_track_where(track, |state| state.queued.is_none())
    }

    pub(crate) fn contains_track(
        &self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.tracks
            .values()
            .any(|state| state.track.identity() == track)
    }

    pub(crate) fn has_active_tracks(&self) -> bool {
        self.tracks.values().any(|track| track.active)
    }

    pub(crate) fn set_reduced_motion(
        &mut self,
        posture: super::UiPresentationReducedMotionPosture,
    ) {
        self.reduced_motion = posture;
    }

    pub(crate) const fn reduced_motion(&self) -> super::UiPresentationReducedMotionPosture {
        self.reduced_motion
    }

    pub(crate) fn shutdown(&mut self) -> usize {
        let retained = self.tracks.len();
        self.tracks.clear();
        retained
    }

    fn deny<T>(
        &mut self,
        denial: UiPresentationMotionSamplingDenial,
    ) -> Result<T, UiPresentationMotionSamplingDenial> {
        self.denial_count = self.denial_count.saturating_add(1);
        self.last_denial = Some(denial);
        Err(denial)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn certification_observation(
        &self,
    ) -> (
        usize,
        usize,
        Option<u64>,
        Option<super::UiPresentationMotionSampleReceipt>,
        u64,
        Option<UiPresentationMotionSamplingDenial>,
    ) {
        (
            self.tracks.values().filter(|track| track.active).count(),
            self.tracks.len(),
            self.last_tick,
            self.tracks.values().find_map(|track| track.current),
            self.denial_count,
            self.last_denial,
        )
    }
}

fn same_surface_binding(
    left: worth_ui_host_contract::UiHostObservationPresentationBasis,
    right: worth_ui_host_contract::UiHostObservationPresentationBasis,
) -> bool {
    left.host_surface() == right.host_surface() && left.binding() == right.binding()
}
