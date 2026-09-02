impl super::WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn install_portal_exit_retention(
        &mut self,
        retention: Option<(
            crate::runtime::portal::UiPortalExitRetentionReceipt,
            crate::runtime::motion::UiMotionExitRetentionReceipt,
        )>,
    ) {
        let Some((portal, motion)) = retention else {
            return;
        };
        self.portal_exit_retention
            .install(portal, motion)
            .expect("settled Portal and Motion exit retention has exact owner affinity");
    }

    pub(in crate::facade::entry) fn install_committed_motion(
        &mut self,
        receipt: Option<crate::runtime::motion::UiMotionCommitReceipt>,
    ) {
        let Some(receipt) = receipt else {
            return;
        };
        if let Some(displaced) = receipt.displaced_exit_retention() {
            // `prepare_portal_motion_request` refuses a successor transition whose
            // target still owns physically pending exit work, so any retention
            // reaching displacement here is either unsettled or awaiting retry and
            // is therefore removable.
            self.portal_exit_retention
                .remove_displaced(displaced)
                .expect("the physical-settlement gate admits only removable displaced exits");
        }
        let installation = self
            .mounted
            .install_motion_commit(receipt)
            .expect("semantic Motion capacity bounds mounted presentation sampling");
        if let Some(terminal) = installation.terminal() {
            self.settle_motion_terminal_request(terminal);
        }
    }

    pub(in crate::facade::entry) fn release_rebound_portal_retention(
        &mut self,
        portal: crate::runtime::portal::UiPortalIdentity,
    ) {
        let Some(motion) = self
            .portal_exit_retention
            .remove_rebound_portal(portal)
            .expect("rebound Portal exit retention has no in-flight physical work")
        else {
            return;
        };
        assert!(self
            .motion
            .as_mut()
            .expect("rebound Portal exit retention retains Motion installation")
            .release_exit_retention(motion));
        self.retire_rebound_motion_sample(motion.track());
    }

    pub(in crate::facade::entry) fn release_rebound_motion_retention(
        &mut self,
        terminal: crate::runtime::motion::UiMotionTerminalReceipt,
    ) {
        let Some(retention) = terminal.exit_retention() else {
            return;
        };
        self.portal_exit_retention
            .remove_displaced(retention)
            .expect("rebound Motion exit retention has exact Portal coordination");
        assert!(self
            .motion
            .as_mut()
            .expect("rebound exit retention retains Motion installation")
            .release_exit_retention(retention));
    }

    pub(in crate::facade::entry) fn retire_rebound_motion_sample(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) {
        if !self.mounted.retire_rebound_motion_sample(track) {
            assert!(
                !self.mounted.contains_motion_track(track),
                "rebound Motion terminal left an active or queued mounted sample"
            );
        }
    }

    pub(in crate::facade::entry) fn prepare_motion_tick(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling,
        crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingDenial,
    > {
        self.mounted.prepare_motion_tick(tick, presentation)
    }

    pub(in crate::facade::entry) fn present_prepared_motion_tick(
        &mut self,
        prepared: crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) {
        let settlement =
            self.mounted
                .present_prepared_motion_tick(&self.host_session, prepared, presentation);
        self.settle_motion_sample_presentation(settlement);
    }

    pub(crate) fn complete_motion_sample_presentation(&mut self) {
        let Some(settlement) = self
            .mounted
            .complete_motion_sample_presentation(&self.host_session)
        else {
            return;
        };
        self.settle_motion_sample_presentation(settlement);
    }

    fn settle_motion_sample_presentation(
        &mut self,
        settlement: crate::mounting::UiMountedMotionSampleSettlement,
    ) {
        if let crate::mounting::UiMountedMotionSampleSettlement::Committed(sampling) = settlement {
            for terminal in sampling.terminals().iter().copied() {
                self.settle_motion_terminal_request(terminal);
            }
        }
    }

    fn settle_motion_terminal_request(
        &mut self,
        request: crate::mounting::presentation::motion_sampling::UiPresentationMotionTerminalRequest,
    ) {
        let Some(motion) = self.motion.as_mut() else {
            return;
        };
        let terminal = motion
            .terminalize(request.track(), request.cause())
            .expect("presentation terminal request names the current committed Motion track");
        self.portal_exit_retention
            .observe_terminal(terminal)
            .expect("terminal Motion evidence matches its installed portal exit retention");
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn inspect_motion_presentation_for_certification(
        &self,
    ) -> crate::certification_support::UiMotionPresentationCertificationSnapshot {
        let (active, retained, last_tick, sample, sampling_denials, last_denial) =
            self.mounted.motion_sampling_observation_for_certification();
        let presentation = sample.and_then(|sample| {
            sample
                .geometry()
                .map(|geometry| geometry.presentation_basis())
        });
        crate::certification_support::UiMotionPresentationCertificationSnapshot::new(
            active,
            retained,
            last_tick,
            self.motion.as_ref().map_or(0, |motion| motion.publication_count()),
            sample.and_then(|sample| sample.geometry().map(|geometry| geometry.components())),
            sample.map(|sample| sample.opacity()),
            sample.map(|sample| sample.hit_test_visible()),
            presentation,
            self.mounted.has_active_motion_samples(),
            presentation.is_none_or(|presentation| {
                self.mounted.interaction_hit_test_basis(presentation).is_ok()
            }),
            sampling_denials,
            matches!(
                last_denial,
                Some(
                    crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingDenial::NonMonotonicTick
                )
            ),
            matches!(
                last_denial,
                Some(
                    crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingDenial::PresentationTruthUnavailable
                )
            ),
        )
    }
}
