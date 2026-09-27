mod portal_dismissal_geometry;

use super::WorthUiMountedSessionState;
use crate::mounting::presentation::motion_sampling::UiPreparedMotionWork;

pub(crate) enum UiMountedMotionSampleSettlement {
    Committed(crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingReceipt),
    Deferred,
    Discarded,
    PresentationIndeterminate,
}

impl WorthUiMountedSessionState {
    pub(crate) fn prepare_motion_entrance(
        &self,
        entrance: Option<crate::runtime::motion::UiPreparedMotionEntrance>,
    ) -> Option<crate::runtime::motion::UiPreparedMotionEntrance> {
        entrance.filter(|entrance| !(entrance.snaps_for_reduced_motion()
            && self.motion_sampling.reduced_motion()
                == crate::mounting::presentation::motion_sampling::UiPresentationReducedMotionPosture::Reduce))
    }

    pub(crate) const fn last_motion_sampling_cost(
        &self,
    ) -> Option<crate::mounting::UiPresentationMotionSamplingCost> {
        self.last_motion_sampling_cost
    }

    pub(super) fn rebind_motion_sampling_after_publication(
        &mut self,
        publication: &crate::mounting::UiMountedFramePublicationReceipt,
    ) {
        publication.with_surface_presentations(|surfaces| {
            for surface in surfaces {
                debug_assert_eq!(surface.displayed_basis().frame(), publication.frame());
                self.motion_sampling.rebind_published_presentation(
                    surface.semantic_surface(),
                    surface.displayed_basis().basis(),
                );
            }
        });
    }

    #[cfg(test)]
    pub(crate) fn accepted_motion_for_command(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        command: worth_ui_host_contract::UiMountedPaintCommandIdentity,
    ) -> Result<
        Option<crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt>,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        self.retention
            .current_semantic_surface_for_presentation(presentation)?;
        self.presentation
            .accepted_motion_for_command(presentation, command)
    }

    pub(crate) fn install_motion_commit(
        &mut self,
        receipt: crate::runtime::motion::UiMotionCommitReceipt,
    ) -> Result<
        crate::mounting::presentation::motion_sampling::UiPresentationMotionInstallationReceipt,
        crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingDenial,
    > {
        let mut installation = self.motion_sampling.install(receipt)?;
        let sample = installation.sample();
        // An entrance becomes displayed evidence only against the witness
        // that displayed its surface; without one it stays published.
        let displayed = self.current_presentation_for_surface(sample.target().semantic_surface());
        if let Some(displayed) = displayed {
            if self
                .presentation
                .accept_published_entrance(sample, displayed)?
            {
                self.motion_sampling.accept_published_entrance(sample);
            }
            // A track that departs on screen is what interaction reads from
            // now on, so the hit index projects it now rather than at the
            // next tick. Settled poses, not samples, move Scroll content rows.
            // The index work is one target's projection and is not a tick's
            // cost, so, as at a publication's refresh, it is not recorded.
            let target = sample.target();
            if target.scope() != crate::runtime::motion::UiMotionTargetScope::ScrollContents {
                let frame = displayed.basis().frame();
                let predecessor = self.retention.hit_evidence(frame);
                let _ = self
                    .retention
                    .refresh_presented_hit_motion(&self.motion_sampling, &[target]);
                installation.record_hit_transition(
                    self.retention.committed_hit_transition(predecessor, frame),
                );
            }
        }
        Ok(installation)
    }

    pub(crate) fn retire_terminal_motion_sample(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.motion_sampling.retire_terminal_track(track)
    }

    pub(crate) fn retire_rebound_motion_sample(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.motion_sampling.retire_rebound_track(track)
    }

    pub(crate) fn contains_motion_track(
        &self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> bool {
        self.motion_sampling.contains_track(track)
    }

    pub(crate) fn prepare_motion_tick(
        &mut self,
        tick: u64,
        displayed: crate::mounting::presentation::UiDisplayedSurfaceBasis,
    ) -> Result<
        crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling,
        crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingDenial,
    > {
        let presentation = displayed.basis();
        if self
            .presentation
            .binding_requires_reconstruction(presentation.binding())
        {
            return self.motion_sampling.reject_presentation_truth_unavailable();
        }
        self.motion_sampling.prepare_tick(tick, presentation)
    }

    pub(crate) fn present_prepared_motion_tick(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        prepared: crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling,
        displayed: crate::mounting::presentation::UiDisplayedSurfaceBasis,
    ) -> UiMountedMotionSampleSettlement {
        let prepared = match prepared.into_work() {
            UiPreparedMotionWork::Unsampled(unsampled) => {
                let receipt = self.motion_sampling.commit_prepared(unsampled);
                self.last_motion_sampling_cost = Some(receipt.cost());
                return UiMountedMotionSampleSettlement::Committed(receipt);
            }
            UiPreparedMotionWork::NeedsPresentation(prepared) => prepared,
        };
        let capability_report = host.capability_report().clone();
        let outcome = self.presentation.present_motion_sample(
            prepared,
            displayed.basis(),
            host.effect_port(),
            super::publication::mounted_host_authority(host, &capability_report),
        );
        self.settle_motion_sample_presentation(outcome)
    }

    pub(crate) fn complete_motion_sample_presentation(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
    ) -> Option<UiMountedMotionSampleSettlement> {
        let outcome = self
            .presentation
            .complete_motion_sample(host.effect_port())?;
        Some(self.settle_motion_sample_presentation(outcome))
    }

    fn settle_motion_sample_presentation(
        &mut self,
        outcome: crate::mounting::UiMotionSamplePresentationOutcome,
    ) -> UiMountedMotionSampleSettlement {
        use crate::mounting::UiMotionSamplePresentationOutcome as Outcome;

        match outcome {
            Outcome::Presented { prepared, witness } => {
                let displayed = witness.displayed_basis();
                let presentation = displayed.basis();
                let Some(prepared) = prepared.into_presented(&witness) else {
                    self.presentation
                        .mark_motion_sample_indeterminate(presentation.binding());
                    return UiMountedMotionSampleSettlement::PresentationIndeterminate;
                };
                let hit_predecessor = self.retention.hit_evidence(presentation.frame());
                let Ok(presented_surface) =
                    self.retention.update_current_presentation_epoch(&witness)
                else {
                    self.presentation
                        .mark_motion_sample_indeterminate(presentation.binding());
                    return UiMountedMotionSampleSettlement::PresentationIndeterminate;
                };
                debug_assert_eq!(
                    presented_surface,
                    witness.requirement().semantic_surface(),
                    "retention records the surface the witness proved"
                );
                let mut receipt = self.motion_sampling.commit_prepared(prepared);
                receipt.record_presented_surface(
                    crate::mounting::presentation::motion_sampling::UiPresentationMotionPresentedSurface::new(
                        presented_surface,
                        displayed,
                    ),
                );
                let targets = receipt
                    .samples()
                    .iter()
                    .map(|sample| sample.target())
                    .filter(|target| {
                        target.scope()
                            != crate::runtime::motion::UiMotionTargetScope::ScrollContents
                    })
                    .collect::<Vec<_>>();
                let hit_work = self
                    .retention
                    .refresh_presented_hit_motion(&self.motion_sampling, &targets);
                receipt.record_hit_index_work(hit_work);
                self.last_motion_sampling_cost = Some(receipt.cost());
                receipt.record_hit_transition(
                    self.retention
                        .committed_hit_transition(hit_predecessor, presentation.frame()),
                );
                let changed = self
                    .presentation
                    .motion_appearance_instances(presentation.binding(), &targets);
                self.identity.mark_motion_appearance_changed(&changed);
                UiMountedMotionSampleSettlement::Committed(receipt)
            }
            Outcome::InFlight => UiMountedMotionSampleSettlement::Deferred,
            Outcome::RejectedBeforeEffects => UiMountedMotionSampleSettlement::Discarded,
            Outcome::Superseded => UiMountedMotionSampleSettlement::Discarded,
            Outcome::PresentationIndeterminate => {
                UiMountedMotionSampleSettlement::PresentationIndeterminate
            }
        }
    }

    pub(crate) fn has_active_motion_samples(&self) -> bool {
        !self.presentation.motion_presentation_truth_unavailable()
            && self.motion_sampling.has_active_tracks()
    }

    pub(crate) fn motion_sample_presentation_pending(&self) -> bool {
        self.presentation.motion_sample_presentation_pending()
    }

    pub(crate) fn pending_motion_sample_matches(
        &self,
        presentation: worth_ui_host_native::UiNativePhysicalPresentationCorrelation,
    ) -> bool {
        self.presentation
            .pending_motion_sample_matches(presentation)
    }

    pub(crate) fn set_reduced_motion_posture(
        &mut self,
        posture: crate::mounting::presentation::motion_sampling::UiPresentationReducedMotionPosture,
    ) {
        self.motion_sampling.set_reduced_motion(posture);
    }

    pub(crate) fn shutdown_motion_sampling(&mut self) -> usize {
        self.motion_sampling.shutdown()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn motion_sampling_observation_for_certification(
        &self,
    ) -> (
        usize,
        usize,
        Option<u64>,
        Option<crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt>,
        u64,
        Option<crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingDenial>,
    ) {
        self.motion_sampling.certification_observation()
    }
}
