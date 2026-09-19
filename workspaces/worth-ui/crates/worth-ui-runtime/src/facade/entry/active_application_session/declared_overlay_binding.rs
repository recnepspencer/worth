use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn declared_region_layout_inputs(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Box<[super::super::UiNativeMountedRegionLayoutInput]> {
        let Some(surface_declaration) = self
            .authored_overlay_bindings
            .bound_owners()
            .find_map(|(declaration, runtime, _)| (runtime == surface).then_some(declaration))
        else {
            return Box::default();
        };
        self.mounted
            .view()
            .mounted_instances()
            .iter()
            .filter(|mounted| mounted.basis().semantic_surface_identity() == surface)
            .flat_map(|mounted| {
                let owner = mounted.identity();
                self.application
                    .mounted_region_declarations(surface_declaration, mounted.graph_node_identity())
                    .0
                    .into_iter()
                    .map(move |binding| {
                        super::super::UiNativeMountedRegionLayoutInput::from_mounted_region(
                            owner,
                            binding.region_kind(),
                            binding.declaration(),
                            binding.executed_region(),
                        )
                    })
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn create_declared_semantic_surface_named(
        &mut self,
        authored_name: &str,
    ) -> Result<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial,
    > {
        let declaration = self
            .application
            .authored_overlay_material()
            .overlay_declaration_bindings()
            .surface_named(authored_name)
            .ok_or(
                crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial::DeclaredSurfaceUnbound,
            )?;
        self.create_declared_semantic_surface(declaration)
    }

    pub(crate) fn create_declared_semantic_surface(
        &mut self,
        declaration: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    ) -> Result<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial,
    > {
        let generation = self.active_generation_identity();
        self.authored_overlay_bindings.validate_declared_surface(
            &generation,
            self.application.authored_overlay_material(),
            declaration,
        )?;
        let surface =
            self.mounted
                .create_semantic_surface_for(
                    crate::facade::mounted::UiMountedProjectionAudience::full(),
                )
                .map_err(crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial::Mounted)?;
        self.materialize_initial_appearance_theme_binding(surface)
            .map_err(crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial::Mounted)?;
        self.authored_overlay_bindings.install_declared_surface(
            &generation,
            declaration,
            surface,
        )?;
        Ok(surface)
    }

    pub(crate) fn admit_authored_portal_open(
        &self,
        declaration: worth_ui_dsl::UiPortalDeclarationId,
        portal: crate::runtime::portal::UiPortalIdentity,
        runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<
        crate::runtime::portal::UiPortalOverlayBindingStage,
        crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial,
    > {
        let portal_state = self.portal.as_ref().ok_or(
            crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial::DeclaredSurfaceUnbound,
        )?;
        self.authored_overlay_bindings.admit_open(
            &self.active_generation_identity(),
            self.application.authored_overlay_material(),
            &self.mounted,
            portal_state,
            declaration,
            portal,
            runtime_surface,
        )
    }

    pub(crate) fn authored_portal_policy(
        &self,
        declaration: worth_ui_dsl::UiPortalDeclarationId,
    ) -> Option<crate::declaration::UiPortalPolicy> {
        self.application
            .authored_overlay_material()
            .portal_anchor_binding(declaration)
            .map(|binding| binding.policy())
    }

    pub(crate) fn commit_authored_overlay_binding(
        &mut self,
        commit: crate::runtime::portal::UiPortalOverlayBindingCommit,
    ) -> Result<(), crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial> {
        self.authored_overlay_bindings
            .commit_published(&self.active_generation_identity(), commit)
    }

    #[cfg(test)]
    pub(crate) fn authored_overlay_binding_exports(
        &self,
    ) -> Result<
        Box<[crate::runtime::portal::UiPortalOverlayBindingOwnerExport]>,
        crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial,
    > {
        self.authored_overlay_bindings
            .exports(&self.active_generation_identity(), self.portal.as_ref())
    }

    pub(super) fn clear_authored_overlay_bindings(&mut self) {
        self.authored_overlay_bindings.clear_for_shutdown();
    }
}
