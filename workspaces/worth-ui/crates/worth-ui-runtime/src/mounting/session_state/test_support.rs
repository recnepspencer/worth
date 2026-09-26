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

    /// Stage no pose, only marking `unmoved` as a settle does, so a test can
    /// name an occurrence the mounted identity no longer knows.
    pub(crate) fn stage_unmoved_occurrences_for_test(
        &mut self,
        unmoved: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) -> Result<(), crate::mounting::UiMountedOccurrenceGeometryDenial> {
        self.stage_scroll_poses(std::iter::empty(), unmoved.iter().copied())
    }
}
