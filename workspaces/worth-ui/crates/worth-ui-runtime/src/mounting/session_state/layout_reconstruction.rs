use super::WorthUiMountedSessionState;

impl WorthUiMountedSessionState {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn require_current_layout_reconstruction(
        &mut self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Result<usize, crate::mounting::UiMountedProjectionDenial> {
        let binding_view = self
            .identity
            .surface_binding(binding)
            .ok_or(crate::mounting::UiMountedProjectionDenial::MissingSurfaceBinding)?;
        let lost = self.identity.require_current_layout_reconstruction()?;
        if lost == 0 {
            return Err(
                crate::mounting::UiMountedProjectionDenial::MissingSemanticTextReconstructionSource,
            );
        }
        self.presentation
            .host_truth_mut()
            .block_presentation(crate::mounting::binding_requirement(binding_view));
        Ok(lost)
    }

    pub(crate) fn reconstruct_current_layouts(
        &mut self,
    ) -> Result<usize, crate::mounting::UiMountedProjectionDenial> {
        self.identity.reconstruct_current_layouts()
    }
}
