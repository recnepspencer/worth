use super::{UiAssembledMountedFrame, UiMountedFramePreparationDenial};

pub struct UiPreparedMountedFrame {
    frame: UiAssembledMountedFrame,
}

impl UiAssembledMountedFrame {
    pub(crate) fn prepared_theme_binding(
        &self,
    ) -> Option<&crate::runtime::appearance::UiActiveThemeBinding> {
        self.prepared_theme_binding.as_ref()
    }

    pub(crate) fn resolve_appearance(
        mut self,
        projection: crate::facade::entry::appearance_projection::UiAppearanceFrameProjection<'_>,
    ) -> Result<UiPreparedMountedFrame, UiMountedFramePreparationDenial> {
        let basis = crate::graph::UiGraphFactIndexBasis::from_generation(
            projection.graph.snapshot(),
            projection.capabilities,
        );
        if !self
            .candidate
            .owner
            .appearance()
            .matches_consumer_basis(basis)
        {
            return Err(UiMountedFramePreparationDenial::Projection(
                crate::mounting::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch,
            ));
        }
        if let crate::facade::entry::appearance_projection::UiAppearanceProjectionPhase::ThemeSwitch(change) = projection.phase {
            projection.presentation.appearance_theme_state()
                .ok_or(UiMountedFramePreparationDenial::AppearanceThemeSwitch(
                    crate::runtime::appearance::UiThemeSwitchDenial::MissingActiveBinding))?
                .validate_prepared_switch(change.prepared())
                .map_err(UiMountedFramePreparationDenial::AppearanceThemeSwitch)?;
            self.prepared_theme_binding = Some(change.prepared().successor().clone());
        }
        projection.finish(&mut self)?;
        Ok(UiPreparedMountedFrame { frame: self })
    }
}

impl std::ops::Deref for UiPreparedMountedFrame {
    type Target = UiAssembledMountedFrame;

    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl UiPreparedMountedFrame {
    pub(in crate::mounting) fn reconcile_current(
        identity: &crate::mounting::UiMountedIdentityState,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        protocol: worth_ui_host_contract::UiHostProtocolAgreement,
        capability_report: &worth_ui_host_contract::WorthUiHostCapabilityReport,
    ) -> Result<Self, crate::mounting::UiMountedIdentityDenial> {
        let frame = identity.assemble_current_reconciliation_frame(
            replacements,
            protocol,
            capability_report,
        )?;
        Ok(Self { frame })
    }

    pub(crate) fn record_accepted_motion_commands_visited(&mut self, count: usize) {
        self.frame.record_accepted_motion_commands_visited(count)
    }

    pub(crate) fn lower_appearance_with_motion_and_overlays(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
        overlays: &[crate::mounting::UiMountedAppearanceSurfaceOverlayInput],
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        self.frame.lower_appearance_with_motion_and_overlays(
            presentation,
            profile,
            motion,
            overlays,
        )
    }

    pub(crate) fn deny_appearance_output(
        &mut self,
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        self.frame.deny_appearance_output()
    }

    pub(crate) fn into_publication_parts(
        self,
    ) -> (
        crate::mounting::UiProjectedMountedFrameCandidate,
        worth_ui_host_contract::UiMountedFrameManifest,
        worth_ui_host_contract::UiMountedFrameCanonicalCore,
        crate::mounting::UiMountedFrameReuseContract,
    ) {
        self.frame.into_publication_parts()
    }

    pub(in crate::mounting) fn reserve_appearance_retry_basis(&mut self) {
        self.frame.reserve_appearance_retry_basis()
    }

    pub(in crate::mounting) fn restore_rejected_appearance(&mut self) {
        self.frame.restore_rejected_appearance()
    }

    pub(crate) fn prepare_appearance_reconstruction(
        &mut self,
    ) -> Result<(), crate::mounting::projection::UiMountedProjectionDenial> {
        self.frame.prepare_appearance_reconstruction()
    }
}

#[cfg(test)]
mod test_access;
