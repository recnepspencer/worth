#[derive(Debug)]
pub enum WorthUiNativePresentationRecoveryDenial {
    CurrentPresentationUnavailable,
    LayoutReconstructionUnavailable,
    LayoutReconstructionObservationUnavailable,
    CurrentPublicationUnavailable,
    SurfaceRebindUnavailable,
    ViewportSettlementPublicationLease(Box<crate::mounting::UiMountedPublicationLeaseDenial>),
    ViewportSettlementEvidence(Box<crate::facade::host::UiHostMeasurementEvidenceDenial>),
    ViewportSettlementTransition(
        Box<crate::facade::entry::mounted_application_presentation::UiMountedHostMeasurementTransitionDenial>,
    ),
    ViewportSettlementOccurrenceGeometry(crate::mounting::UiMountedOccurrenceGeometryDenial),
    FramePreparationUnavailable,
    FramePresentationUnavailable,
    RasterReconstructionObservationUnavailable,
}

pub enum WorthUiNativePhysicalPresentationRecovery {
    Awaiting(crate::mounting::UiMountedIndeterminateFrame),
    Blocked {
        frame: crate::mounting::UiMountedIndeterminateFrame,
        denial: WorthUiNativePresentationRecoveryDenial,
    },
    Recovered(crate::mounting::UiMountedFrameOutcome),
}

impl super::WorthUiNativeApplicationShell {
    /// Recover an exclusively host-required native presentation reconstruction.
    ///
    /// Every other outcome is returned unchanged, so application runtimes do
    /// not need to reproduce physical-host denial classification.
    pub fn recover_reconstruction_required_presentation(
        &mut self,
        outcome: crate::mounting::UiMountedFrameOutcome,
        deadline_tick: u64,
        now_tick: u64,
    ) -> Result<crate::mounting::UiMountedFrameOutcome, WorthUiNativePresentationRecoveryDenial>
    {
        let requires_reconstruction = matches!(
            &outcome,
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected)
                if !rejected.rejections().is_empty()
                    && rejected.rejections().iter().all(|rejection| {
                        rejection.denial()
                            == worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired
                    })
        );
        if !requires_reconstruction {
            return Ok(outcome);
        }
        self.reconstruct_current_presentation_detailed(deadline_tick, now_tick)
    }

    /// Reconstruct after the exact physical correlation required by an
    /// indeterminate native presentation has reached the application driver.
    pub fn progress_indeterminate_presentation_recovery(
        &mut self,
        frame: crate::mounting::UiMountedIndeterminateFrame,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
        deadline_tick: u64,
        now_tick: u64,
    ) -> WorthUiNativePhysicalPresentationRecovery {
        self.progress_indeterminate_presentation_recovery_with_correlation(
            frame,
            progress.recovery_presentation(),
            deadline_tick,
            now_tick,
        )
    }

    pub(crate) fn progress_indeterminate_presentation_recovery_with_correlation(
        &mut self,
        frame: crate::mounting::UiMountedIndeterminateFrame,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
        deadline_tick: u64,
        now_tick: u64,
    ) -> WorthUiNativePhysicalPresentationRecovery {
        if frame.report().awaits_physical_recovery() {
            let Some(presentation) = presentation else {
                return WorthUiNativePhysicalPresentationRecovery::Awaiting(frame);
            };
            if presentation.attempt() != frame.report().attempt()
                || !frame
                    .report()
                    .physical_recovery_bindings()
                    .contains(&presentation.binding())
            {
                return WorthUiNativePhysicalPresentationRecovery::Awaiting(frame);
            }
        }
        match self.reconstruct_current_presentation_detailed(deadline_tick, now_tick) {
            Ok(outcome) => WorthUiNativePhysicalPresentationRecovery::Recovered(outcome),
            Err(denial) => WorthUiNativePhysicalPresentationRecovery::Blocked { frame, denial },
        }
    }

    pub(crate) fn reconstruct_current_presentation(
        &mut self,
        deadline_tick: u64,
        now_tick: u64,
    ) -> Result<crate::mounting::UiMountedFrameOutcome, ()> {
        self.reconstruct_current_presentation_detailed(deadline_tick, now_tick)
            .map_err(|_| ())
    }

    fn reconstruct_current_presentation_detailed(
        &mut self,
        deadline_tick: u64,
        now_tick: u64,
    ) -> Result<crate::mounting::UiMountedFrameOutcome, WorthUiNativePresentationRecoveryDenial>
    {
        let reconstructed_layouts =
            self.session
                .mounted
                .reconstruct_current_layouts()
                .map_err(|_| {
                    WorthUiNativePresentationRecoveryDenial::LayoutReconstructionUnavailable
                })?;
        if reconstructed_layouts > 0 {
            let observed = self.runtime_derived_state_reconstruction.ok_or(
                WorthUiNativePresentationRecoveryDenial::LayoutReconstructionObservationUnavailable,
            )?;
            self.runtime_derived_state_reconstruction = Some(
                worth_ui_host_native::UiNativeClientDerivedStateReconstructionObservation::reported(
                    observed.class(),
                    observed.loss_count(),
                    observed.reconstruction_count().saturating_add(1),
                    observed.derived_items_lost(),
                    observed
                        .derived_items_reconstructed()
                        .saturating_add(u64::try_from(reconstructed_layouts).unwrap_or(u64::MAX)),
                ),
            );
        }
        let affected = self
            .session
            .mounted
            .current_publication()
            .and_then(|publication| publication.bindings().first().copied())
            .ok_or(WorthUiNativePresentationRecoveryDenial::CurrentPublicationUnavailable)?;
        if affected == self.binding && self.pending_surface_reconciliation.is_none() {
            self.rebind_native_surface_scale(self.scale_factor_milli)
                .map_err(|()| WorthUiNativePresentationRecoveryDenial::SurfaceRebindUnavailable)?;
        }
        self.settle_pending_native_viewport_measurements()
            .map_err(|stop| match stop {
                crate::facade::entry::mounted_application_presentation::UiMountedHostMeasurementSettlementStop::PublicationLease(denial) => {
                    WorthUiNativePresentationRecoveryDenial::ViewportSettlementPublicationLease(
                        Box::new(denial),
                    )
                }
                crate::facade::entry::mounted_application_presentation::UiMountedHostMeasurementSettlementStop::Evidence(denial) => {
                    WorthUiNativePresentationRecoveryDenial::ViewportSettlementEvidence(denial)
                }
                crate::facade::entry::mounted_application_presentation::UiMountedHostMeasurementSettlementStop::Transition(denial) => {
                    WorthUiNativePresentationRecoveryDenial::ViewportSettlementTransition(denial)
                }
                crate::facade::entry::mounted_application_presentation::UiMountedHostMeasurementSettlementStop::OccurrenceGeometry(denial) => {
                    WorthUiNativePresentationRecoveryDenial::ViewportSettlementOccurrenceGeometry(denial)
                }
            })?;
        let request = self.session.mounted_frame_request();
        let replacement = self.pending_surface_reconciliation.unwrap_or_else(|| {
            crate::mounting::UiMountedSurfaceReconciliationBinding::new(affected, self.binding)
        });
        let replacements = [replacement];
        let frame = self
            .session
            .prepare_mounted_reconstruction_frame_with_application_presentation(
                request,
                &replacements,
                |_| {},
            )
            .map_err(|_| WorthUiNativePresentationRecoveryDenial::FramePreparationUnavailable)?;
        let outcome = self
            .session
            .present_prepared_mounted_frame_for_reconciliation(
                frame,
                &replacements,
                worth_ui_host_contract::UiPresentationDeadline::at_tick(deadline_tick),
                now_tick,
            )
            .map_err(|_| WorthUiNativePresentationRecoveryDenial::FramePresentationUnavailable)?;
        let reconstructed_rasters = self.session.mounted.take_reconstructed_raster_cache_items();
        if reconstructed_rasters > 0 {
            self.record_runtime_derived_state_reconstruction(reconstructed_rasters)
                .map_err(|()| {
                    WorthUiNativePresentationRecoveryDenial::RasterReconstructionObservationUnavailable
                })?;
        }
        self.settle_surface_reconciliation(&outcome);
        Ok(outcome)
    }

    pub(crate) fn require_current_layout_reconstruction(&mut self) -> Result<usize, ()> {
        if self.runtime_derived_state_reconstruction.is_some() {
            return Err(());
        }
        let lost = self
            .session
            .mounted
            .require_current_layout_reconstruction(self.binding)
            .map_err(|_| ())?;
        self.runtime_derived_state_reconstruction = Some(
            worth_ui_host_native::UiNativeClientDerivedStateReconstructionObservation::reported(
                worth_ui_host_native::UiNativeClientDerivedStateLossClass::MountedLayouts,
                1,
                0,
                u64::try_from(lost).unwrap_or(u64::MAX),
                0,
            ),
        );
        Ok(lost)
    }

    pub(crate) fn require_current_raster_cache_reconstruction(&mut self) -> Result<usize, ()> {
        if self.runtime_derived_state_reconstruction.is_some() {
            return Err(());
        }
        let lost = self
            .session
            .mounted
            .require_raster_cache_reconstruction(self.binding)
            .map_err(|_| ())?;
        self.runtime_derived_state_reconstruction = Some(
            worth_ui_host_native::UiNativeClientDerivedStateReconstructionObservation::reported(
                worth_ui_host_native::UiNativeClientDerivedStateLossClass::RasterCache,
                1,
                0,
                u64::try_from(lost).unwrap_or(u64::MAX),
                0,
            ),
        );
        Ok(lost)
    }

    fn record_runtime_derived_state_reconstruction(
        &mut self,
        reconstructed: usize,
    ) -> Result<(), ()> {
        let observed = self.runtime_derived_state_reconstruction.ok_or(())?;
        self.runtime_derived_state_reconstruction = Some(
            worth_ui_host_native::UiNativeClientDerivedStateReconstructionObservation::reported(
                observed.class(),
                observed.loss_count(),
                observed.reconstruction_count().saturating_add(1),
                observed.derived_items_lost(),
                observed
                    .derived_items_reconstructed()
                    .saturating_add(u64::try_from(reconstructed).unwrap_or(u64::MAX)),
            ),
        );
        Ok(())
    }

    pub(crate) const fn runtime_derived_state_reconstruction(
        &self,
    ) -> Option<worth_ui_host_native::UiNativeClientDerivedStateReconstructionObservation> {
        self.runtime_derived_state_reconstruction
    }
}

#[cfg(test)]
#[path = "presentation_recovery/tests.rs"]
mod tests;
