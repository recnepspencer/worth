use std::collections::BTreeMap;

mod presented_samples;

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
}

#[must_use = "prepared motion samples must be committed after presentation or discarded"]
pub(crate) struct UiPreparedMotionSampling {
    successor: UiMountedMotionSampler,
    receipt: super::UiPresentationMotionSamplingReceipt,
}

impl UiPreparedMotionSampling {
    pub(crate) const fn receipt(&self) -> &super::UiPresentationMotionSamplingReceipt {
        &self.receipt
    }

    pub(crate) fn with_presented_basis(
        mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<Self, super::UiPresentationGeometrySamplingDenial> {
        for sample in self.receipt.samples.iter_mut() {
            *sample = sample.with_presentation_basis(presentation)?;
            if let Some(state) = self.successor.tracks.get_mut(&sample.target()) {
                state.current = Some(*sample);
            }
        }
        Ok(self)
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
        }
    }
}

impl UiMountedMotionSampler {
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
    }

    pub(crate) fn install(
        &mut self,
        receipt: crate::runtime::motion::UiMotionCommitReceipt,
    ) -> Result<super::UiPresentationMotionInstallationReceipt, UiPresentationMotionSamplingDenial>
    {
        let track = receipt.track();
        let target = track.target();
        if !self.tracks.contains_key(&target) && self.tracks.len() == MAX_PRESENTATION_TRACKS {
            let settled = self
                .tracks
                .iter()
                .find_map(|(target, state)| (!state.active).then_some(*target));
            if let Some(settled) = settled {
                self.tracks.remove(&settled);
            } else {
                return self.deny(UiPresentationMotionSamplingDenial::TrackCapacityExceeded);
            }
        }
        let current = self.tracks.get(&target).and_then(|state| {
            state
                .active
                .then_some((state.current_geometry, state.current_opacity_units))
        });
        match super::interruption::resolve(track, current, self.reduced_motion) {
            super::interruption::UiPresentationMotionInstallation::Install {
                geometry,
                opacity_units,
                duration_ticks,
            } => {
                let state = match super::track_sampling::UiPresentationTrackState::new(
                    track,
                    None,
                    geometry,
                    opacity_units,
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

    pub(crate) fn prepare_tick(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<UiPreparedMotionSampling, UiPresentationMotionSamplingDenial> {
        if self.last_tick.is_some_and(|previous| tick <= previous) {
            return self.deny(UiPresentationMotionSamplingDenial::NonMonotonicTick);
        }
        let mut successor = self.clone();
        match successor.sample_tick(tick, presentation) {
            Ok(receipt) => Ok(UiPreparedMotionSampling { successor, receipt }),
            Err(denial) => self.deny(denial),
        }
    }

    pub(crate) fn reject_presentation_truth_unavailable(
        &mut self,
    ) -> Result<UiPreparedMotionSampling, UiPresentationMotionSamplingDenial> {
        self.deny(UiPresentationMotionSamplingDenial::PresentationTruthUnavailable)
    }

    pub(crate) fn commit_prepared(
        &mut self,
        prepared: UiPreparedMotionSampling,
    ) -> super::UiPresentationMotionSamplingReceipt {
        let UiPreparedMotionSampling { successor, receipt } = prepared;
        *self = successor;
        receipt
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
            if self.reduced_motion == super::UiPresentationReducedMotionPosture::Reduce
                && state.track.declaration().reduced_motion()
                    == crate::runtime::motion::UiMotionReducedMotionPolicy::SystemRespecting
            {
                if state.track.declaration().decorative() {
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
                state.shorten_system_reduced_motion();
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
        let target = self.tracks.iter().find_map(|(target, state)| {
            (!state.active && state.queued.is_none() && state.track.identity() == track)
                .then_some(*target)
        });
        target.is_some_and(|target| self.tracks.remove(&target).is_some())
    }

    pub(crate) fn retire_rebound_track(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        let target = self.tracks.iter().find_map(|(target, state)| {
            (state.queued.is_none() && state.track.identity() == track).then_some(*target)
        });
        target.is_some_and(|target| self.tracks.remove(&target).is_some())
    }

    pub(crate) fn contains_track(
        &self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.tracks
            .values()
            .any(|state| state.track.identity() == track)
    }

    /// Tracks the sampler still retains, active or not. Retention is what makes
    /// a zero `tracks_considered` meaningful rather than vacuous.
    pub(crate) fn retained_track_count(&self) -> usize {
        self.tracks.len()
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
