use super::UiMountedPresentationCoordinator;

impl UiMountedPresentationCoordinator {
    #[cfg(feature = "certification-support")]
    pub(crate) fn require_raster_cache_reconstruction(
        &mut self,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
    ) -> usize {
        let lost = self.text.require_raster_cache_reconstruction();
        if lost > 0 {
            self.host_truth.block_presentation(requirement);
        }
        lost
    }

    pub(crate) fn take_reconstructed_raster_cache_items(&mut self) -> usize {
        self.text.take_reconstructed_raster_cache_items()
    }

    pub(crate) const fn peak_raster_cache_entries(&self) -> usize {
        self.text.peak_raster_cache_entries()
    }
}
