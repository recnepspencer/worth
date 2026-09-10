use super::WorthUiMountedSessionState;

impl WorthUiMountedSessionState {
    pub(crate) fn has_current_occurrence_geometry(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> bool {
        self.identity
            .projection_instance(instance)
            .is_some_and(|view| {
                self.occurrence_geometry
                    .projection(&view)
                    .is_ok_and(|projection| projection.is_some())
            })
    }

    pub(crate) fn current_surface_viewport(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<(
        crate::mounting::UiMountedLayoutRevision,
        worth_ui_host_contract::UiMountedCanonicalBox,
    )> {
        let binding = self
            .identity
            .projection_surface(surface)?
            .0
            .binding_generation();
        let (geometry_binding, revision, viewport) =
            self.occurrence_geometry.surface_viewport(surface)?;
        (geometry_binding == binding).then_some((revision, viewport))
    }
}
