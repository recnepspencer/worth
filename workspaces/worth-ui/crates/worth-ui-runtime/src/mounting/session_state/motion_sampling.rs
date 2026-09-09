use super::WorthUiMountedSessionState;

pub(crate) enum UiMountedMotionSampleSettlement {
    Committed(crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingReceipt),
    Deferred,
    Discarded,
    PresentationIndeterminate,
}

impl WorthUiMountedSessionState {
    pub(super) fn rebind_motion_sampling_after_publication(
        &mut self,
        publication: &crate::mounting::UiMountedFramePublicationReceipt,
    ) {
        publication.with_surface_presentations(|surfaces| {
            for surface in surfaces {
                self.motion_sampling.rebind_published_presentation(
                    surface.semantic_surface(),
                    worth_ui_host_contract::UiHostObservationPresentationBasis::new(
                        surface.host_surface(),
                        publication.frame(),
                        surface.binding(),
                        surface.epoch(),
                    ),
                );
            }
        });
    }

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
        self.motion_sampling.install(receipt)
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
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling,
        crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingDenial,
    > {
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
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> UiMountedMotionSampleSettlement {
        if prepared.receipt().samples().is_empty() {
            return UiMountedMotionSampleSettlement::Committed(
                self.motion_sampling.commit_prepared(prepared),
            );
        }
        let capability_report = host.capability_report().clone();
        let outcome = self.presentation.present_motion_sample(
            prepared,
            presentation,
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
            Outcome::Presented {
                prepared,
                presentation,
            } => {
                let Ok(prepared) = prepared.with_presented_basis(presentation) else {
                    self.presentation
                        .mark_motion_sample_indeterminate(presentation.binding());
                    return UiMountedMotionSampleSettlement::PresentationIndeterminate;
                };
                let hit_predecessor = self.retention.current_hit_evidence();
                if self
                    .retention
                    .update_current_presentation_epoch(presentation)
                    .is_err()
                {
                    self.presentation
                        .mark_motion_sample_indeterminate(presentation.binding());
                    return UiMountedMotionSampleSettlement::PresentationIndeterminate;
                }
                let mut receipt = self.motion_sampling.commit_prepared(prepared);
                let targets = receipt
                    .samples()
                    .iter()
                    .map(|sample| sample.target())
                    .collect::<Vec<_>>();
                let hit_work = self
                    .retention
                    .refresh_presented_hit_motion(&self.motion_sampling, &targets);
                receipt.record_hit_index_work(hit_work);
                receipt.record_hit_transition(
                    self.retention.committed_hit_transition(hit_predecessor),
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

    pub(crate) fn committed_motion_geometry_for_instance(
        &self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        if self
            .presentation
            .binding_requires_reconstruction(presentation.binding())
        {
            return Err(crate::mounting::UiPresentedFrameBasisDenial::PresentationTruthUnavailable);
        }
        let coordinate_space = self
            .retention
            .interaction_hit_test_basis(presentation)?
            .rows()
            .iter()
            .find(|row| row.mounted_instance() == mounted_instance)
            .map(|row| row.bounds().coordinate_space())
            .ok_or(crate::mounting::UiPresentedFrameBasisDenial::Unknown)?;
        let Some(geometry) = self
            .motion_sampling
            .current_sample_for(mounted_instance, presentation)
            .and_then(|sample| sample.geometry())
        else {
            return Ok(None);
        };
        let components = geometry.components();
        worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
            worth_ui_host_contract::UiMountedCanonicalBoxInput {
                x: components[0],
                y: components[1],
                width: components[2],
                height: components[3],
                coordinate_space,
            },
        )
        .map(Some)
        .map_err(|_| crate::mounting::UiPresentedFrameBasisDenial::Unknown)
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
