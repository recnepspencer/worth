use super::WorthUiMountedSessionState;

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
        let installation = self.motion_sampling.install(receipt)?;
        if let Some(sample) = installation.sample() {
            if self.presentation.accept_published_entrance(sample)? {
                self.motion_sampling.accept_published_entrance(sample);
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
            let receipt = self.motion_sampling.commit_prepared(prepared);
            self.last_motion_sampling_cost = Some(receipt.cost());
            return UiMountedMotionSampleSettlement::Committed(receipt);
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
                let hit_predecessor = self.retention.hit_evidence(presentation.frame());
                let Ok(presented_surface) = self
                    .retention
                    .update_current_presentation_epoch(presentation)
                else {
                    self.presentation
                        .mark_motion_sample_indeterminate(presentation.binding());
                    return UiMountedMotionSampleSettlement::PresentationIndeterminate;
                };
                let mut receipt = self.motion_sampling.commit_prepared(prepared);
                receipt.record_presented_surface(
                    crate::mounting::presentation::motion_sampling::UiPresentationMotionPresentedSurface::new(
                        presented_surface,
                        presentation,
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

    /// The presentation at which a press on `admitted` may be classified
    /// against one Portal. A later publication on the same physical surface
    /// can supersede `admitted` while the host still holds the press, and the
    /// press still names what the reader saw. It is read at the current
    /// presentation only when that Portal looked the same in both: the
    /// admitted frame showed the same overlay geometry, and no Motion sample
    /// of the Portal reached the screen after the press. Otherwise what the
    /// reader saw is unknown and the press stays stale.
    pub(crate) fn portal_dismissal_presentation(
        &self,
        admitted: worth_ui_host_contract::UiHostObservationPresentationBasis,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> Result<
        worth_ui_host_contract::UiHostObservationPresentationBasis,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        use crate::mounting::UiPresentedFrameBasisDenial as Denial;
        if self
            .current_semantic_surface_for_presentation(admitted)
            .is_ok()
        {
            return Ok(admitted);
        }
        if self
            .presentation
            .binding_requires_reconstruction(admitted.binding())
        {
            return Err(Denial::PresentationTruthUnavailable);
        }
        let current = self
            .current_surface_for_binding(admitted.binding())
            .and_then(|surface| self.current_presentation_for_surface(surface))
            .filter(|current| {
                current.host_surface() == admitted.host_surface()
                    && current.epoch() > admitted.epoch()
            })
            .ok_or(Denial::Expired)?;
        let seen = self.retention.presented_portal_overlay(admitted, target)?;
        let shown = self.retention.presented_portal_overlay(current, target)?;
        let unchanged = match (seen, shown) {
            (Some(seen), Some(shown)) => same_portal_geometry(seen, shown),
            _ => false,
        };
        if !unchanged
            || self
                .motion_sampling
                .target_presented_after(target, admitted)
        {
            return Err(Denial::Expired);
        }
        Ok(current)
    }

    pub(crate) fn committed_motion_geometry_for_target(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
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
        self.current_semantic_surface_for_presentation(presentation)?;
        let coordinate_space = self
            .retention
            .interaction_hit_test_basis(presentation)?
            .rows()
            .iter()
            .find(|row| row.mounted_instance() == target.mounted_instance())
            .map(|row| row.bounds().coordinate_space())
            .ok_or(crate::mounting::UiPresentedFrameBasisDenial::Unknown)?;
        let sample = self
            .motion_sampling
            .current_sample_for_target(target, presentation);
        let Some(geometry) = sample.and_then(|sample| sample.geometry()) else {
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

/// Everything a Portal dismissal reads from a presented overlay. Frame and
/// receipt identities differ across publications that leave it in place.
fn same_portal_geometry(
    seen: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
    shown: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
) -> bool {
    seen.surface() == shown.surface()
        && seen.owner() == shown.owner()
        && seen.portal_identity() == shown.portal_identity()
        && seen.anchor_bounds() == shown.anchor_bounds()
        && seen.bounds() == shown.bounds()
        && seen.clip_bounds() == shown.clip_bounds()
        && seen.lifecycle() == shown.lifecycle()
        && seen.shielding() == shown.shielding()
}
