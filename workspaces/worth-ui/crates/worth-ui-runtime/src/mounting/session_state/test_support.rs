impl super::WorthUiMountedSessionState {
    pub(crate) fn current_mosaic_clips_for_test(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<Box<[worth_ui_host_contract::UiMountedCanonicalBox]>> {
        let instance = self.identity.projection_instance(instance)?;
        Some(self.occurrence_geometry.mosaic_clips(&instance).into())
    }

    pub(crate) fn current_surface_paint_posture_for_test(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<crate::mounting::UiMountedSurfacePaintPosture> {
        let instance = self.identity.projection_instance(instance)?;
        Some(self.occurrence_geometry.surface_paint_posture(&instance))
    }
}
