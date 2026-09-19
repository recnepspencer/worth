use super::WorthUiMountedSessionState;

impl WorthUiMountedSessionState {
    #[cfg(feature = "certification-support")]
    pub(crate) fn require_raster_cache_reconstruction(
        &mut self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Result<usize, crate::mounting::UiMountedProjectionDenial> {
        let binding_view = self
            .identity
            .surface_binding(binding)
            .ok_or(crate::mounting::UiMountedProjectionDenial::MissingSurfaceBinding)?;
        let lost = self.presentation.require_raster_cache_reconstruction(
            crate::mounting::binding_requirement(binding_view),
        );
        if lost == 0 {
            return Err(
                crate::mounting::UiMountedProjectionDenial::MissingSemanticTextReconstructionSource,
            );
        }
        Ok(lost)
    }

    pub(crate) fn take_reconstructed_raster_cache_items(&mut self) -> usize {
        self.presentation.take_reconstructed_raster_cache_items()
    }
}
