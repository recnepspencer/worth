use super::WorthUiMountedSessionState;

impl WorthUiMountedSessionState {
    pub(crate) fn prepare_retained_geometry_succession(
        &self,
        previous: &crate::runtime::session::WorthUiApplicationSessionState,
        candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        plan: &crate::runtime::WorthUiActiveExecutionPlan,
        bindings: &crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    ) -> Result<
        crate::mounting::UiMountedOccurrenceGeometryState,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        let mut geometry = self.occurrence_geometry.clone();
        geometry.prepare_generation_succession(
            &self.identity,
            previous,
            candidate,
            plan,
            bindings,
        )?;
        Ok(geometry)
    }

    pub(crate) fn commit_retained_geometry_succession(
        &mut self,
        geometry: crate::mounting::UiMountedOccurrenceGeometryState,
    ) {
        self.occurrence_geometry = geometry;
    }

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

    /// The viewport a Portal on `surface` is fitted to, at open and in every
    /// frame after: the surface's current layout viewport, which Backdrops
    /// cover too.
    pub(crate) fn current_portal_viewport(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<crate::mounting::presentation::UiPublishedRect> {
        crate::mounting::portal_placement_succession::portal_viewport(
            &self.identity,
            &self.occurrence_geometry,
            surface,
        )
    }
}
