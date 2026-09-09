impl super::UiMountedIdentityState {
    pub(in crate::mounting) fn layout_basis(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        generation: crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    ) -> Result<
        crate::mounting::UiMountedLayoutBasis,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        let binding = self
            .projection_surface(surface)
            .ok_or(crate::mounting::UiMountedOccurrenceGeometryDenial::MissingSurfaceBinding)?
            .0
            .binding_generation();
        Ok(crate::mounting::UiMountedLayoutBasis {
            generation,
            surface,
            binding,
            world: self.world_identity,
            semantic_revision: self.semantic_revision,
        })
    }

    pub(in crate::mounting) fn validates_layout_basis(
        &self,
        basis: &crate::mounting::UiMountedLayoutBasis,
    ) -> bool {
        self.world_identity == basis.world
            && self.semantic_revision == basis.semantic_revision
            && self
                .projection_surface(basis.surface)
                .is_some_and(|(surface, _)| surface.binding_generation() == basis.binding)
    }
}
