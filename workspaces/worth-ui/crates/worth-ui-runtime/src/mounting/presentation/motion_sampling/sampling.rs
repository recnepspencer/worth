#[cfg(worth_ui_compile_probe)]
mod compile_probe;
mod in_flight;
mod presented_samples;
mod scroll_group;
mod tick;
mod track_table;

pub(super) const MAX_PRESENTATION_TRACKS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPresentationMotionSamplingDenial {
    NonMonotonicTick,
    TrackCapacityExceeded,
    PresentationTruthUnavailable,
    InvalidSampleGeometry(super::UiPresentationGeometrySamplingDenial),
}

pub(crate) struct UiMountedMotionSampler {
    tracks: track_table::UiMotionTrackTable,
    last_tick: Option<u64>,
    reduced_motion: super::UiPresentationReducedMotionPosture,
    denial_count: u64,
    last_denial: Option<UiPresentationMotionSamplingDenial>,
}

#[must_use = "prepared motion samples must be committed after presentation or discarded"]
pub(crate) struct UiPreparedMotionSampling {
    successor: track_table::UiMotionTickTracks,
    tick: u64,
    receipt: super::UiPresentationMotionSamplingReceipt,
    prepared_at: worth_ui_host_contract::UiHostObservationPresentationBasis,
}

/// A prepared tick that may land: a host acknowledgement proved its samples
/// on the surface generation they were sampled for, or it sampled nothing.
/// [`UiMountedMotionSampler::commit_prepared`] accepts nothing else.
#[must_use = "presented motion samples must be committed"]
pub(crate) struct UiPresentedMotionSampling {
    successor: track_table::UiMotionTickTracks,
    tick: u64,
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
            self.successor.present(*sample);
        }
        Some(UiPresentedMotionSampling {
            successor: self.successor,
            tick: self.tick,
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
            tick: self.tick,
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
            tracks: track_table::UiMotionTrackTable::new(),
            last_tick: None,
            reduced_motion: super::UiPresentationReducedMotionPosture::NoPreference,
            denial_count: 0,
            last_denial: None,
        }
    }
}

impl UiMountedMotionSampler {
    pub(in crate::mounting) fn accept_published_entrance(
        &mut self,
        sample: super::UiPresentationMotionSampleReceipt,
    ) {
        self.tracks.accept_entrance(sample);
    }

    pub(in crate::mounting) fn retained_targets(
        &self,
    ) -> Vec<crate::runtime::motion::UiMotionTargetIdentity> {
        self.tracks.entries().map(|(target, _)| *target).collect()
    }

    pub(in crate::mounting) fn rebind_published_presentation(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) {
        self.tracks.rebind(surface, presentation);
    }

    /// Installs `receipt`'s track; a refused install leaves every track as it was.
    pub(crate) fn install(
        &mut self,
        receipt: crate::runtime::motion::UiMotionCommitReceipt,
    ) -> Result<super::UiPresentationMotionInstallationReceipt, UiPresentationMotionSamplingDenial>
    {
        let track = receipt.track();
        let target = track.target();
        let full = self.tracks.len() >= MAX_PRESENTATION_TRACKS;
        let evicted = if self.tracks.holds(&target) || !full {
            None
        } else {
            let settled = self
                .tracks
                .entries()
                .find_map(|(target, state)| (!state.is_running()).then_some(*target));
            match settled {
                Some(settled) => Some(settled),
                None => {
                    return self.deny(UiPresentationMotionSamplingDenial::TrackCapacityExceeded)
                }
            }
        };
        let interruption_tick = self.last_tick.unwrap_or(0);
        let predecessor = self.tracks.get(&target).copied();
        let current = predecessor.as_ref().and_then(|state| {
            state
                .is_running()
                .then(|| super::interruption::UiPresentationInterruptedSample {
                    tick: interruption_tick,
                    geometry: state.current_geometry(),
                    opacity_units: state.current_opacity_units,
                    outgoing: state.outgoing_curve(interruption_tick),
                })
        });
        let (built, terminal) =
            match super::interruption::resolve(track, current, self.reduced_motion) {
                super::interruption::UiPresentationMotionInstallation::Install {
                    geometry,
                    opacity_units,
                    start_velocity,
                    duration_ticks,
                    start_tick,
                } => (
                    super::track_sampling::UiPresentationTrackState::new(
                        track,
                        start_tick,
                        geometry,
                        opacity_units,
                        start_velocity,
                        duration_ticks,
                    ),
                    None,
                ),
                super::interruption::UiPresentationMotionInstallation::SnapToTarget => (
                    super::track_sampling::UiPresentationTrackState::terminal(
                        track,
                        interruption_tick,
                    ),
                    Some(super::UiPresentationMotionTerminalRequest::new(
                        track.identity(),
                        crate::runtime::motion::UiMotionTerminalCause::SnappedToTarget,
                    )),
                ),
            };
        // A successor that departs from the sample the host shows starts on
        // screen there, as a landing tick's departure does; left unpresented,
        // it hid that sample from the Scroll settle and from hit testing.
        let state = match built {
            Ok(mut state) => {
                if terminal.is_none() {
                    if let Some(predecessor) = &predecessor {
                        state.depart_on_screen(predecessor);
                    }
                }
                state
            }
            Err(denial) => {
                return self.deny(UiPresentationMotionSamplingDenial::InvalidSampleGeometry(
                    denial,
                ))
            }
        };
        if let Some(settled) = evicted {
            self.tracks.retire(settled);
        }
        let sample = state.current;
        self.tracks.install(target, state);
        Ok(super::UiPresentationMotionInstallationReceipt::new(
            sample, terminal,
        ))
    }

    pub(crate) fn reject_presentation_truth_unavailable(
        &mut self,
    ) -> Result<UiPreparedMotionSampling, UiPresentationMotionSamplingDenial> {
        self.deny(UiPresentationMotionSamplingDenial::PresentationTruthUnavailable)
    }

    pub(crate) fn retire_terminal_track(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.retire_track_where(track, |state| !state.is_running())
    }

    pub(crate) fn retire_rebound_track(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.retire_track_where(track, |_| true)
    }

    pub(crate) fn contains_track(
        &self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.tracks
            .states()
            .any(|state| state.track.identity() == track)
    }

    pub(crate) fn has_active_tracks(&self) -> bool {
        self.tracks.states().any(|track| track.is_running())
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
        self.tracks.retire_all()
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
            self.tracks
                .states()
                .filter(|track| track.is_running())
                .count(),
            self.tracks.len(),
            self.last_tick,
            self.tracks.states().map(|track| track.current).next(),
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
