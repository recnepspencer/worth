use super::{UiPortalOverlayBindingLifecycle, UiPortalOverlayBindingLifecycleDenial as Denial};

impl UiPortalOverlayBindingLifecycle {
    /// Re-admit surviving authored bindings against the actual successor
    /// declarations. This builds the owner committed after host acceptance.
    pub(crate) fn prepare_application_replacement(
        &self,
        previous: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        authority: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        removed: &[super::UiPortalIdentity],
    ) -> Result<Self, Denial> {
        let generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.generation.session_identity(),
            authority.generation_identity(),
        );
        if previous.generation_identity() != self.generation.prepared_generation() {
            return Err(Denial::ForeignGeneration);
        }
        let previous_declarations = previous
            .authored_overlay_material()
            .overlay_declaration_bindings();
        let material = authority.authored_overlay_material();
        let declarations = material.overlay_declaration_bindings();
        let mut successor = Self::new(generation.clone());
        for (declaration, surface, owner) in self.bound_owners() {
            if owner.generation() != self.generation.prepared_generation() {
                return Err(Denial::ForeignGeneration);
            }
            let name = previous_declarations
                .surface_name(declaration)
                .ok_or(Denial::ForeignSurfaceDeclaration)?;
            let Some(declaration) = declarations.surface_named(name) else {
                if owner
                    .bindings()
                    .any(|(portal, _)| !removed.contains(&portal))
                {
                    return Err(Denial::ForeignSurfaceDeclaration);
                }
                continue;
            };
            successor.install_declared_surface(&generation, declaration, surface)?;
            let next = successor.owners.get_mut(&surface).expect("installed owner");
            for (portal, binding) in owner.bindings() {
                if removed.contains(&portal) {
                    continue;
                }
                let name = previous_declarations
                    .portal_name(binding)
                    .ok_or(Denial::UnknownPortalDeclaration)?;
                let binding = declarations
                    .portal_named(name)
                    .ok_or(Denial::UnknownPortalDeclaration)?;
                let authored = material
                    .portal_anchor_binding(binding)
                    .ok_or(Denial::UnknownPortalDeclaration)?;
                if authored.surface_declaration_id() != Some(declaration) {
                    return Err(Denial::PortalSurfaceMismatch);
                }
                next.bind(binding, portal).map_err(Denial::Owner)?;
            }
        }
        Ok(successor)
    }
}
